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

pub const NEXUSTACK: Symbol = Symbol("nexustack");
pub const CRATE: Symbol = Symbol("crate");

#[cfg(feature = "inject")]
pub const CTOR: Symbol = Symbol("ctor");

#[cfg(feature = "cron")]
pub const SCHEDULE: Symbol = Symbol("schedule");

#[cfg(feature = "cron")]
pub const SCHEDULE_WITH: Symbol = Symbol("schedule_with");

#[cfg(feature = "module")]
pub const FEATURES: Symbol = Symbol("features");

#[cfg(any(feature = "openapi", feature = "http"))]
pub const DESCRIPTION: Symbol = Symbol("description");

#[cfg(any(feature = "openapi", feature = "http"))]
pub const DEPRECATED: Symbol = Symbol("deprecated");

#[cfg(any(feature = "openapi", feature = "http"))]
pub const DOC: Symbol = Symbol("doc");

#[cfg(any(feature = "openapi", feature = "http"))]
pub const ALIAS: Symbol = Symbol("alias");

#[cfg(any(feature = "openapi", feature = "http"))]
pub const RENAME: Symbol = Symbol("rename");

#[cfg(any(feature = "openapi", feature = "http"))]
pub const DEFAULT: Symbol = Symbol("default");

#[cfg(feature = "inject")]
#[path = ""]
mod inject {
    use super::*;

    pub const INJECT: Symbol = Symbol("inject");
    pub const INJECTABLE: Symbol = Symbol("injectable");
}

#[cfg(feature = "inject")]
pub use inject::*;

#[cfg(feature = "openapi")]
#[path = ""]
mod optionapi {
    use super::*;

    pub const API_PROPERTY: Symbol = Symbol("api_property");
    pub const API_VARIANT: Symbol = Symbol("api_variant");
    pub const BORROW: Symbol = Symbol("borrow");
    pub const BOUND: Symbol = Symbol("bound");
    pub const CONTENT: Symbol = Symbol("content");
    pub const DENY_UNKNOWN_FIELDS: Symbol = Symbol("deny_unknown_fields");
    pub const DESERIALIZE_WITH: Symbol = Symbol("deserialize_with");
    pub const DESERIALIZE: Symbol = Symbol("deserialize");
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
}

#[cfg(feature = "openapi")]
pub use optionapi::*;

#[cfg(feature = "http")]
#[path = ""]
mod http {
    use super::*;

    pub const HTTP: Symbol = Symbol("http");
    pub const CONTROLLER: Symbol = Symbol("http_controller");
    pub const API_SKIP: Symbol = Symbol("api_skip");
    pub const DECODER: Symbol = Symbol("decoder");
    pub const ENCODER: Symbol = Symbol("encoder");
    pub const STATUS_CODE: Symbol = Symbol("status_code");
    pub const ROUTE: Symbol = Symbol("route");
    pub const TAGS: Symbol = Symbol("tags");
}

#[cfg(feature = "http")]
pub use http::*;

impl PartialEq<Ident> for Symbol {
    fn eq(&self, other: &Ident) -> bool {
        other == self.0
    }
}

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
