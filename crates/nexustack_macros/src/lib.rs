/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(any(feature = "inject", feature = "cron"))]
mod dummy;
#[cfg(feature = "openapi")]
#[macro_use]
mod fragment;
mod internals;

#[cfg(feature = "inject")]
mod inject;

#[cfg(feature = "openapi")]
mod openapi;

#[cfg(feature = "cron")]
mod cron;

#[cfg(feature = "inject")]
use crate::inject::injectable as injectable_impl;

#[cfg(feature = "openapi")]
use crate::openapi::api_schema as api_schema_impl;

#[cfg(feature = "cron")]
use crate::cron::{cron as cron_impl, cron_jobs as cron_jobs_impl};

#[cfg(feature = "inject")]
#[proc_macro_attribute]
pub fn injectable(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    injectable_impl(attr.into(), item.into()).into()
}

#[cfg(feature = "openapi")]
#[proc_macro_attribute]
pub fn api_schema(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    api_schema_impl(attr.into(), item.into()).into()
}

#[cfg(feature = "cron")]
#[proc_macro_attribute]
#[cfg_attr(not(doctest), doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "src/cron/CRON.md")))]
pub fn cron(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    cron_impl(attr.into(), item.into()).into()
}

/// A macro to register multiple cron jobs with a configuration function.
///
/// This expands to a closure that configures the provided cron jobs.
#[cfg(feature = "cron")]
#[proc_macro]
pub fn cron_jobs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    cron_jobs_impl(input.into()).into()
}
