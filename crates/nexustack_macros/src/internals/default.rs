/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/// Represents the default to use for a field when deserializing.
#[derive(Debug, PartialEq, Eq)]
pub enum Default {
    /// Field must always be specified because it does not have a default.
    None,
    /// The default is given by `std::default::Default::default()`.
    #[allow(clippy::enum_variant_names)]
    Default,
    /// The default is given by this function.
    Path(syn::ExprPath),
}

impl Default {
    pub fn is_none(&self) -> bool {
        match self {
            Default::None => true,
            Default::Default | Default::Path(_) => false,
        }
    }

    pub fn or<'a>(&'a self, other: &'a Default) -> &'a Default {
        if self.is_none() {
            return other;
        }

        self
    }
}
