/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://github.com/serde-rs/serde/blob/master/serde_derive/src/dummy.rs
 *
 * Licensed under the MIT license. See https://github.com/serde-rs/serde/blob/master/LICENSE-MIT
 */
use proc_macro2::TokenStream;
use quote::quote;

pub fn wrap_in_const(
    serde_path: Option<&syn::Path>,
    crate_path: Option<&syn::Path>,
    code: TokenStream,
) -> TokenStream {
    let use_serde = match serde_path {
        Some(path) => quote! {
            use #path as _serde;
        },
        None => quote! {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
        },
    };

    let use_crate = match crate_path {
        Some(path) => quote! {
            use #path as _nexustack;
        },
        None => quote! {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate nexustack as _nexustack;
        },
    };

    quote! {
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
            non_camel_case_types,
            deprecated,
            non_snake_case,
            dead_code,
        )]
        const _: () = {
            #use_serde
            #use_crate
            #code
        };
    }
}
