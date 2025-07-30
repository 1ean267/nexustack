/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    container::container_entry::{ContainerEntry, UntypedContainerEntry},
    injection_error::{ConstructionError, ConstructionResult, InjectionError, InjectionResult},
    injector::Injector,
    service_token::ServiceToken,
};
use std::sync::Arc;

// aka service-descriptor
pub(crate) trait ScopedUntypedContainerEntryBuilder: UntypedContainerEntryBuilder {
    fn to_builder(&self) -> Box<dyn UntypedContainerEntryBuilder>;
}

pub(crate) trait UntypedContainerEntryBuilder {
    fn service_token(&self) -> ServiceToken;
    fn build(
        self: Box<Self>,
        injector: &Injector,
    ) -> Box<dyn UntypedContainerEntry + Send + Sync + 'static>;
}

pub(crate) struct TransientContainerEntryBuilder<TService> {
    factory: Box<dyn Fn(&Injector) -> ConstructionResult<TService> + Send + Sync>,
}

impl<TService> TransientContainerEntryBuilder<TService> {
    pub(crate) fn new(
        factory: impl Fn(&Injector) -> ConstructionResult<TService> + Send + Sync + 'static,
    ) -> Self {
        Self {
            factory: Box::new(factory),
        }
    }
}

impl<TService: Send + Sync + 'static> UntypedContainerEntryBuilder
    for TransientContainerEntryBuilder<TService>
{
    fn service_token(&self) -> ServiceToken {
        ServiceToken::create::<TService>()
    }

    fn build(
        self: Box<Self>,
        _injector: &Injector,
    ) -> Box<dyn UntypedContainerEntry + Send + Sync + 'static> {
        Box::new(ContainerEntry::transient(self.factory))
    }
}

pub(crate) struct SingletonContainerEntryBuilder<TService> {
    factory: Box<dyn FnOnce(&Injector) -> ConstructionResult<TService>>,
    clone_resolved: fn(service: &InjectionResult<TService>) -> InjectionResult<TService>,
}

impl<TService: Clone> SingletonContainerEntryBuilder<TService> {
    pub(crate) fn new(
        factory: impl FnOnce(&Injector) -> ConstructionResult<TService> + 'static,
    ) -> Self {
        Self {
            factory: Box::new(factory),
            clone_resolved: InjectionResult::<TService>::clone,
        }
    }
}

impl<TService: Send + Sync + 'static> UntypedContainerEntryBuilder
    for SingletonContainerEntryBuilder<TService>
{
    fn service_token(&self) -> ServiceToken {
        ServiceToken::create::<TService>()
    }

    fn build(
        self: Box<Self>,
        injector: &Injector,
    ) -> Box<dyn UntypedContainerEntry + Send + Sync + 'static> {
        let factory = self.factory;
        Box::new(ContainerEntry::singleton(
            factory(injector).map_err(|err| match err {
                ConstructionError::InjectionError(injection_error) => injection_error,
                ConstructionError::Custom(error) => InjectionError::Custom {
                    service: ServiceToken::create::<TService>(),
                    // TODO: Is this correct?
                    dependency_chain: injector.resolve_dependency_chain(),
                    source: error.into(),
                },
            }),
            self.clone_resolved,
        ))
    }
}

#[derive(Clone)]
pub(crate) struct ScopedContainerEntryBuilder<TService> {
    factory: Arc<dyn Fn(&Injector) -> ConstructionResult<TService> + Send + Sync>,
    clone_resolved: fn(service: &InjectionResult<TService>) -> InjectionResult<TService>,
}

impl<TService: Clone> ScopedContainerEntryBuilder<TService> {
    pub(crate) fn new(
        factory: impl Fn(&Injector) -> ConstructionResult<TService> + Send + Sync + 'static,
    ) -> Self {
        Self {
            factory: Arc::new(factory),
            clone_resolved: InjectionResult::<TService>::clone,
        }
    }
}

impl<TService: Clone + Send + Sync + 'static> ScopedUntypedContainerEntryBuilder
    for ScopedContainerEntryBuilder<TService>
{
    fn to_builder(&self) -> Box<dyn UntypedContainerEntryBuilder> {
        Box::new(self.clone())
    }
}

impl<TService: Clone + Send + Sync + 'static> UntypedContainerEntryBuilder
    for ScopedContainerEntryBuilder<TService>
{
    fn service_token(&self) -> ServiceToken {
        ServiceToken::create::<TService>()
    }

    fn build(
        self: Box<Self>,
        injector: &Injector,
    ) -> Box<dyn UntypedContainerEntry + Send + Sync + 'static> {
        let factory = self.factory;

        Box::new(ContainerEntry::scoped(
            factory(injector).map_err(|err| match err {
                ConstructionError::InjectionError(injection_error) => injection_error,
                ConstructionError::Custom(error) => InjectionError::Custom {
                    service: ServiceToken::create::<TService>(),
                    // TODO: Is this correct?
                    dependency_chain: injector.resolve_dependency_chain(),
                    source: error.into(),
                },
            }),
            self.clone_resolved,
        ))
    }
}
