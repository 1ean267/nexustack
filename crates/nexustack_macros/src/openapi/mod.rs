/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

#[macro_use]
mod bound;
mod dummy;
mod expand;
mod generics;
mod internals;
mod serde;

pub use expand::expand_api_schema;

use proc_macro2::TokenStream;

pub fn api_schema(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = match syn::parse2::<syn::DeriveInput>(input) {
        Ok(data) => data,
        Err(err) => {
            return err.to_compile_error();
        }
    };

    expand_api_schema(attr, &mut input).unwrap_or_else(syn::Error::into_compile_error)
}
