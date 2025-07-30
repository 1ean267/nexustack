/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

mod container_builder;
mod container_entry;
mod container_entry_builder;

pub(crate) use container_builder::*;
pub(crate) use container_entry_builder::*;

use crate::{
    injection_error::{InjectionError, InjectionResult},
    injector::Injector,
    service_provider::ServiceProvider,
    service_token::ServiceToken,
};
use container_entry::{ContainerEntry, UntypedContainerEntry};
use std::{any::TypeId, collections::HashMap};

pub(crate) struct Container {
    entries: HashMap<TypeId, Box<dyn UntypedContainerEntry + Send + Sync>>,
    parent_service_provider: Option<ServiceProvider>,
}

impl Container {
    pub(crate) fn new(
        entries: HashMap<TypeId, Box<dyn UntypedContainerEntry + Send + Sync>>,
        parent_service_provider: Option<ServiceProvider>,
    ) -> Self {
        Self {
            entries,
            parent_service_provider,
        }
    }

    pub(crate) fn resolve_core<TService: 'static>(
        &self,
        parent_injector: Option<&Injector>,
    ) -> InjectionResult<TService> {
        let service_token = ServiceToken::create::<TService>();
        match self.entries.get(service_token.type_id()) {
            Some(entry) => {
                let typed_entry = entry
                    .as_any()
                    .downcast_ref::<ContainerEntry<TService>>()
                    .unwrap();

                let injector = Injector::from_container(self, service_token, parent_injector);

                typed_entry.resolve(&injector)
            }
            None => match &self.parent_service_provider {
                Some(parent_service_provider) => parent_service_provider.resolve(),
                None => Err(InjectionError::ServiceNotFound {
                    service: service_token,
                    // TODO: Validate that this is correct!
                    dependency_chain: parent_injector
                        .map(|parent_injector| parent_injector.resolve_dependency_chain())
                        .unwrap_or_default(),
                }),
            },
        }
    }
}
