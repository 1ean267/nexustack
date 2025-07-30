/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    container::Container,
    injection_error::{InjectionError, InjectionResult},
    service_token::ServiceToken,
    utils::{atomic_once_cell::AtomicOnceCell, ensure_send, ensure_sync},
};
use std::sync::{Arc, Weak};

const _: () = ensure_send::<ServiceProvider>();
const _: () = ensure_sync::<ServiceProvider>();

#[derive(Clone)]
pub struct ServiceProvider {
    inner: ServiceProviderInner,
}

impl ServiceProvider {
    pub(crate) fn create(container: Arc<AtomicOnceCell<Container>>) -> Self {
        Self {
            inner: ServiceProviderInner::Container(container),
        }
    }

    pub(crate) fn create_weak(container: Weak<AtomicOnceCell<Container>>) -> Self {
        Self {
            inner: ServiceProviderInner::ContainerWeak(container),
        }
    }

    fn resolve_from_container<TService: 'static>(
        container: &Arc<AtomicOnceCell<Container>>,
    ) -> InjectionResult<TService> {
        match container.get() {
            Some(container) => container.resolve_core(None),
            None => Err(InjectionError::UninitializedServiceProvider {
                service: ServiceToken::create::<TService>(),
                // TODO: Dependency chain is missing here! (Is it possible this is not the root call from the caller?)
                dependency_chain: Vec::new(),
            }),
        }
    }

    pub fn resolve<TService: 'static>(&self) -> InjectionResult<TService> {
        match &self.inner {
            ServiceProviderInner::Container(container) => Self::resolve_from_container(container),
            ServiceProviderInner::ContainerWeak(container_weak) => match container_weak.upgrade() {
                Some(container) => Self::resolve_from_container(&container),
                None => Err(InjectionError::DroppedServiceProvider {
                    service: ServiceToken::create::<TService>(),
                    // TODO: Dependency chain is missing here! (Is it possible this is not the root call from the caller?)
                    dependency_chain: Vec::new(),
                }),
            },
        }
    }
}

#[derive(Clone)]
enum ServiceProviderInner {
    Container(Arc<AtomicOnceCell<Container>>),
    ContainerWeak(Weak<AtomicOnceCell<Container>>),
}
