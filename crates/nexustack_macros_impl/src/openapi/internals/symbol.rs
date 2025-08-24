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

pub const ALIAS: Symbol = Symbol("alias");
pub const API_PROPERTY: Symbol = Symbol("api_property");
pub const API_VARIANT: Symbol = Symbol("api_variant");
pub const BORROW: Symbol = Symbol("borrow");
pub const BOUND: Symbol = Symbol("bound");
pub const CONTENT: Symbol = Symbol("content");
pub const CRATE: Symbol = Symbol("crate");
pub const DEFAULT: Symbol = Symbol("default");
pub const DENY_UNKNOWN_FIELDS: Symbol = Symbol("deny_unknown_fields");
pub const DEPRECATED: Symbol = Symbol("deprecated");
pub const DESCRIPTION: Symbol = Symbol("description");
pub const DESERIALIZE_WITH: Symbol = Symbol("deserialize_with");
pub const DESERIALIZE: Symbol = Symbol("deserialize");
pub const DOC: Symbol = Symbol("doc");
pub const EXPECTING: Symbol = Symbol("expecting");
pub const FIELD_IDENTIFIER: Symbol = Symbol("field_identifier");
pub const FLATTEN: Symbol = Symbol("flatten");
pub const FROM: Symbol = Symbol("from");
pub const GETTER: Symbol = Symbol("getter");
pub const INTO: Symbol = Symbol("into");
pub const NON_EXHAUSTIVE: Symbol = Symbol("non_exhaustive");
pub const OTHER: Symbol = Symbol("other");
pub const READ: Symbol = Symbol("read");
pub const REMOTE: Symbol = Symbol("remote");
pub const RENAME_ALL_FIELDS: Symbol = Symbol("rename_all_fields");
pub const RENAME_ALL: Symbol = Symbol("rename_all");
pub const RENAME: Symbol = Symbol("rename");
pub const SERDE: Symbol = Symbol("serde");
pub const SERIALIZE_WITH: Symbol = Symbol("serialize_with");
pub const SERIALIZE: Symbol = Symbol("serialize");
pub const SKIP_DESERIALIZING: Symbol = Symbol("skip_deserializing");
pub const SKIP_SERIALIZING_IF: Symbol = Symbol("skip_serializing_if");
pub const SKIP_SERIALIZING: Symbol = Symbol("skip_serializing");
pub const SKIP: Symbol = Symbol("skip");
pub const TAG: Symbol = Symbol("tag");
pub const TRANSPARENT: Symbol = Symbol("transparent");
pub const TRY_FROM: Symbol = Symbol("try_from");
pub const UNTAGGED: Symbol = Symbol("untagged");
pub const VARIANT_IDENTIFIER: Symbol = Symbol("variant_identifier");
pub const WITH: Symbol = Symbol("with");
pub const WRITE: Symbol = Symbol("write");

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
