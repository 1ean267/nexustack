/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use nexustack_macros_impl::cron::{cron as cron_impl, cron_jobs as cron_jobs_impl};
use nexustack_macros_impl::inject::injectable as injectable_impl;
use nexustack_macros_impl::openapi::api_schema as api_schema_impl;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn injectable(attr: TokenStream, item: TokenStream) -> TokenStream {
    injectable_impl(attr.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn api_schema(attr: TokenStream, item: TokenStream) -> TokenStream {
    api_schema_impl(attr.into(), item.into()).into()
}

#[proc_macro_attribute]
#[cfg_attr(not(doctest), doc = include_str!("../../nexustack_macros_impl/src/cron/expand/CRON.md"))]
pub fn cron(attr: TokenStream, item: TokenStream) -> TokenStream {
    cron_impl(attr.into(), item.into()).into()
}

/// A macro to register multiple cron jobs with a configuration function.
///
/// This expands to a closure that configures the provided cron jobs.
#[proc_macro]
pub fn cron_jobs(input: TokenStream) -> TokenStream {
    cron_jobs_impl(input.into()).into()
}
