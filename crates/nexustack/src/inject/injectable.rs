/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::inject::{injection_error::ConstructionResult, injector::Injector};
use std::sync::Arc;

// The injector is a non-send, non-sync type with a lifetime to ensure that it can not escape the
// [`FromInjector::from_injector`] factory function.

/// The implementing type is a service resolvable from an [`Injector`]. The provided injector can be
/// used to resolve dependencies of the implementing service
///
/// The provided `#[injectable]` macro can be used to auto-generate the implementation of this trait.
///
/// # Example
///
/// ```rust
/// use nexustack::inject::ConstructionResult;
/// use nexustack::inject::FromInjector;
/// use nexustack::inject::Injector;
///
/// #[derive(Clone)]
/// pub struct DependencyA { }
///
/// #[derive(Clone)]
/// pub struct DependencyB { }
///
///
/// #[derive(Clone)]
/// pub struct MyService {
///     dependency_a: DependencyA,
///     dependency_b: DependencyB,
/// }
///
/// impl FromInjector for MyService {
///     fn from_injector(injector: &Injector) -> ConstructionResult<Self>
///         where
///             Self: Sized
///     {
///         let dependency_a = injector.resolve::<DependencyA>()?;
///         let dependency_b = injector.resolve::<DependencyB>()?;
///
///         Ok(Self { dependency_a, dependency_b })
///     }
/// }
/// ```
pub trait FromInjector {
    /// Constructs the service, resolving all necessary dependencies from the provided [`Injector`]
    ///
    /// # Errors
    ///
    /// This function will return an error if either a dependency could not be resolved via the provided
    /// [`Injector`] resulting in a [`crate::inject::ConstructionError::InjectionError`]
    /// or the construction of the service itself errored resulting in a
    /// [`crate::inject::ConstructionError::Custom`] error.
    fn from_injector(injector: &Injector) -> ConstructionResult<Self>
    where
        Self: Sized;
}

/// Marks a type to be injectable as a dependency into other services.
///
/// This is a marker trait that requires
/// implementors to implement the [`FromInjector`] trait. Not all services that can be resolved from an injector
/// are also injectable into other services. If a service should only be constructed from an injector but never
/// be injectable into other services implement only the [`FromInjector`] trait and not the [`Injectable`] trait.
pub trait Injectable: FromInjector {}

impl<T: FromInjector> FromInjector for Arc<T> {
    fn from_injector(injector: &Injector) -> ConstructionResult<Self>
    where
        Self: Sized,
    {
        Ok(Self::new(T::from_injector(injector)?))
    }
}

impl<T: Injectable> Injectable for Arc<T> {}
