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
use syn::{Ident, LitStr};

#[derive(Debug, Clone)]
pub struct Name {
    pub value: String,
    pub span: Span,
}

impl ToTokens for Name {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        LitStr::new(&self.value, self.span).to_tokens(tokens);
    }
}

impl Ord for Name {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.value, &other.value)
    }
}

impl PartialOrd for Name {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Eq for Name {}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl From<&Ident> for Name {
    fn from(ident: &Ident) -> Self {
        Name {
            value: ident.to_string(),
            span: ident.span(),
        }
    }
}

impl From<&LitStr> for Name {
    fn from(lit: &LitStr) -> Self {
        Name {
            value: lit.value(),
            span: lit.span(),
        }
    }
}

impl Display for Name {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.value, formatter)
    }
}
