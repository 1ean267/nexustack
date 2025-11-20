/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    Callsite,
    openapi::{error, schema::builder::SchemaId},
};
use thiserror::Error;

/// Trait used by [`Schema`] implementations to generically construct errors belonging to the [`SchemaBuilder`] against which they are currently running.
///
/// # Example
///
/// ```rust
/// #[derive(Debug, PartialEq)]
/// struct Error(String);
///
/// impl nexustack::openapi::Error for Error {
///     fn custom<T>(msg: T) -> Self
///         where
///             T: std::fmt::Display {
///         Self(msg.to_string())
///     }
/// }
///
/// impl std::fmt::Display for Error {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
///         f.write_str(&self.0)
///     }
/// }
///
/// impl std::error::Error for Error { }
/// ```
///
/// [`Schema`]: crate::openapi::schema::Schema
/// [`SchemaBuilder`]: crate::openapi::schema_builder::SchemaBuilder
pub trait Error: Sized + std::error::Error {
    /// Used when a [`Schema`] implementation encounters any error while describing a type.
    ///
    /// The message should not be capitalized and should not end with a period.
    ///
    /// # Paramaters
    /// - `msg` - The error message to be included in the custom error. Must implement [`std::fmt::Display`].
    ///
    /// [`Schema`]: crate::openapi::schema::Schema
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display;
}

#[derive(Clone, Debug, Error)]
#[non_exhaustive]
pub enum DocumentGenerationError {
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

    /// Raised when a response for the same status code is defined multiple times for the same operation.
    #[error("duplicate response definition for status code {status_code}")]
    DuplicateResponseDefinition { status_code: u16 },

    /// Raised when a content type is defined multiple times for the same response and status code.
    #[error("duplicate content type definition for {content_type}")]
    DuplicateContentType { content_type: &'static str },

    /// Raised when a security requirement with the same name is defined multiple times for the same operation.
    #[error("duplicate security requirement definition for {name}")]
    DuplicateSecurityRequirement { name: &'static str },

    /// Raised when a request body is defined without any content type.
    #[error("request body must have at least one content type")]
    RequestBodyMustHaveContentType,

    /// Raised when an unsupported HTTP method is used.
    #[error("unsupported HTTP method: {method}")]
    UnsupportedHttpMethod { method: &'static str },

    /// Raised when an operation with the same HTTP method is defined multiple times for the same path.
    #[error("duplicate operation definition for {method} at {path}")]
    DuplicateOperation {
        method: &'static str,
        path: &'static str,
    },

    /// Raised when a custom error is thrown during the construction of a schema.
    #[error("schema cannot be constructed due to an error")]
    Custom(
        /// The underlying construction error
        String,
    ),
}

impl DocumentGenerationError {
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

impl error::Error for DocumentGenerationError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::custom(msg)
    }
}
