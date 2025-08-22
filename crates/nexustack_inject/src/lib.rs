/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![warn(clippy::style)]
#![warn(clippy::perf)]
#![warn(clippy::complexity)]
#![warn(clippy::suspicious)]
#![warn(clippy::correctness)]
#![allow(clippy::redundant_pub_crate)]
#![cfg_attr(feature = "unsize", feature(unsize))]

mod container;
mod injectable;
mod injection_error;
mod injector;
mod service_collection;
mod service_provider;
mod service_scope;
mod service_token;
mod utils;

#[cfg(feature = "derive")]
extern crate nexustack_inject_macros;

#[cfg(feature = "derive")]
pub use nexustack_inject_macros::injectable;

pub use injectable::{FromInjector, Injectable};
pub use injection_error::{
    ConstructionError, ConstructionResult, InjectionError, InjectionResult, IntoConstructionResult,
};
pub use injector::Injector;
pub use service_collection::ServiceCollection;
pub use service_provider::ServiceProvider;
pub use service_scope::ServiceScope;
pub use service_token::ServiceToken;
