/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    Callsite,
    openapi::{Error, HttpContentTypeBuilder, HttpResponseBuilder, IntoSchemaBuilder},
};
use serde::Serialize;
use std::fmt::Display;

/// Identifier for an HTTP operation, including its name and callsite.
///
/// This struct is used to uniquely identify an HTTP operation definition within the `OpenAPI` builder.
/// It contains the name of the HTTP operation and the callsite information, which helps with tracking
/// where the HTTP operation was defined in the codebase. This is useful for documentation, debugging,
/// and ensuring HTTP operation name uniqueness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpOperationId {
    /// The name of the HTTP operation.
    name: &'static str,

    /// The callsite information.
    callsite: Callsite,
}

impl HttpOperationId {
    /// Create a new HTTP operation identifier.
    ///
    /// # Paramaters
    /// - `name` - The name of the HTTP operation.
    /// - `callsite` - The callsite information.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::HttpOperationId;
    /// use nexustack::callsite;
    ///
    /// callsite!(MyTypeCallsite);
    ///
    /// let id = HttpOperationId::new("MyType", *MyTypeCallsite);
    /// ```
    #[must_use]
    pub const fn new(name: &'static str, callsite: Callsite) -> Self {
        Self { name, callsite }
    }

    /// The name of the HTTP operation.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// The callsite information.
    #[must_use]
    pub const fn callsite(&self) -> &Callsite {
        &self.callsite
    }
}

impl Display for HttpOperationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ {}", self.name, self.callsite)
    }
}

/// Builder for describing HTTP security requirements.
///
/// This trait provides methods for describing security requirements and finalizing the security requirement description.
pub trait HttpSecurityRequirementBuilder {
    /// The output type produced when the security requirement description is finalized.
    type Ok;
    /// The error type for security requirement building.
    type Error: Error;

    /// Describe a security requirement for the HTTP operation.
    ///
    /// # Paramaters
    /// - `name` - The name of the security scheme.
    /// - `scopes` - Optional iterator over required scopes for the security scheme.
    ///
    /// # Errors
    ///
    /// Returns an error if security requirement description fails due to invalid type information or builder-specific errors.
    fn describe_requirement<S>(
        &mut self,
        name: &'static str,
        scopes: Option<S>,
    ) -> Result<(), Self::Error>
    where
        S: IntoIterator<Item = &'static str>;

    /// Finalize the security requirement description and return the result.
    ///
    /// # Errors
    ///
    /// Returns an error if finalization fails due to builder-specific errors.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// Builder for describing HTTP operations.
///
/// This trait provides methods for describing parameters, request bodies, security requirements, and finalizing the operation description.
pub trait HttpOperationBuilder {
    /// The output type produced when the operation description is finalized.
    type Ok;

    /// The error type for operation building.
    type Error: Error;

    /// Builder for describing parameter schemas.
    type ParameterSchemaBuilder<'a>: IntoSchemaBuilder<Ok = (), Error = Self::Error>
    where
        Self: 'a;

    /// Builder for describing request body schemas.
    type RequestBodySchemaBuilder<'a>: HttpContentTypeBuilder<Ok = (), Error = Self::Error>
    where
        Self: 'a;

    /// Builder for describing security requirements.
    type SecurityRequirementBuilder<'a>: HttpSecurityRequirementBuilder<Ok = (), Error = Self::Error>
    where
        Self: 'a;

    /// Builder for describing HTTP responses.
    type HttpResponseBuilder: HttpResponseBuilder<Ok = Self::Ok, Error = Self::Error>;

    // TODO: Style, example

    /// Describe a query parameter for the HTTP operation.
    ///
    /// # Paramaters
    /// - `name` - The name of the query parameter.
    /// - `description` - Optional description for the parameter.
    /// - `deprecated` - Whether the parameter is deprecated.
    /// - `required` - An `Option` that specifies whether the parameter is required.
    ///   - `Some(true)` indicates the parameter is required.
    ///   - `Some(false)` indicates the parameter is optional.
    ///   - `None` allows the requiredness to be autodetected based on the schema.
    ///
    /// # Errors
    ///
    /// Returns an error if parameter description fails due to invalid type information or builder-specific errors.
    fn describe_query_parameter<'a>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
    ) -> Result<Self::ParameterSchemaBuilder<'a>, Self::Error>;

