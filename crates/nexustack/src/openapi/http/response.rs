/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::openapi::{Error, HttpContentTypeBuilder};

/// Builder for describing HTTP responses.
///
/// This trait provides methods for describing HTTP response status codes, their content types, and finalizing the response description.
pub trait HttpResponseBuilder: Sized {
    /// The output type produced when the response description is finalized.
    type Ok;
    /// The error type for response building.
    type Error: Error;
    /// Builder for describing the content type of the response.
    type ContentTypeBuilder<'a>: HttpContentTypeBuilder<Ok = (), Error = Self::Error>
    where
        Self: 'a;

    /// Describe a response for a given status code.
    ///
    /// # Paramaters
    /// - `status_code` - The HTTP status code (e.g., 200, 404).
    /// - `description` - Optional description for the response.
    /// - `deprecated` - Whether the response is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if response description fails due to invalid type information or builder-specific errors.
    fn describe_response<'a>(
        &'a mut self,
        status_code: u16,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::ContentTypeBuilder<'a>, Self::Error>;

    /// Describes an empty HTTP response for a given status code.
    ///
    /// This method is a convenience wrapper around `describe_response` for cases where
    /// the response does not have any content.
    ///
    /// # Paramaters
    ///
    /// - `status_code` - The HTTP status code (e.g., 204 for No Content).
    /// - `description` - Optional description for the response.
    /// - `deprecated` - Whether the response is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if the response description fails due to invalid type information
    /// or builder-specific errors.
    fn describe_empty_response(
        &mut self,
        status_code: u16,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<(), Self::Error> {
        let content_type_builder =
            HttpResponseBuilder::describe_response(self, status_code, description, deprecated)?;

        content_type_builder.end()
    }

    /// Collect and describe a response for a given status code.
    ///
    /// # Paramaters
    /// - `status_code` - The HTTP status code (e.g., 200, 404).
    /// - `description` - Optional description for the response.
    /// - `deprecated` - Whether the response is deprecated.
    /// - `describe` - A closure that describes the content type of the response.
    ///
    /// # Errors
    ///
    /// Returns an error if response description fails due to invalid type information or builder-specific errors.
    fn collect_response<'a, D>(
        &'a mut self,
        status_code: u16,
        description: Option<&'static str>,
        deprecated: bool,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(Self::ContentTypeBuilder<'a>) -> Result<(), Self::Error>,
    {
        describe(HttpResponseBuilder::describe_response(
            self,
            status_code,
            description,
            deprecated,
        )?)
    }

    /// Finalize the response description and return the result.
    ///
    /// # Errors
    ///
    /// Returns an error if finalization fails due to builder-specific errors.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// Trait for types that can describe themselves as HTTP responses.
pub trait HttpResponse {
    /// Describe the HTTP response using the provided response builder.
    ///
    /// # Paramaters
    /// - `response_builder` - A builder that constructs the HTTP response description.
    ///
    /// # Errors
    ///
    /// Returns an error if response description fails due to invalid type information or builder-specific errors.
    fn describe<B>(response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpResponseBuilder;
}
