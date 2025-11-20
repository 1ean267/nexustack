/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use std::{
    cmp::Ordering,
    fmt::{self, Display},
};
use syn::LitStr;

#[derive(Debug, Clone)]
pub struct Route {
    pub value: String,
    pub span: Span,
}

impl ToTokens for Route {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        LitStr::new(&self.value, self.span).to_tokens(tokens);
    }
}

impl Ord for Route {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.value, &other.value)
    }
}

impl PartialOrd for Route {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Eq for Route {}

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl From<&LitStr> for Route {
    fn from(lit: &LitStr) -> Self {
        Route {
            value: lit.value(),
            span: lit.span(),
        }
    }
}

impl Display for Route {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.value, formatter)
    }
}
