/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

mod expand;

pub use expand::expand_injectable;

use proc_macro2::TokenStream;

pub fn injectable(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_injectable(attr, item).unwrap_or_else(syn::Error::into_compile_error)
}
