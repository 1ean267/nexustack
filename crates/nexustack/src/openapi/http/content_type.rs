/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::openapi::{Error, IntoSchemaBuilder};
use serde::Serialize;

/// Builder for describing the content type of an HTTP response or operation request body.
///
/// This trait provides methods for describing content types, their schemas, and finalizing the content type description.
/// It is used for both HTTP response content types and operation request bodies in `OpenAPI` documentation.
pub trait HttpContentTypeBuilder {
    /// The output type produced when the content type description is finalized.
    type Ok;
    /// The error type for content type building.
    type Error: Error;
    /// Builder for describing the schema of the content type.
    type SchemaBuilder<'a>: IntoSchemaBuilder<Ok = (), Error = Self::Error>
    where
        Self: 'a;

    /// Describe a content type for the HTTP response.
    ///
    /// # Paramaters
    /// - `content_type` - The MIME type of the content (e.g., "application/json").
    /// - `description` - Optional description for the content type.
    /// - `deprecated` - Whether the content type is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if content type description fails due to invalid type information or builder-specific errors.
    fn describe_content_type<'a>(
        &'a mut self,
        content_type: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::SchemaBuilder<'a>, Self::Error>;

    /// Collect and describe a content type for the HTTP response.
    ///
    /// This method allows you to describe a content type and its schema using a closure.
    ///
    /// # Paramaters
    /// - `content_type` - The MIME type of the content (e.g., "application/json").
    /// - `description` - Optional description for the content type.
    /// - `deprecated` - Whether the content type is deprecated.
    /// - `describe` - A closure that describes the schema of the content type.
    ///
    /// # Errors
    ///
    /// Returns an error if content type description fails due to invalid type information or builder-specific errors.
    fn collect_content_type<'a, D, E: Iterator<Item: Serialize + 'static>>(
        &'a mut self,
        content_type: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(
            <Self::SchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<E>,
        ) -> Result<(), Self::Error>,
    {
        describe(
            HttpContentTypeBuilder::describe_content_type(
                self,
                content_type,
                description,
                deprecated,
            )?
            .into_schema_builder(),
        )
    }

    /// Finalize the content type description and return the result.
    ///
    /// # Errors
    ///
    /// Returns an error if finalization fails due to builder-specific errors.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// Trait for types that can describe themselves as HTTP content types.
///
/// This trait is used to define how a type can describe its content type in the context of an HTTP response or request body.
/// Implementations of this trait provide a `describe` method that uses a content type builder to define the content type.
///
/// # Examples
/// ```rust
/// use nexustack::openapi::{HttpContentType, HttpContentTypeBuilder};
///
/// struct MyContentType;
///
/// impl HttpContentType for MyContentType {
///     fn describe<B>(content_type_builder: B) -> Result<B::Ok, B::Error>
///     where
///         B: HttpContentTypeBuilder,
///     {
///         content_type_builder.describe_content_type("application/json", None, false)?.end()
///     }
/// }
/// ```
pub trait HttpContentType<T = Self> {
    /// Describe the HTTP content type using the provided content type builder.
    ///
    /// # Paramaters
    /// - `content_type_builder` - A builder that constructs the HTTP content type description.
    ///
    /// # Errors
    ///
    /// Returns an error if content type description fails due to invalid type information or builder-specific errors.
    fn describe<B>(content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpContentTypeBuilder;
}
