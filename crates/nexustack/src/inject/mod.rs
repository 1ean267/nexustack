/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

mod container;
mod injectable;
mod injection_error;
mod injector;
mod service_collection;
mod service_provider;
mod service_scope;
mod service_token;

#[cfg(feature = "derive")]
pub use nexustack_macros::injectable;

pub use injectable::{FromInjector, Injectable};
pub use injection_error::{
    ConstructionError, ConstructionResult, InjectionError, InjectionResult, IntoConstructionResult,
};
pub use injector::Injector;
pub use service_collection::ServiceCollection;
pub use service_provider::ServiceProvider;
pub use service_scope::ServiceScope;
pub use service_token::ServiceToken;
