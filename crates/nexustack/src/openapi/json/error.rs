/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    Callsite,
    openapi::{error, schema_builder::SchemaId},
};
use thiserror::Error;

#[derive(Clone, Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Raised when a another schema with the same name as the currently constructed one is defined.
    #[error(
        "conflicting definition of schema {} at {} and {}",
        schema_id.name(),
        schema_id.callsite(),
        conflicting_callsite,
    )]
    ConflictingDefinition {
        schema_id: SchemaId,
        conflicting_callsite: Callsite,
    },
    /// Raised when a custom error is thrown during the construction of a schema.
    #[error("schema cannot be constructed due to an error")]
    Custom(
        /// The underlying construction error
        String,
    ),
}

impl Error {
    pub fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }

    pub(crate) const fn conflicting_definition(
        schema_id: SchemaId,
        conflicting_callsite: Callsite,
    ) -> Self {
        Self::ConflictingDefinition {
            schema_id,
            conflicting_callsite,
        }
    }
}

impl error::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::custom(msg)
    }
}
