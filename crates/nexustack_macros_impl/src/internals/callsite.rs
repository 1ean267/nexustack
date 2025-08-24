/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use proc_macro2::{Span, TokenStream};
use quote::quote;

pub fn callsite(span: &Span) -> TokenStream {
    let file = span.file();
    let start = span.start();
    let line = start.line;
    let column = start.column;

    quote! { _nexustack::Callsite::new(#file, #line, #column) }
}
