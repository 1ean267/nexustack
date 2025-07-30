/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    container::{Container, ContainerBuilder},
    injection_error::{InjectionError, InjectionResult},
    service_token::ServiceToken,
};
use std::{any::TypeId, marker::PhantomData};

pub struct Injector<'i> {
    inner: InjectorInner<'i>,
    service_token: ServiceToken,
    parent_injector: Option<&'i Injector<'i>>,
    _not_send_sync: PhantomData<*const ()>,
}

impl<'i> Injector<'i> {
    pub(crate) fn from_container(
        container: &'i Container,
        service_token: ServiceToken,
        parent_injector: Option<&'i Injector<'i>>,
    ) -> Self {
        Injector {
            inner: InjectorInner::Container(container),
            service_token,
            parent_injector,
            _not_send_sync: PhantomData::default(),
        }
    }

    pub(crate) fn from_container_builder(
        container_builder: &'i ContainerBuilder,
        service_token: ServiceToken,
        parent_injector: Option<&'i Injector<'i>>,
    ) -> Self {
        Injector {
            inner: InjectorInner::ContainerBuilder(container_builder),
            service_token,
            parent_injector,
            _not_send_sync: PhantomData::default(),
        }
    }

    fn has_service_type_in_chain(&self, service_type: TypeId) -> bool {
        self.service_token.type_id() == &service_type
            || self.parent_injector.map_or(false, |parent_injector| {
                parent_injector.has_service_type_in_chain(service_type)
            })
    }

    pub(crate) fn resolve_dependency_chain(&self) -> Vec<ServiceToken> {
        let mut result = Vec::new();
        result.push(self.service_token.clone());
        let mut curr = self.parent_injector;

        while let Some(injector) = curr {
            result.push(injector.service_token.clone());
            curr = injector.parent_injector;
        }

        return result;
    }

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
