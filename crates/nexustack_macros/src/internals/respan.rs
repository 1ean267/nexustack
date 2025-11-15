/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/respan.rs
 */

use proc_macro2::{Group, Span, TokenStream, TokenTree};

pub(crate) fn respan(stream: TokenStream, span: Span) -> TokenStream {
    stream
        .into_iter()
        .map(|token| respan_token(token, span))
        .collect()
}

fn respan_token(mut token: TokenTree, span: Span) -> TokenTree {
    if let TokenTree::Group(g) = &mut token {
        *g = Group::new(g.delimiter(), respan(g.stream(), span));
    }
    token.set_span(span);
    token
}
