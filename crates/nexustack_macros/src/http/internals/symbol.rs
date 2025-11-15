/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/symbol.rs
 */

use std::fmt::{self, Display};
use syn::{Ident, Path};

#[derive(Copy, Clone)]
pub struct Symbol(&'static str);

pub const API_SKIP: Symbol = Symbol("api_skip");
pub const CRATE: Symbol = Symbol("crate");
pub const DEPRECATED: Symbol = Symbol("deprecated");
pub const DESCRIPTION: Symbol = Symbol("description");
pub const DOC: Symbol = Symbol("doc");
pub const ENCODER: Symbol = Symbol("encoder");
pub const STATUS_CODE: Symbol = Symbol("status_code");
pub const HTTP_RESPONSE_VARIANT: Symbol = Symbol("http_response_variant");

impl PartialEq<Symbol> for Ident {
    fn eq(&self, word: &Symbol) -> bool {
        self == word.0
    }
}

impl PartialEq<Symbol> for &Ident {
    fn eq(&self, word: &Symbol) -> bool {
        *self == word.0
    }
}

impl PartialEq<Symbol> for Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl PartialEq<Symbol> for &Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl Display for Symbol {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.0)
    }
}
