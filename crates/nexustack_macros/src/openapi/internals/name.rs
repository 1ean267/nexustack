/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/name.rs
 */

use crate::internals::attr::{Attr, VecAttr};
use crate::internals::name::Name;
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct MultiName {
    pub(crate) serialize: Name,
    pub(crate) serialize_renamed: bool,
    pub(crate) deserialize: Name,
    pub(crate) deserialize_renamed: bool,
    pub(crate) deserialize_aliases: BTreeSet<Name>,
}

impl MultiName {
    pub(crate) fn from_attrs(
        source_name: Name,
        ser_name: Attr<Name>,
        de_name: Attr<Name>,
        de_aliases: Option<VecAttr<Name>>,
    ) -> Self {
        let mut alias_set = BTreeSet::new();
        if let Some(de_aliases) = de_aliases {
            for alias_name in de_aliases.get() {
                alias_set.insert(alias_name);
            }
        }

        let ser_name = ser_name.get();
        let ser_renamed = ser_name.is_some();
        let de_name = de_name.get();
        let de_renamed = de_name.is_some();
        MultiName {
            serialize: ser_name.unwrap_or_else(|| source_name.clone()),
            serialize_renamed: ser_renamed,
            deserialize: de_name.unwrap_or(source_name),
            deserialize_renamed: de_renamed,
            deserialize_aliases: alias_set,
        }
    }

    /// Return the container name for the container when serializing.
    pub fn serialize_name(&self) -> &Name {
        &self.serialize
    }

    /// Return the container name for the container when deserializing.
    pub fn deserialize_name(&self) -> &Name {
        &self.deserialize
    }

    pub(crate) fn deserialize_aliases(&self) -> &BTreeSet<Name> {
        &self.deserialize_aliases
    }
}
