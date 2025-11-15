/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(rustdoc_internals))]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "../../README.md"))]

#[cfg(feature = "derive")]
extern crate nexustack_macros;

// Used by generated code and doc tests. Not public API.
#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;

#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "src/inject/README.md"))]
pub mod inject;

#[cfg(feature = "openapi")]
pub mod openapi;

#[cfg(feature = "cron")]
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "src/cron/README.md"))]
pub mod cron;

mod application;
mod callsite;
mod utils;

pub use application::{
    Application, ApplicationBuilder, ApplicationPart, ApplicationPartBuilder, Chain, Here, InHead,
    InTail, Index, Node, application_builder,
};
pub use callsite::Callsite;
