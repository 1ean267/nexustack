/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    container::Container,
    container::container_entry::{ContainerEntry, UntypedContainerEntry},
    container::container_entry_builder::{
        ScopedUntypedContainerEntryBuilder, SingletonContainerEntryBuilder,
        TransientContainerEntryBuilder, UntypedContainerEntryBuilder,
    },
    injection_error::{InjectionError, InjectionResult},
    injector::Injector,
    service_provider::ServiceProvider,
    service_scope::ServiceScope,
    service_token::ServiceToken,
    utils::atomic_once_cell::AtomicOnceCell,
};
use std::{any::TypeId, cell::RefCell, collections::HashMap, sync::Arc};

enum ContainerBuilderEntry {
    Builder(Box<dyn UntypedContainerEntryBuilder>),
    Entry(Box<dyn UntypedContainerEntry + Send + Sync>),
    Building,
}

impl ContainerBuilderEntry {
    fn unwrap_into_builder(self) -> Box<dyn UntypedContainerEntryBuilder> {
        match self {
            ContainerBuilderEntry::Builder(builder) => builder,
            _ => {
                panic!("Cannot unwrap into builder.")
            }
        }
    }

    fn unwrap_into_entry(self) -> Box<dyn UntypedContainerEntry + Send + Sync> {
        match self {
            ContainerBuilderEntry::Entry(entry) => entry,
            _ => {
                panic!("Cannot unwrap into entry.")
            }
        }
    }

    fn unwrap_entry_mut(&mut self) -> &mut Box<dyn UntypedContainerEntry + Send + Sync> {
        match self {
            ContainerBuilderEntry::Entry(entry) => entry,
            _ => {
                panic!("Cannot unwrap entry.")
            }
        }
    }
}

pub(crate) struct ContainerBuilder {
    entries: HashMap<TypeId, RefCell<ContainerBuilderEntry>>,
    container: Arc<AtomicOnceCell<Container>>,
    parent_service_provider: Option<ServiceProvider>,
}

impl ContainerBuilder {
    pub(crate) fn new(
        mut entry_builders: Vec<Box<dyn UntypedContainerEntryBuilder>>,
        scoped_builders: Option<Vec<Box<dyn ScopedUntypedContainerEntryBuilder + Send + Sync>>>,
        parent_service_provider: Option<ServiceProvider>,
    ) -> Self {
        let container: Arc<AtomicOnceCell<Container>> = Arc::new(AtomicOnceCell::new());
        let inner_service_provider = ServiceProvider::create_weak(Arc::downgrade(&container));

        if let Some(scoped_builders) = scoped_builders {
            let inner_service_provider = inner_service_provider.clone();
            entry_builders.push(Box::new(TransientContainerEntryBuilder::new(move |_| {
                let entry_builders = scoped_builders
                    .iter()
                    .map(|builder| builder.to_builder())
                    .collect::<Vec<_>>();

                let scope_container_builder = ContainerBuilder::new(
                    entry_builders,
                    None,
                    Some(inner_service_provider.clone()),
                );

                let scope_service_provider = scope_container_builder.build();

                Ok(ServiceScope::new(scope_service_provider))
            })));
        }

        entry_builders.push(Box::new(SingletonContainerEntryBuilder::new(|_| {
            Ok(inner_service_provider)
        })));

        Self {
            entries: entry_builders
                .into_iter()
                .map(|builder| {
                    let service_type = *builder.service_token().type_id();
                    let entry = RefCell::new(ContainerBuilderEntry::Builder(builder));

                    (service_type, entry)
                })
                .collect(),
            container,
            parent_service_provider,
        }
    }

    pub(crate) fn build(self) -> ServiceProvider {
        for entry in self.entries.values() {
            let mut entry = match entry.try_borrow_mut() {
                Ok(entry) => entry,
                Err(_) => {
                    unreachable!(
                        "The injector guarantees that there are no cyclic references, so that the ref-cell cannot be borrowed here."
                    )
                }
            };
            if let ContainerBuilderEntry::Builder(builder) = &*entry {
                let service_token = builder.service_token();
                let injector = Injector::from_container_builder(&self, service_token, None);
                let mut builder_or_entry = ContainerBuilderEntry::Building;
                std::mem::swap(&mut *entry, &mut builder_or_entry);

                let builder = builder_or_entry.unwrap_into_builder();
                builder_or_entry = ContainerBuilderEntry::Entry(builder.build(&injector));
                std::mem::swap(&mut *entry, &mut builder_or_entry);
            }
        }

        let entries = self
            .entries
            .into_iter()
            .map(|(service_type, entry)| (service_type, entry.into_inner().unwrap_into_entry()))
            .collect::<HashMap<_, _>>();

        let container = Container::new(entries, self.parent_service_provider);

        // TODO: Do not discard error!
        _ = self.container.set(container);

        ServiceProvider::create(self.container)
    }

    pub(crate) fn resolve_core<TService: 'static>(
        &self,
        parent_injector: Option<&Injector>,
    ) -> InjectionResult<TService> {
        let injector = Injector::from_container_builder(
            self,
            ServiceToken::create::<TService>(),
            parent_injector,
        );

        match self.entries.get(&TypeId::of::<TService>()) {
            Some(entry) => {
                let mut entry = match entry.try_borrow_mut() {
                    Ok(entry) => entry,
                    Err(_) => {
                        unreachable!(
                            "The injector guarantees that there are no cyclic references, so that the ref-cell cannot be borrowed here."
                        )
                    }
                };

                let entry = match &mut *entry {
                    ContainerBuilderEntry::Builder(_) => {
                        let mut builder_or_entry = ContainerBuilderEntry::Building;
                        std::mem::swap(&mut *entry, &mut builder_or_entry);
                        builder_or_entry = ContainerBuilderEntry::Entry(
                            builder_or_entry.unwrap_into_builder().build(&injector),
                        );
                        std::mem::swap(&mut *entry, &mut builder_or_entry);
                        entry.unwrap_entry_mut()
                    }
                    ContainerBuilderEntry::Entry(entry) => entry,
                    ContainerBuilderEntry::Building => {
                        unreachable!(
                            "The injector guarantees that there are no cyclic references, so that the ref-cell cannot be borrowed here."
                        )
                    }
                };

                let typed_entry = entry
                    .as_any()
                    .downcast_ref::<ContainerEntry<TService>>()
                    .unwrap();

                typed_entry.resolve(&injector)
            }
            None => match &self.parent_service_provider {
                Some(parent_service_provider) => parent_service_provider.resolve(),
                None => Err(InjectionError::ServiceNotFound {
                    service: ServiceToken::create::<TService>(),
                    // TODO: Validate that this is correct!
                    dependency_chain: parent_injector
                        .map(|parent_injector| parent_injector.resolve_dependency_chain())
                        .unwrap_or_default(),
                }),
            },
        }
    }
}
