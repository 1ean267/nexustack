/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::inject::{
    ConstructionResult, FromInjector,
    container::Container,
    injection_error::{InjectionError, InjectionResult},
    service_token::ServiceToken,
};
use crate::utils::{AtomicOnceCell, ensure_send, ensure_sync};
use std::sync::{Arc, Weak};

const _: () = ensure_send::<ServiceProvider>();
const _: () = ensure_sync::<ServiceProvider>();

/// Represents a service-provider built from a [`crate::inject::ServiceCollection`].
///
/// It  can be used to resolve services from the constructed dependency injection container
/// via its [`ServiceProvider::resolve`] function.
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

    fn construct_from_container<TService: FromInjector + 'static>(
        container: &Arc<AtomicOnceCell<Container>>,
    ) -> ConstructionResult<TService> {
        container.get().map_or_else(
            || {
                Err(InjectionError::UninitializedServiceProvider {
                    service: ServiceToken::create::<TService>(),
                    // TODO: Dependency chain is missing here! (Is it possible this is not the root call from the caller?)
                    dependency_chain: Vec::new(),
                }
                .into())
            },
            Container::construct_core,
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
    /// let mut services = ServiceCollection::new();
    /// services.add_singleton::<MyService>();
    /// let service_provider = services.build();
    ///
    /// let my_service = service_provider.resolve::<MyService>().unwrap();
    /// ```
    ///
    /// # Errors
    ///  * `crate::inject::InjectionError` when the service cannot be resolved either due to a resolution error or when a constructor/factory function
    ///    has raised a custom error. See the [`crate::inject::InjectionError`] enum for further information.
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

    /// Creates a service that implemented [`FromInjector`] with the required dependencies loaded from the provider.
    /// If the service cannot be created or a dependency cannot be resolved, a [`crate::inject::ConstructionError`] is returned.
    ///
    /// # Type arguments
    ///
    /// * `TService` - The type of the service to create.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexustack::inject::injectable;
    /// use nexustack::inject::ServiceCollection;
    /// use nexustack::inject::ServiceScope;
    /// use nexustack::inject::FromInjector;
    /// use nexustack::inject::Injector;
    /// use nexustack::inject::ConstructionResult;
    ///
    /// #[derive(Clone)]
    /// #[injectable]
    /// struct Dependency {}
    ///
    /// struct MyService(Dependency);
    ///
    /// impl FromInjector for MyService {
    ///     fn from_injector(injector: &Injector) -> ConstructionResult<Self> {
    ///         let dependency = injector.resolve::<Dependency>()?;
    ///         Ok(Self(dependency))
    ///     }
    /// }
    ///
    /// let mut services = ServiceCollection::new();
    /// services.add_singleton::<Dependency>();
    /// let service_provider = services.build();
    ///
    /// let my_service = service_provider.construct::<MyService>().unwrap();
    /// ```
    ///
    /// # Errors
    ///  * [`crate::inject::ConstructionError`] when the service cannot be created or one of its dependencies cannot be resolved
    ///    either due to a resolution error or when a constructor/factory function
    ///    has raised a custom error. See the [`crate::inject::ConstructionError`] enum for further information.
    ///
    pub fn construct<TService: FromInjector + 'static>(&self) -> ConstructionResult<TService> {
        match &self.inner {
            ServiceProviderInner::Container(container) => Self::construct_from_container(container),
            ServiceProviderInner::ContainerWeak(container_weak) => {
                container_weak.upgrade().map_or_else(
                    || {
                        Err(InjectionError::DroppedServiceProvider {
                            service: ServiceToken::create::<TService>(),
                            // TODO: Dependency chain is missing here! (Is it possible this is not the root call from the caller?)
                            dependency_chain: Vec::new(),
                        }
                        .into())
                    },
                    |container| Self::construct_from_container(&container),
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
