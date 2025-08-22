/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::inject::{
    container::Container,
    injection_error::{InjectionError, InjectionResult},
    service_token::ServiceToken,
};
use crate::utils::{atomic_once_cell::AtomicOnceCell, ensure_send, ensure_sync};
use std::sync::{Arc, Weak};

const _: () = ensure_send::<ServiceProvider>();
const _: () = ensure_sync::<ServiceProvider>();

/// Represents a service-provider built from a [`ServiceCollection`] that can be used to resolve services from the
/// constructed dependency injection container via its [`resolve`] function.
#[derive(Clone)]
pub struct ServiceProvider {
    inner: ServiceProviderInner,
}

impl ServiceProvider {
    pub(crate) const fn create(container: Arc<AtomicOnceCell<Container>>) -> Self {
        Self {
            inner: ServiceProviderInner::Container(container),
        }
    }

    pub(crate) const fn create_weak(container: Weak<AtomicOnceCell<Container>>) -> Self {
        Self {
            inner: ServiceProviderInner::ContainerWeak(container),
        }
    }

    fn resolve_from_container<TService: 'static>(
        container: &Arc<AtomicOnceCell<Container>>,
    ) -> InjectionResult<TService> {
        container.get().map_or_else(
            || {
                Err(InjectionError::UninitializedServiceProvider {
                    service: ServiceToken::create::<TService>(),
                    // TODO: Dependency chain is missing here! (Is it possible this is not the root call from the caller?)
                    dependency_chain: Vec::new(),
                })
            },
            |container| container.resolve_core(None),
        )
    }

    /// Resolves a service from the provider. If the service cannot be resolved, an [`InjectionError`] is returned.
    ///
    /// # Type arguments
    ///
    /// * `TService` - The type of the service to resolve from the provider.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexustack::inject::injectable;
    /// use nexustack::inject::ServiceCollection;
    /// use nexustack::inject::ServiceScope;
    ///
    /// #[derive(Clone)]
    /// struct MyService { }
    ///
    /// #[injectable]
    /// impl MyService {
    ///     pub fn new() -> Self {
    ///         Self { }
    ///     }
    /// }
    ///
    /// let service_provider = ServiceCollection::new()
    ///     .add_singleton::<MyService>()
    ///     .build();
    ///
    /// let my_service = service_provider.resolve::<MyService>().unwrap();
    /// ```
    ///
    /// # Errors
    ///  * `InjectionError` when the service cannot be resolved either due to a resolution error or when a constructor/factory function
    ///    has raised a custom error. See the [`InjectError`] enum for further information.
    ///  
    pub fn resolve<TService: 'static>(&self) -> InjectionResult<TService> {
        match &self.inner {
            ServiceProviderInner::Container(container) => Self::resolve_from_container(container),
            ServiceProviderInner::ContainerWeak(container_weak) => {
                container_weak.upgrade().map_or_else(
                    || {
                        Err(InjectionError::DroppedServiceProvider {
                            service: ServiceToken::create::<TService>(),
                            // TODO: Dependency chain is missing here! (Is it possible this is not the root call from the caller?)
                            dependency_chain: Vec::new(),
                        })
                    },
                    |container| Self::resolve_from_container(&container),
                )
            }
        }
    }
}

#[derive(Clone)]
enum ServiceProviderInner {
    Container(Arc<AtomicOnceCell<Container>>),
    ContainerWeak(Weak<AtomicOnceCell<Container>>),
}
