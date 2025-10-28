/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::inject::{
    container::{Container, ContainerBuilder},
    injection_error::{InjectionError, InjectionResult},
    service_token::ServiceToken,
};
use std::{any::TypeId, marker::PhantomData};

/// Represents a service injector that is used to perform the actual injection of services
/// into dependent services.
///
/// # Remarks
/// This type cannot be constructed manually but is used as a
/// Non-Static, Non-Send, Non-Sync proxy that cannot escape the calling stack of a service
/// factory function (see [`Injectable`]). A [`ServiceProvider`] can be resolved from the injector
/// (for example for lazy service retrieval) but it is only usable when the [`ServiceProvider`] or
/// [`ServiceScope`] is fully constructed. Prior use will result in a [`InjectionError`] at service
/// retrieval time. For a general purpose way to retrieve services, see the [`ServiceProvider`] type.
pub struct Injector<'i> {
    inner: InjectorInner<'i>,
    service_token: ServiceToken,
    parent: Option<&'i Self>,
    _not_send_sync: PhantomData<*const ()>,
}

impl<'i> Injector<'i> {
    pub(crate) const fn from_container(
        container: &'i Container,
        service_token: ServiceToken,
        parent_injector: Option<&'i Self>,
    ) -> Self {
        Injector {
            inner: InjectorInner::Container(container),
            service_token,
            parent: parent_injector,
            _not_send_sync: PhantomData,
        }
    }

    pub(crate) const fn from_container_builder(
        container_builder: &'i ContainerBuilder,
        service_token: ServiceToken,
        parent_injector: Option<&'i Self>,
    ) -> Self {
        Injector {
            inner: InjectorInner::ContainerBuilder(container_builder),
            service_token,
            parent: parent_injector,
            _not_send_sync: PhantomData,
        }
    }

    fn has_service_type_in_chain(&self, service_type: TypeId) -> bool {
        self.service_token.type_id() == &service_type
            || self.parent.is_some_and(|parent_injector| {
                parent_injector.has_service_type_in_chain(service_type)
            })
    }

    pub(crate) fn resolve_dependency_chain(&self) -> Vec<ServiceToken> {
        let mut result = Vec::new();
        result.push(self.service_token.clone());
        let mut curr = self.parent;

        while let Some(injector) = curr {
            result.push(injector.service_token.clone());
            curr = injector.parent;
        }

        result
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
    /// #[derive(Clone)]
    /// struct MyOtherService {
    ///     my_service: MyService
    /// }
    ///
    /// impl MyOtherService {
    ///     pub fn new(my_service: MyService) -> Self {
    ///         Self { my_service }
    ///     }
    /// }
    ///
    /// let mut services = ServiceCollection::new();
    /// services.add_singleton::<MyService>()
    ///     .add_singleton_factory(|injector| Ok(MyOtherService::new(injector.resolve::<MyService>()?)));
    /// let service_provider = services.build();
    ///
    /// let my_other_service = service_provider.resolve::<MyOtherService>().unwrap();
    /// ```
    ///
    /// # Errors
    ///  * `InjectionError` when the service cannot be resolved either due to a resolution error or when a constructor/factory function
    ///    has raised a custom error. See the [`InjectError`] enum for further information.
    ///
    pub fn resolve<TService: 'static>(&self) -> InjectionResult<TService> {
        if self.has_service_type_in_chain(TypeId::of::<TService>()) {
            return Err(InjectionError::CyclicReference {
                service: ServiceToken::create::<TService>(),
                dependency_chain: self.resolve_dependency_chain(),
            });
        }

        match self.inner {
            InjectorInner::Container(container) => container.resolve_core(Some(self)),
            InjectorInner::ContainerBuilder(container_builder) => {
                container_builder.resolve_core(Some(self))
            }
        }
    }
}

enum InjectorInner<'i> {
    Container(&'i Container),
    ContainerBuilder(&'i ContainerBuilder),
}
