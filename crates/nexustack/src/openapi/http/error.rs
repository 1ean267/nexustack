/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use std::borrow::Cow;

use crate::{
    Callsite,
    openapi::{self, SchemaGenerationError, SchemaId},
};

/// An error that occurs during HTTP document generation for `OpenAPI`.
///
/// This enum represents various failure modes that can occur when generating
/// `OpenAPI` documentation for HTTP endpoints in the nexustack framework. These
/// errors help identify issues with API definitions, such as conflicting schemas,
/// duplicate definitions, or unsupported configurations.
///
/// # Error Variants
///
/// * `ConflictingDefinition` - Multiple schemas with the same name
/// * `DuplicateResponseDefinition` - Multiple responses for the same status code
/// * `DuplicateContentType` - Multiple content types for the same response
/// * `DuplicateSecurityRequirement` - Multiple security requirements with the same name
/// * `RequestBodyMustHaveContentType` - Request body without content type
/// * `UnsupportedHttpMethod` - HTTP method not supported by `OpenAPI`
/// * `DuplicateOperation` - Multiple operations with the same method and path
/// * `Custom` - Custom error with a descriptive message
///
/// # Examples
///
/// Errors are typically handled by the framework's error reporting system,
/// but can be inspected manually:
///
/// ```rust,ignore
/// match generate_openapi_doc() {
///     Ok(doc) => println!("Generated: {:?}", doc),
///     Err(HttpDocumentGenerationError::ConflictingDefinition { schema_id, conflicting_callsite }) => {
///         println!("Schema '{}' defined at both {} and {}",
///                  schema_id.name(), schema_id.callsite(), conflicting_callsite);
///     }
///     // ... handle other variants
/// }
/// ```
#[derive(Clone, Debug, thiserror::Error)]
#[non_exhaustive]
pub enum HttpDocumentGenerationError {
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

    /// Raised when a response for the same status code is defined multiple times for the same operation.
    ///
    /// HTTP operations can only have one response definition per status code.
    /// Attempting to define multiple responses for the same status code within
    /// a single operation will trigger this error.
    #[error("duplicate response definition for status code {status_code}")]
    DuplicateResponseDefinition {
        /// The HTTP status code that has duplicate response definitions.
        status_code: u16,
    },

    /// Raised when a content type is defined multiple times for the same response and status code.
    ///
    /// Each response can specify multiple content types, but each content type
    /// can only be defined once per response. This prevents duplicate MIME type
    /// definitions within the same response.
    #[error("duplicate content type definition for {content_type}")]
    DuplicateContentType {
        /// The content type that is duplicated.
        content_type: Cow<'static, str>,
    },

    #[error("duplicate response header definition for {name}")]
    DuplicateResponseHeader { name: http::header::HeaderName },

    #[error("illegal response header name {name}")]
    IllegalResponseHeader { name: http::header::HeaderName },

    #[error("duplicate response cookie definition for {name}")]
    DuplicateResponseCookie { name: Cow<'static, str> },

    /// Raised when a security requirement with the same name is defined multiple times for the same operation.
    ///
    /// Security requirements within an operation must have unique names to avoid
    /// conflicts in the `OpenAPI` security definitions.
    #[error("duplicate security requirement definition for {name}")]
    DuplicateSecurityRequirement {
        /// The name of the duplicated security requirement.
        name: Cow<'static, str>,
    },

    /// Raised when a request body is defined without any content type.
    ///
    /// `OpenAPI` requires that request bodies specify at least one content type
    /// to indicate how the body should be interpreted. This error ensures
    /// proper request body definitions.
    #[error("request body must have at least one content type")]
    RequestBodyMustHaveContentType,

    /// Raised when an unsupported HTTP method is used.
    ///
    /// The framework only supports certain HTTP methods for `OpenAPI` generation.
    /// Using an unsupported method will result in this error.
    #[error("unsupported HTTP method: {method}")]
    UnsupportedHttpMethod {
        /// The HTTP method that is not supported.
        method: Cow<'static, str>,
    },

    /// Raised when an operation with the same HTTP method is defined multiple times for the same path.
    ///
    /// Each path can only have one operation per HTTP method. Defining multiple
    /// operations with the same method and path creates ambiguity.
    #[error("duplicate operation definition for {method} at {path}")]
    DuplicateOperation {
        /// The HTTP method of the duplicated operation.
        method: Cow<'static, str>,
        /// The path of the duplicated operation.
        path: Cow<'static, str>,
    },

    /// Raised when a custom error is thrown during the construction of a schema.
    ///
    /// This variant wraps custom error messages that may be generated during
    /// schema construction or other processing steps.
    #[error("schema cannot be constructed due to an error")]
    Custom(
        /// The underlying construction error
        String,
    ),
}

impl HttpDocumentGenerationError {
    /// Creates a new `HttpDocumentGenerationError::Custom` with the given message.
    ///
    /// This method is used to construct custom errors during HTTP document generation.
    /// The message should provide a clear description of what went wrong.
    ///
    /// # Parameters
    ///
    /// * `msg` - The error message, which can be any type that implements `Display`
    ///
    /// # Returns
    ///
    /// A new `HttpDocumentGenerationError::Custom` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// let error = HttpDocumentGenerationError::custom("Invalid schema configuration");
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

impl openapi::error::Error for HttpDocumentGenerationError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::custom(msg)
    }
}

impl From<SchemaGenerationError> for HttpDocumentGenerationError {
    fn from(value: SchemaGenerationError) -> Self {
        match value {
            SchemaGenerationError::ConflictingDefinition {
                schema_id,
                conflicting_callsite,
            } => Self::conflicting_definition(schema_id, conflicting_callsite),
            SchemaGenerationError::Custom(msg) => Self::custom(msg),
        }
    }
}
