/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

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
