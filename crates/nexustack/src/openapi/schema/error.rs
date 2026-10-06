/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    Callsite,
    openapi::{SchemaId, error},
};

// TODO: Schema generation error should include the type that failed and whenever the error
//       is passed to the root level, it should include the full path from the root schema to the failed
//       type.

/// Errors that can occur while generating `OpenAPI` schema definitions.
///
/// `SchemaGenerationError` is returned when schema construction fails due to
/// conflicting metadata or an underlying custom error during schema assembly.
/// It is used to surface meaningful diagnostics when schema generation cannot
/// proceed safely.
///
/// # Variants
///
/// * `ConflictingDefinition` - when two schemas are found with the same name
///   and would otherwise collide in the generated `OpenAPI` document.
/// * `Custom` - when a custom failure occurs during schema generation and a
///   free-form error message must be bubbled up.
///
/// # Examples
///
/// ```rust
/// # use nexustack::openapi::schema::error::SchemaGenerationError;
/// let error = SchemaGenerationError::custom("invalid schema configuration");
/// assert!(matches!(error, SchemaGenerationError::Custom(_)));
/// ```
#[derive(Clone, Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaGenerationError {
    /// Raised when another schema with the same name as the currently constructed one is defined.
    ///
    /// This error occurs when two or more schemas attempt to use the same name,
    /// which would create ambiguity in the `OpenAPI` specification. Each schema
    /// must have a unique name within the document.
    #[error(
        "conflicting definition of schema {} at {} and {}",
        schema_id.name(),
        schema_id.callsite(),
        conflicting_callsite,
    )]
    ConflictingDefinition {
        /// The schema ID that conflicts with another definition.
        schema_id: SchemaId,
        /// The callsite where the conflicting schema definition occurs.
        conflicting_callsite: Callsite,
    },

    /// Raised when a custom error is thrown during the construction of a schema.
    #[error("schema cannot be constructed due to an error")]
    Custom(
        /// The underlying construction error
        String,
    ),
}

impl SchemaGenerationError {
    /// Creates a new `SchemaGenerationError::Custom` with the given message.
    ///
    /// This method is used to construct custom errors during schema generation.
    /// The message should provide a clear description of what went wrong.
    ///
    /// # Parameters
    ///
    /// * `msg` - The error message, which can be any type that implements `Display`
    ///
    /// # Returns
    ///
    /// A new `SchemaGenerationError::Custom` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// let error = SchemaGenerationError::custom("Invalid schema configuration");
    /// ```
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

impl error::Error for SchemaGenerationError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::custom(msg)
    }
}
