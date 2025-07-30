/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    service_token::ServiceToken,
    utils::{ensure_clone, ensure_send, ensure_sync},
};
use std::{borrow::Cow, sync::Arc};
use thiserror::Error;

const _: () = ensure_send::<InjectionResult<String>>();
const _: () = ensure_sync::<InjectionResult<String>>();
const _: () = ensure_clone::<InjectionResult<String>>();

const _: () = ensure_send::<ConstructionResult<String>>();
const _: () = ensure_sync::<ConstructionResult<String>>();

pub type InjectionResult<T> = std::result::Result<T, InjectionError>;
pub type ConstructionResult<T> = std::result::Result<T, ConstructionError>;

fn format_service<'r>(
    service: &'r ServiceToken,
    dependency_chain: &[ServiceToken],
) -> Cow<'r, str> {
    if dependency_chain.is_empty() {
        return Cow::Borrowed(service.type_name());
    }

    let mut result = String::new();
    result.push_str(service.type_name());
    result.push(' ');
    result.push('(');

    for (index, dependency) in dependency_chain.iter().enumerate() {
        if index > 0 {
            result.push(' ');
            result.push('-');
            result.push('>');
            result.push(' ');
        }
        result.push_str(dependency.type_name());
    }

    result.push(')');

    Cow::Owned(result)
}

#[derive(Clone, Debug, Error)]
#[non_exhaustive]
pub enum InjectionError {
    #[error(
        "cannot resolve service {} from uninitialized service-provider",
        format_service(service, dependency_chain)
    )]
    UninitializedServiceProvider {
        service: ServiceToken,
        dependency_chain: Vec<ServiceToken>,
    },

    #[error(
        "cannot resolve service {} from dropped service-provider",
        format_service(service, dependency_chain)
    )]
    DroppedServiceProvider {
        service: ServiceToken,
        dependency_chain: Vec<ServiceToken>,
    },

    #[error(
        "cannot resolve service {} with cyclic reference",
        format_service(service, dependency_chain)
    )]
    CyclicReference {
        service: ServiceToken,
        dependency_chain: Vec<ServiceToken>,
    },

    #[error("service {} not found", format_service(service, dependency_chain))]
    ServiceNotFound {
        service: ServiceToken,
        dependency_chain: Vec<ServiceToken>,
    },

    #[error(
        "service {} cannot be constructed due to an error",
        format_service(service, dependency_chain)
    )]
    Custom {
        service: ServiceToken,
        dependency_chain: Vec<ServiceToken>,
        #[source]
        source: Arc<dyn std::error::Error + Send + Sync>,
    },
}

impl InjectionError {
    pub fn service(&self) -> &ServiceToken {
        match self {
            InjectionError::UninitializedServiceProvider {
                service,
                dependency_chain: _,
            } => service,
            InjectionError::DroppedServiceProvider {
                service,
                dependency_chain: _,
            } => service,
            InjectionError::CyclicReference {
                service,
                dependency_chain: _,
            } => service,
            InjectionError::ServiceNotFound {
                service,
                dependency_chain: _,
            } => service,
            InjectionError::Custom {
                service,
                dependency_chain: _,
                source: _,
            } => service,
        }
    }

    pub fn dependency_chain(&self) -> &Vec<ServiceToken> {
        match self {
            InjectionError::UninitializedServiceProvider {
                service: _,
                dependency_chain,
            } => dependency_chain,
            InjectionError::DroppedServiceProvider {
                service: _,
                dependency_chain,
            } => dependency_chain,
            InjectionError::CyclicReference {
                service: _,
                dependency_chain,
            } => dependency_chain,
            InjectionError::ServiceNotFound {
                service: _,
                dependency_chain,
            } => dependency_chain,
            InjectionError::Custom {
                service: _,
                dependency_chain,
                source: _,
            } => dependency_chain,
        }
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConstructionError {
    #[error(transparent)]
    InjectionError(#[from] InjectionError),

    #[error(transparent)]
    Custom(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub trait IntoConstructionResult {
    type Value;

    fn into_construction_result(self) -> ConstructionResult<Self::Value>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> IntoConstructionResult for Result<T, E> {
    type Value = T;

    fn into_construction_result(self) -> ConstructionResult<Self::Value> {
        match self {
            Ok(value) => ConstructionResult::Ok(value),
            Err(err) => ConstructionResult::Err(ConstructionError::Custom(Box::new(err))),
        }
    }
}
