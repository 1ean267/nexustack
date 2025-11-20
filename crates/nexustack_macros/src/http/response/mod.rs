/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

mod expand;
mod internals;

pub use expand::expand_http_response;

use proc_macro2::TokenStream;

pub fn http_response(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = match syn::parse2::<syn::DeriveInput>(input) {
        Ok(data) => data,
        Err(err) => {
            return err.to_compile_error();
        }
    };

    expand_http_response(attr, &mut input).unwrap_or_else(syn::Error::into_compile_error)
}