    /// Collect and describe a query parameter for the HTTP operation.
    ///
    /// # Paramaters
    /// - `name` - The name of the query parameter.
    /// - `description` - Optional description for the parameter.
    /// - `deprecated` - Whether the parameter is deprecated.
    /// - `required` - An `Option` that specifies whether the parameter is required.
    ///   - `Some(true)` indicates the parameter is required.
    ///   - `Some(false)` indicates the parameter is optional.
    ///   - `None` allows the requiredness to be autodetected based on the schema.
    /// - `describe` - A closure that describes the schema of the parameter.
    ///
    /// # Errors
    ///
    /// Returns an error if parameter description fails due to invalid type information or builder-specific errors.
    fn collect_query_parameter<'a, D, E: Iterator<Item: Serialize + 'static>>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(
            <Self::ParameterSchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<E>,
        ) -> Result<(), Self::Error>,
    {
        describe(
            HttpOperationBuilder::describe_query_parameter(
                self,
                name,
                description,
                deprecated,
                required,
            )?
            .into_schema_builder(),
        )
    }

    // TODO: Style, example

    /// Describe a header parameter for the HTTP operation.
    ///
    /// # Paramaters
    /// - `name` - The name of the header parameter.
    /// - `description` - Optional description for the parameter.
    /// - `deprecated` - Whether the parameter is deprecated.
    /// - `required` - An `Option` that specifies whether the parameter is required.
    ///   - `Some(true)` indicates the parameter is required.
    ///   - `Some(false)` indicates the parameter is optional.
    ///   - `None` allows the requiredness to be autodetected based on the schema.
    ///
    /// # Errors
    ///
    /// Returns an error if parameter description fails due to invalid type information or builder-specific errors.
    fn describe_header_parameter<'a>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
    ) -> Result<Self::ParameterSchemaBuilder<'a>, Self::Error>;

    /// Collect and describe a header parameter for the HTTP operation.
    ///
    /// # Paramaters
    /// - `name` - The name of the header parameter.
    /// - `description` - Optional description for the parameter.
    /// - `deprecated` - Whether the parameter is deprecated.
    /// - `required` - An `Option` that specifies whether the parameter is required.
    ///   - `Some(true)` indicates the parameter is required.
    ///   - `Some(false)` indicates the parameter is optional.
    ///   - `None` allows the requiredness to be autodetected based on the schema.
    /// - `describe` - A closure that describes the schema of the parameter.
    ///
    /// # Errors
    ///
    /// Returns an error if parameter description fails due to invalid type information or builder-specific errors.
    fn collect_header_parameter<'a, D, E: Iterator<Item: Serialize + 'static>>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(
            <Self::ParameterSchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<E>,
        ) -> Result<(), Self::Error>,
    {
        describe(
            HttpOperationBuilder::describe_header_parameter(
                self,
                name,
                description,
                deprecated,
                required,
            )?
            .into_schema_builder(),
        )
    }

    // TODO: Style, example

    /// Describe a path parameter for the HTTP operation.
    ///
    /// # Paramaters
    /// - `name` - The name of the path parameter.
    /// - `description` - Optional description for the parameter.
    /// - `deprecated` - Whether the parameter is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if parameter description fails due to invalid type information or builder-specific errors.
    fn describe_path_parameter<'a>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::ParameterSchemaBuilder<'a>, Self::Error>;

    /// Collect and describe a path parameter for the HTTP operation.
    ///
    /// # Paramaters
    /// - `name` - The name of the path parameter.
    /// - `description` - Optional description for the parameter.
    /// - `deprecated` - Whether the parameter is deprecated.
    /// - `describe` - A closure that describes the schema of the parameter.
    ///
    /// # Errors
    ///
    /// Returns an error if parameter description fails due to invalid type information or builder-specific errors.
    fn collect_path_parameter<'a, D, E: Iterator<Item: Serialize + 'static>>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(
            <Self::ParameterSchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<E>,
        ) -> Result<(), Self::Error>,
    {
        describe(
            HttpOperationBuilder::describe_path_parameter(self, name, description, deprecated)?
                .into_schema_builder(),
        )
    }

    // TODO: Style, example

    /// Describe a cookie parameter for the HTTP operation.
    ///
    /// # Paramaters
    /// - `name` - The name of the cookie parameter.
    /// - `description` - Optional description for the parameter.
    /// - `deprecated` - Whether the parameter is deprecated.
    /// - `required` - An `Option` that specifies whether the parameter is required.
    ///   - `Some(true)` indicates the parameter is required.
    ///   - `Some(false)` indicates the parameter is optional.
    ///   - `None` allows the requiredness to be autodetected based on the schema.
    ///
    /// # Errors
    ///
    /// Returns an error if parameter description fails due to invalid type information or builder-specific errors.
    fn describe_cookie_parameter<'a>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
    ) -> Result<Self::ParameterSchemaBuilder<'a>, Self::Error>;

    /// Collect and describe a cookie parameter for the HTTP operation.
    ///
    /// # Paramaters
    /// - `name` - The name of the cookie parameter.
    /// - `description` - Optional description for the parameter.
    /// - `deprecated` - Whether the parameter is deprecated.
    /// - `required` - An `Option` that specifies whether the parameter is required.
    ///   - `Some(true)` indicates the parameter is required.
    ///   - `Some(false)` indicates the parameter is optional.
    ///   - `None` allows the requiredness to be autodetected based on the schema.
    /// - `describe` - A closure that describes the schema of the parameter.
    ///
    /// # Errors
    ///
    /// Returns an error if parameter description fails due to invalid type information or builder-specific errors.
    fn collect_cookie_parameter<'a, D, E: Iterator<Item: Serialize + 'static>>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(
            <Self::ParameterSchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<E>,
        ) -> Result<(), Self::Error>,
    {
        describe(
            HttpOperationBuilder::describe_cookie_parameter(
                self,
                name,
                description,
                deprecated,
                required,
            )?
            .into_schema_builder(),
        )
    }

    // TODO: File uploads have a request-body but no schema

    /// Describe the request body for the HTTP operation.
    ///
    /// # Paramaters
    /// - `description` - Optional description for the request body.
    /// - `deprecated` - Whether the request body is deprecated.
    /// - `required` - Whether the request body is required.
    ///
    /// # Errors
    ///
    /// Returns an error if request body description fails due to invalid type information or builder-specific errors.
    fn describe_request_body<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
    ) -> Result<Self::RequestBodySchemaBuilder<'a>, Self::Error>;

    /// Collect and describe the request body for the HTTP operation.
    ///
    /// # Paramaters
    /// - `description` - Optional description for the request body.
    /// - `deprecated` - Whether the request body is deprecated.
    /// - `required` - Whether the request body is required.
    /// - `describe` - A closure that describes the schema of the request body.
    ///
    /// # Errors
    ///
    /// Returns an error if request body description fails due to invalid type information or builder-specific errors.
    fn collect_request_body<'a, D>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(Self::RequestBodySchemaBuilder<'a>) -> Result<(), Self::Error>,
    {
        describe(HttpOperationBuilder::describe_request_body(
            self,
            description,
            deprecated,
            required,
        )?)
    }

    /// Describe a security requirement for the HTTP operation.
    ///
    /// # Errors
    ///
    /// Returns an error if security requirement description fails due to invalid type information or builder-specific errors.
    fn describe_security_requirement(
        &mut self,
    ) -> Result<Self::SecurityRequirementBuilder<'_>, Self::Error>;

    // TODO: Rename to end?

    /// Finalize the HTTP operation description.
    ///
    /// # Paramaters
    /// - `id` - The operation identifier.
    /// - `method` - The HTTP method (e.g., "GET", "POST").
    /// - `path` - The path for the operation (e.g., "/users/{id}").
    /// - `tags` - Optional iterator over tags for the operation.
    /// - `description` - Optional description for the operation.
    /// - `deprecated` - Whether the operation is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if operation description fails due to invalid type information or builder-specific errors.
    fn describe_operation<T>(
        self,
        id: HttpOperationId,
        method: &'static str,
        path: &'static str,
        tags: Option<T>,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::HttpResponseBuilder, Self::Error>
    where
        T: IntoIterator<Item = &'static str>;

    /// Collect and describe the HTTP operation.
    ///
    /// # Paramaters
    /// - `id` - The operation identifier.
    /// - `method` - The HTTP method (e.g., "GET", "POST").
    /// - `path` - The path for the operation (e.g., "/users/{id}").
    /// - `tags` - Optional iterator over tags for the operation.
    /// - `description` - Optional description for the operation.
    /// - `deprecated` - Whether the operation is deprecated.
    /// - `describe` - A closure that describes the responses for the operation.
    ///
    /// # Errors
    ///
    /// Returns an error if operation description fails due to invalid type information or builder-specific errors.
    #[allow(clippy::too_many_arguments)]
    fn collect_operation<T, D>(
        self,
        id: HttpOperationId,
        method: &'static str,
        path: &'static str,
        tags: Option<T>,
        description: Option<&'static str>,
        deprecated: bool,
        describe: D,
    ) -> Result<Self::Ok, Self::Error>
    where
        Self: Sized,
        T: IntoIterator<Item = &'static str>,
        D: FnOnce(Self::HttpResponseBuilder) -> Result<Self::Ok, Self::Error>,
    {
        describe(HttpOperationBuilder::describe_operation(
            self,
            id,
            method,
            path,
            tags,
            description,
            deprecated,
        )?)
    }
}

/// Trait for types that can describe themselves as HTTP operations.
pub trait HttpOperation {
    /// Describe the HTTP operation using the provided operation builder.
    ///
    /// # Paramaters
    /// - `operation_builder` - A builder that constructs the HTTP operation description.
    ///
    /// # Errors
    ///
    /// Returns an error if operation description fails due to invalid type information or builder-specific errors.
    fn describe<B>(operation_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpOperationBuilder;
}
