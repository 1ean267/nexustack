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

/// An injection result that represents the result of a dependency resolution
pub type InjectionResult<T> = std::result::Result<T, InjectionError>;

/// A construction result that represents the result of a service construction
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

/// An injection error representing an error in the resolution of a service
#[derive(Clone, Debug, Error)]
#[non_exhaustive]
pub enum InjectionError {
    /// Raised when the service provider is not initialized yet. It can only be used after
    /// the service-collection was built
    #[error(
        "cannot resolve service {} from uninitialized service-provider",
        format_service(service, dependency_chain)
    )]
    UninitializedServiceProvider {
        /// The [ServiceToken] that describes the service that failed to be resolved
        service: ServiceToken,

        /// The list of [ServiceTokens](ServiceToken) that describe the dependency chain
        /// of the service resolution operation
        dependency_chain: Vec<ServiceToken>,
    },

    /// Raised when the service provider was dropped and can no longer be used
    #[error(
        "cannot resolve service {} from dropped service-provider",
        format_service(service, dependency_chain)
    )]
    DroppedServiceProvider {
        /// The [ServiceToken] that describes the service that failed to be resolved
        service: ServiceToken,

        /// The list of [ServiceTokens](ServiceToken) that describe the dependency chain
        /// of the service resolution operation
        dependency_chain: Vec<ServiceToken>,
    },

    // TODO: Implement weak service reference and link the type here
    /// Raised when a service cannot be resolved due to a cyclic reference its dependency chain.
    /// This can be worked around by using a weak service reference
    #[error(
        "cannot resolve service {} with cyclic reference",
        format_service(service, dependency_chain)
    )]
    CyclicReference {
        /// The [ServiceToken] that describes the service that failed to be resolved
        service: ServiceToken,

        /// The list of [ServiceTokens](ServiceToken) that describe the dependency chain
        /// of the service resolution operation
        dependency_chain: Vec<ServiceToken>,
    },

    /// Raised when the requested service was not found in the service provider
    #[error("service {} not found", format_service(service, dependency_chain))]
    ServiceNotFound {
        /// The [ServiceToken] that describes the service that failed to be resolved
        service: ServiceToken,

        /// The list of [ServiceTokens](ServiceToken) that describe the dependency chain
        /// of the service resolution operation
        dependency_chain: Vec<ServiceToken>,
    },

    /// Raised when the requested service cannot be constructed as there occurred an error during its
    /// construction
    #[error(
        "service {} cannot be constructed due to an error",
        format_service(service, dependency_chain)
    )]
    Custom {
        /// The [ServiceToken] that describes the service that failed to be resolved
        service: ServiceToken,

        /// The list of [ServiceTokens](ServiceToken) that describe the dependency chain
        /// of the service resolution operation
        dependency_chain: Vec<ServiceToken>,

        /// The underlying construction error
        #[source]
        source: Arc<dyn std::error::Error + Send + Sync>,
    },
}

impl InjectionError {
    /// Accesses the [ServiceToken] that describes the service that failed to be resolved
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

    /// Accesses the list of [ServiceTokens](ServiceToken) that describe the dependency chain
    /// of the service resolution operation
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

/// An construction error representing an error in the construction of a service.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConstructionError {
    /// A service cannot be constructed as one of its dependencies cannot be resolved.
    /// Contains the error describing the cause of the dependency resolution failure.
    #[error(transparent)]
    InjectionError(#[from] InjectionError),

    /// A service cannot be constructed as the construction of the service itself errored.
    /// Contains the error describing the construction failure.
    #[error(transparent)]
    Custom(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Conversion into a [ConstructionResult].
///
/// By implementing [IntoConstructionResult] for a type, you define how it will be converted to a construction result.
/// This is common for services that take part in the dependency injection system.
pub trait IntoConstructionResult {
    // TODO: Rename Value to Service

    /// The type of service.
    type Value;

    /// Creates a [ConstructionResult] from a value.
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
