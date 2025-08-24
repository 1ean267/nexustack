/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "unsize", feature(unsize))]

#[cfg(feature = "derive")]
extern crate nexustack_macros;

// Used by generated code and doc tests. Not public API.
#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;

#[doc = include_str!("./inject/README.md")]
pub mod inject;

#[cfg(feature = "openapi")]
pub mod openapi;

mod callsite;
mod utils;

pub use callsite::Callsite;
