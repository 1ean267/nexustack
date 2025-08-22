/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use std::{
    any::{TypeId, type_name},
    fmt::Display,
};

/// Represents a service (i.e. a type) in the injection system.
#[derive(Debug, Clone)]
pub struct ServiceToken {
    type_id: TypeId,
    type_name: &'static str,
}

impl ServiceToken {
    pub(crate) fn create<TService: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<TService>(),
            type_name: type_name::<TService>(),
        }
    }

    /// The [type identifier](TypeId) as return by [`TypeId::of`].
    #[allow(clippy::must_use_candidate)]
    pub const fn type_id(&self) -> &TypeId {
        &self.type_id
    }

    /// A string slice containing the human readable type name that is provided in a best effort approach.
    #[allow(clippy::must_use_candidate)]
    pub const fn type_name(&self) -> &str {
        self.type_name
    }
}

impl Display for ServiceToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.type_name)
    }
}
