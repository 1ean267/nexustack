/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

mod expand;

pub use expand::{expand_cron, expand_cron_jobs};

use proc_macro2::TokenStream;

pub fn cron(attr: TokenStream, input: TokenStream) -> TokenStream {
    expand_cron(attr, input).unwrap_or_else(syn::Error::into_compile_error)
}

pub fn cron_jobs(input: TokenStream) -> TokenStream {
    expand_cron_jobs(input).unwrap_or_else(syn::Error::into_compile_error)
}
