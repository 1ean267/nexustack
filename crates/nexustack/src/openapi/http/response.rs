/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use std::{borrow::Cow, time::Duration};

use crate::openapi::{Error, IntoSchemaBuilder, http::HttpContentTypeBuilder};
use serde::Serialize;

#[derive(Debug, Copy, Clone)]
pub enum CookieSameSite {
    None,
    Lax,
    Strict,
}

pub trait HttpStatusResponseBuilder: HttpContentTypeBuilder {
    /// Builder for describing response cookie schemas.
    type ResponseCookieSchemaBuilder<'a>: IntoSchemaBuilder<Ok = (), Error = Self::SchemaBuilderError>
    where
        Self: 'a;

    /// Builder for describing response header schemas.
    type ResponseHeaderSchemaBuilder<'a>: IntoSchemaBuilder<Ok = (), Error = Self::SchemaBuilderError>
    where
        Self: 'a;

    /// Describe a cookie for the HTTP status response.
    ///
    /// # Paramaters
    /// - `name` - The name of the cookie.
    /// - `description` - Optional description for the cookie.
    /// - `deprecated` - Whether the cookie is deprecated.
    /// - `required` - An `Option` that specifies whether the cookie is required.
    ///   - `Some(true)` indicates the cookie is required.
    ///   - `Some(false)` indicates the cookie is optional.
    ///   - `None` allows the requiredness to be autodetected based on the schema.
    ///
    /// # Errors
    ///
    /// Returns an error if parameter description fails due to invalid type information or builder-specific errors.
    fn describe_response_cookie<'a>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        domain: Option<Cow<'static, str>>,
        path: Option<Cow<'static, str>>,
        max_age: Option<Duration>,
        same_site: CookieSameSite,
        deprecated: bool,
        required: bool,
        http_only: bool,
        secure: bool,
        partitioned: bool,
    ) -> Result<Self::ResponseCookieSchemaBuilder<'a>, Self::Error>;

    /// Collect and describe a cookie for the HTTP status response.
    ///
    /// # Paramaters
    /// - `name` - The name of the cookie.
    /// - `description` - Optional description for the cookie.
    /// - `deprecated` - Whether the cookie is deprecated.
    /// - `required` - An `Option` that specifies whether the cookie is required.
    ///   - `Some(true)` indicates the cookie is required.
    ///   - `Some(false)` indicates the cookie is optional.
    ///   - `None` allows the requiredness to be autodetected based on the schema.
    /// - `describe` - A closure that describes the schema of the cookie.
    ///
    /// # Errors
    ///
    /// Returns an error if parameter description fails due to invalid type information or builder-specific errors.
    fn collect_response_cookie<'a, D, E: Iterator<Item: Serialize + 'static>>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        domain: Option<Cow<'static, str>>,
        path: Option<Cow<'static, str>>,
        max_age: Option<Duration>,
        same_site: CookieSameSite,
        deprecated: bool,
        required: bool,
        http_only: bool,
        secure: bool,
        partitioned: bool,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(
            <Self::ResponseCookieSchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<E>,
        ) -> Result<(), Self::SchemaBuilderError>,
    {
        describe(
            HttpStatusResponseBuilder::describe_response_cookie(
                self,
                name,
                description,
                domain,
                path,
                max_age,
                same_site,
                deprecated,
                required,
                http_only,
                secure,
                partitioned,
            )?
            .into_schema_builder(),
        )
        .map_err(Into::into)
    }

    /// Describe a header for the HTTP status response.
    ///
    /// # Paramaters
    /// - `name` - The name of the header.
    /// - `description` - Optional description for the header.
    /// - `deprecated` - Whether the header is deprecated.
    /// - `required` - An `Option` that specifies whether the header is required.
    ///   - `Some(true)` indicates the header is required.
    ///   - `Some(false)` indicates the header is optional.
    ///   - `None` allows the requiredness to be autodetected based on the schema.
    ///
    /// # Errors
    ///
    /// Returns an error if parameter description fails due to invalid type information or builder-specific errors.
    fn describe_response_header<'a>(
        &'a mut self,
        name: http::header::HeaderName,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
    ) -> Result<Self::ResponseHeaderSchemaBuilder<'a>, Self::Error>;

    /// Collect and describe a header for the HTTP status response.
    ///
    /// # Paramaters
    /// - `name` - The name of the header.
    /// - `description` - Optional description for the header.
    /// - `deprecated` - Whether the header is deprecated.
    /// - `required` - An `Option` that specifies whether the header is required.
    ///   - `Some(true)` indicates the header is required.
    ///   - `Some(false)` indicates the header is optional.
    ///   - `None` allows the requiredness to be autodetected based on the schema.
    /// - `describe` - A closure that describes the schema of the header.
    ///
    /// # Errors
    ///
    /// Returns an error if parameter description fails due to invalid type information or builder-specific errors.
    fn collect_response_header<'a, D, E: Iterator<Item: Serialize + 'static>>(
        &'a mut self,
        name: http::header::HeaderName,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(
            <Self::ResponseHeaderSchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<E>,
        ) -> Result<(), Self::SchemaBuilderError>,
    {
        describe(
            HttpStatusResponseBuilder::describe_response_header(
                self,
                name,
                description,
                deprecated,
                required,
            )?
            .into_schema_builder(),
        )
        .map_err(Into::into)
    }
}

/// Builder for describing HTTP responses.
///
/// This trait provides methods for describing HTTP response status codes, their content types, and finalizing the response description.
pub trait HttpResponseBuilder: Sized {
    /// The output type produced when the response description is finalized.
    type Ok;
    /// The error type for response building.
    type Error: Error;
    /// Builder for describing the response for a given status of the response.
    type StatusResponseBuilder<'a>: HttpStatusResponseBuilder<Ok = (), Error = Self::Error>
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
    fn describe_status_response<'a>(
        &'a mut self,
        status_code: u16,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::StatusResponseBuilder<'a>, Self::Error>;

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
    fn describe_empty_status_response(
        &mut self,
        status_code: u16,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<(), Self::Error> {
        let content_type_builder = HttpResponseBuilder::describe_status_response(
            self,
            status_code,
            description,
            deprecated,
        )?;

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
    fn collect_status_response<'a, D>(
        &'a mut self,
        status_code: u16,
        description: Option<&'static str>,
        deprecated: bool,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(Self::StatusResponseBuilder<'a>) -> Result<(), Self::Error>,
    {
        describe(HttpResponseBuilder::describe_status_response(
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
