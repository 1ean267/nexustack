/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::inject::{
    injection_error::{ConstructionError, ConstructionResult, InjectionError, InjectionResult},
    injector::Injector,
    service_token::ServiceToken,
};
use std::any::Any;

type TransientServiceFactory<TService> =
    dyn Fn(&Injector) -> ConstructionResult<TService> + Send + Sync;

pub(crate) trait UntypedContainerEntry {
    fn as_any(&self) -> &dyn Any;
}

pub(crate) enum ContainerEntry<TService> {
    Transient(TransientContainerEntry<TService>),
    Singleton(SingletonContainerEntry<TService>),
    Scoped(ScopedContainerEntry<TService>),
}

impl<TService: 'static> ContainerEntry<TService> {
    pub(crate) fn transient(factory: Box<TransientServiceFactory<TService>>) -> Self {
        Self::Transient(TransientContainerEntry { factory })
    }

    pub(crate) fn singleton(
        resolved: InjectionResult<TService>,
        clone_resolved: fn(service: &InjectionResult<TService>) -> InjectionResult<TService>,
    ) -> Self {
        Self::Singleton(SingletonContainerEntry {
            resolved,
            clone_resolved,
        })
    }

    pub(crate) fn scoped(
        resolved: InjectionResult<TService>,
        clone_resolved: fn(service: &InjectionResult<TService>) -> InjectionResult<TService>,
    ) -> Self {
        Self::Scoped(ScopedContainerEntry {
            resolved,
            clone_resolved,
        })
    }

    pub(crate) fn resolve(&self, injector: &Injector) -> InjectionResult<TService> {
        match self {
            Self::Transient(transient) => transient.resolve(injector),
            Self::Singleton(singleton) => singleton.resolve(),
            Self::Scoped(scoped) => scoped.resolve(),
        }
    }
}

impl<TService: 'static> UntypedContainerEntry for ContainerEntry<TService> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub(crate) struct TransientContainerEntry<TService> {
    factory: Box<TransientServiceFactory<TService>>,
}

impl<TService: 'static> TransientContainerEntry<TService> {
    fn resolve(&self, injector: &Injector) -> InjectionResult<TService> {
        let factory = self.factory.as_ref();
        factory(injector).map_err(|err| match err {
            ConstructionError::InjectionError(injection_error) => injection_error,
            ConstructionError::Custom(error) => InjectionError::Custom {
                service: ServiceToken::create::<TService>(),
                // TODO: Is this correct?
                dependency_chain: injector.resolve_dependency_chain(),
                source: error.into(),
            },
        })
    }
}

pub(crate) struct SingletonContainerEntry<TService> {
    resolved: InjectionResult<TService>,
    clone_resolved: fn(service: &InjectionResult<TService>) -> InjectionResult<TService>,
}

impl<TService: 'static> SingletonContainerEntry<TService> {
    fn resolve(&self) -> InjectionResult<TService> {
        let clone_resolved = self.clone_resolved;
        clone_resolved(&self.resolved)
    }
}

pub(crate) struct ScopedContainerEntry<TService> {
    resolved: InjectionResult<TService>,
    clone_resolved: fn(service: &InjectionResult<TService>) -> InjectionResult<TService>,
}

impl<TService: 'static> ScopedContainerEntry<TService> {
    fn resolve(&self) -> InjectionResult<TService> {
        let clone_resolved = self.clone_resolved;
        clone_resolved(&self.resolved)
    }
}
