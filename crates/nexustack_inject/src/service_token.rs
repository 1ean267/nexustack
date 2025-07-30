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

#[derive(Debug, Clone)]
pub struct ServiceToken {
    type_id: TypeId,
    type_name: &'static str,
}

impl ServiceToken {
    pub fn create<TService: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<TService>(),
            type_name: type_name::<TService>(),
        }
    }

    pub fn type_id(&self) -> &TypeId {
        &self.type_id
    }

    pub fn type_name(&self) -> &str {
        self.type_name
    }
}

impl Display for ServiceToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.type_name)
    }
}
