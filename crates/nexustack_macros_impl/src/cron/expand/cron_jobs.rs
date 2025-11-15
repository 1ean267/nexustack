/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse::Parser as _, spanned::Spanned};

pub fn expand_cron_jobs(input: TokenStream) -> syn::Result<TokenStream> {
    let jobs =
        syn::punctuated::Punctuated::<syn::Type, syn::Token![,]>::parse_terminated.parse2(input)?;

    // TODO: Hygiene. This used CronRunner
    let expanded = jobs.iter().map(|job| {
        quote_spanned! { job.span()=> .add_cron_job::<#job>() }
    });

    Ok(quote! {
        |configure| {
            configure
                #(#expanded)*
                ;
        }
    })
}
