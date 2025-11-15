/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

//! HTTP `OpenAPI` builder traits and types for describing HTTP operations, responses, content types, and security requirements.
//!
//! This module provides traits and types for building `OpenAPI` documentation for HTTP APIs, including operation identifiers, response and content type builders, and security requirements.
//!
//! # Overview
//!
//! - [`HttpOperationId`]: Uniquely identifies an HTTP operation with a name and callsite.
//! - [`HttpResponseBuilder`], [`HttpContentTypeBuilder`]: Traits for describing HTTP responses and their content types.
//! - [`HttpOperationBuilder`]: Trait for describing HTTP operations, parameters, request bodies, and security requirements.
//! - [`HttpSecurityRequirementBuilder`]: Trait for describing security requirements for HTTP operations.
//! - [`HttpResponse`], [`HttpOperation`]: Traits for types that can describe themselves as HTTP responses or operations.
//!
//! All builder traits follow a similar pattern: they provide methods to describe parts of an HTTP API, returning sub-builders for further description, and an `end` method to finalize the description.
//!
//! # Errors
//!
//! All builder methods that return `Result` may fail due to invalid type information, unsupported types, or builder-specific errors encountered during description.
//!
//! # See Also
//!
//! - [`crate::openapi::schema_builder`]: For schema building traits and types.
//! - [`crate::openapi::Error`]: The error trait used throughout the `OpenAPI` builder traits.

use serde::Serialize;

use crate::{
    Callsite,
    openapi::{
        Error, IntoSchemaBuilder, Schema,
        json::{
            SchemaCollection, add_http_operation_to_paths, build_http_operation_with_collection,
            specification,
        },
    },
};
use std::{borrow::Cow, cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

pub struct HttpServerVariable {
    name: Cow<'static, str>,
    default: Cow<'static, str>,
    allowed_values: Option<Cow<'static, [Cow<'static, str>]>>,
    description: Option<Cow<'static, str>>,
}

impl HttpServerVariable {
    pub fn new(name: Cow<'static, str>, default: Cow<'static, str>) -> Self {
        Self {
            name,
            default,
            allowed_values: None,
            description: None,
        }
    }

    pub fn name(&self) -> &Cow<'static, str> {
        &self.name
    }

    pub fn default(&self) -> &Cow<'static, str> {
        &self.default
    }

    // TODO: Very complex type
    pub fn enum_values(&self) -> Option<&Cow<'static, [Cow<'static, str>]>> {
        self.allowed_values.as_ref()
    }

    pub fn with_enum_values(
        &mut self,
        enum_values: Cow<'static, [Cow<'static, str>]>,
    ) -> &mut Self {
        self.allowed_values = Some(enum_values);
        self
    }

    pub fn description(&self) -> Option<&Cow<'static, str>> {
        self.description.as_ref()
    }

    pub fn with_description(&mut self, description: Cow<'static, str>) -> &mut Self {
        self.description = Some(description);
        self
    }
}

// TODO: Rename
pub struct HttpServer {
    url: Cow<'static, str>,
    description: Option<Cow<'static, str>>,
    variables: Option<Vec<HttpServerVariable>>,
}

impl HttpServer {
    pub fn new(url: Cow<'static, str>) -> Self {
        Self {
            url,
            description: None,
            variables: None,
        }
    }

    fn url(&self) -> &Cow<'static, str> {
        &self.url
    }

    fn description(&self) -> Option<&Cow<'static, str>> {
        self.description.as_ref()
    }

    pub fn with_description(&mut self, description: Cow<'static, str>) -> &mut Self {
        self.description = Some(description);
        self
    }

    fn variables(&self) -> Option<&[HttpServerVariable]> {
        self.variables.as_deref()
    }

    pub fn with_variables<V>(&mut self, variables: V) -> &mut Self
    where
        V: IntoIterator<Item = HttpServerVariable>,
    {
        self.variables = Some(variables.into_iter().collect());
        self
    }
}

impl From<HttpServer> for specification::ServerObject {
    fn from(server: HttpServer) -> Self {
        Self {
            url: server.url,
            description: server.description,
            variables: server.variables.map(|vars| {
                vars.into_iter()
                    .map(|var| {
                        (
                            var.name,
                            specification::ServerVariableObject {
                                r#enum: var.allowed_values.map(|vals| vals.to_vec()),
                                default: var.default,
                                description: var.description,
                            },
                        )
                    })
                    .collect()
            }),
        }
    }
}

pub struct Tag {
    name: Cow<'static, str>,
    description: Option<Cow<'static, str>>,
}

impl Tag {
    pub fn new(name: Cow<'static, str>) -> Self {
        Self {
            name,
            description: None,
        }
    }

    pub fn name(&self) -> &Cow<'static, str> {
        &self.name
    }

    pub fn description(&self) -> Option<&Cow<'static, str>> {
        self.description.as_ref()
    }

    pub fn with_description(&mut self, description: Cow<'static, str>) -> &mut Self {
        self.description = Some(description);
        self
    }
}

impl From<Tag> for specification::TagObject {
    fn from(tag: Tag) -> Self {
        Self {
            name: tag.name,
            description: tag.description,
            external_docs: None,
        }
    }
}

pub struct HttpDocumentBuilder {
    info: specification::InfoObject,
    paths: specification::PathsObject,
    schema_collection: Rc<RefCell<SchemaCollection>>,
    servers: Option<Vec<specification::ServerObject>>,
    tags: Option<Vec<specification::TagObject>>,
}

impl HttpDocumentBuilder {
    pub fn new(title: &'static str, version: &'static str) -> Self {
        Self {
            info: specification::InfoObject {
                title: title.into(),
                version: version.into(),
                summary: None,
                description: None,
                terms_of_service: None,
                contact: None,
                license: None,
            },
            paths: specification::PathsObject(HashMap::new()),
            schema_collection: Rc::new(RefCell::new(SchemaCollection::new())),
            servers: None,
            tags: None,
        }
    }

    pub fn with_summary(mut self, summary: &'static str) -> Self {
        self.info.summary = Some(summary.into());
        self
    }

    pub fn with_description(mut self, description: &'static str) -> Self {
        self.info.description = Some(description.into());
        self
    }

    pub fn with_terms_of_service(mut self, terms_of_service: &'static str) -> Self {
        self.info.terms_of_service = Some(terms_of_service.into());
        self
    }

    pub fn with_contact(
        mut self,
        name: Option<&'static str>,
        url: Option<&'static str>,
        email: Option<&'static str>,
    ) -> Self {
        self.info.contact = Some(specification::ContactObject {
            name: name.map(Into::into),
            url: url.map(Into::into),
            email: email.map(Into::into),
        });
        self
    }

    pub fn with_license_url(mut self, name: &'static str, url: &'static str) -> Self {
        self.info.license = Some(specification::LicenseObject {
            name: name.into(),
            identifier: None,
            url: Some(url.into()),
        });
        self
    }

    pub fn with_spdx_license(mut self, name: &'static str, identifier: &'static str) -> Self {
        self.info.license = Some(specification::LicenseObject {
            name: name.into(),
            identifier: Some(identifier.into()),
            url: None,
        });
        self
    }

    pub fn with_servers<S>(mut self, servers: S) -> Self
    where
        S: IntoIterator<Item = HttpServer>,
    {
        self.servers = Some(servers.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_tags<T>(mut self, tags: T) -> Self
    where
        T: IntoIterator<Item = Tag>,
    {
        self.tags = Some(tags.into_iter().map(Into::into).collect());
        self
    }

    // TODO: Error type
    pub fn add_operation<T>(&mut self) -> Result<&mut Self, Box<dyn std::error::Error>>
    where
        T: HttpOperation + 'static,
    {
        let keyed_operation = build_http_operation_with_collection::<T>(
            specification::Specification::OpenAPI3_1,
            self.schema_collection.clone(),
        )?;

        add_http_operation_to_paths(&mut self.paths, keyed_operation)?;
        Ok(self)
    }

    pub fn build(self) -> specification::OpenAPIObject {
        let schemas = Rc::try_unwrap(self.schema_collection)
            .map_err(|_| "Should be the only Rc strong reference")
            .unwrap() // TODO: No unwrap
            .into_inner()
            .to_schemas_object();

        // TODO: security schemas

        specification::OpenAPIObject {
            openapi: "3.1.0".into(),
            info: self.info,
            paths: self.paths,
            components: Some(specification::ComponentsObject {
                schemas: if schemas.is_empty() {
                    None
                } else {
                    Some(schemas)
                },
                ..Default::default()
            }),
            servers: self.servers,
            tags: self.tags,
            json_schema_dialect: Some("https://spec.openapis.org/oas/3.1/dialect/base".into()),
            external_docs: None,
            security: None,
            webhooks: None,
        }
    }
}

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
    /// # Arguments
    /// * `content_type` - The MIME type of the content (e.g., "application/json").
    /// * `description` - Optional description for the content type.
    /// * `deprecated` - Whether the content type is deprecated.
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

pub trait HttpContentType {
    fn describe<T: Schema, B>(content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpContentTypeBuilder;
}

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
    /// # Arguments
    /// * `status_code` - The HTTP status code (e.g., 200, 404).
    /// * `description` - Description for the response.
    /// * `deprecated` - Whether the response is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if response description fails due to invalid type information or builder-specific errors.
    fn describe_response<'a>(
        &'a mut self,
        status_code: u16,
        description: &'static str,
        deprecated: bool,
    ) -> Result<Self::ContentTypeBuilder<'a>, Self::Error>;

    fn describe_empty_response(
        &mut self,
        status_code: u16,
        description: &'static str,
        deprecated: bool,
    ) -> Result<(), Self::Error> {
        let content_type_builder =
            HttpResponseBuilder::describe_response(self, status_code, description, deprecated)?;

        content_type_builder.end()
    }

    fn collect_response<'a, D>(
        &'a mut self,
        status_code: u16,
        description: &'static str,
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
    /// # Arguments
    /// * `response_builder` - A builder that constructs the HTTP response description.
    ///
    /// # Errors
    ///
    /// Returns an error if response description fails due to invalid type information or builder-specific errors.
    fn describe<B>(response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpResponseBuilder;
}

struct WrappedHttpResponseBuilder<B> {
    inner: B,
}

impl<B> HttpResponseBuilder for &mut WrappedHttpResponseBuilder<B>
where
    B: HttpResponseBuilder,
{
    type Ok = ();
    type Error = B::Error;

    type ContentTypeBuilder<'a>
        = B::ContentTypeBuilder<'a>
    where
        Self: 'a;

    fn describe_response<'a>(
        &'a mut self,
        status_code: u16,
        description: &'static str,
        deprecated: bool,
    ) -> Result<Self::ContentTypeBuilder<'a>, Self::Error> {
        self.inner
            .describe_response(status_code, description, deprecated)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<T, E> HttpResponse for Result<T, E>
where
    T: HttpResponse,
    E: HttpResponse,
{
    fn describe<B>(response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpResponseBuilder,
    {
        let mut wrapped = WrappedHttpResponseBuilder {
            inner: response_builder,
        };

        <T as HttpResponse>::describe(&mut wrapped)?;
        <E as HttpResponse>::describe(&mut wrapped)?;

        wrapped.inner.end()
    }
}

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
    /// # Arguments
    /// * `name` - The name of the HTTP operation.
    /// * `callsite` - The callsite information.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::HttpOperationId;
    /// use nexustack::init_callsite;
    ///
    /// init_callsite!(MyTypeCallsite);
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
    /// # Arguments
    /// * `name` - The name of the security scheme.
    /// * `scopes` - Optional iterator over required scopes for the security scheme.
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
    /// # Arguments
    /// * `name` - The name of the query parameter.
    /// * `description` - Optional description for the parameter.
    /// * `deprecated` - Whether the parameter is deprecated.
    /// * `required` - Whether the parameter is required.
    ///
    /// # Errors
    ///
    /// Returns an error if parameter description fails due to invalid type information or builder-specific errors.
    fn describe_query_parameter<'a>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        required: bool,
    ) -> Result<Self::ParameterSchemaBuilder<'a>, Self::Error>;

    // TODO: Style, example

    /// Describe a header parameter for the HTTP operation.
    ///
    /// # Arguments
    /// * `name` - The name of the header parameter.
    /// * `description` - Optional description for the parameter.
    /// * `deprecated` - Whether the parameter is deprecated.
    /// * `required` - Whether the parameter is required.
    ///
    /// # Errors
    ///
    /// Returns an error if parameter description fails due to invalid type information or builder-specific errors.
    fn describe_header_parameter<'a>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        required: bool,
    ) -> Result<Self::ParameterSchemaBuilder<'a>, Self::Error>;

    fn collect_header_parameter<'a, D, E: Iterator<Item: Serialize + 'static>>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        required: bool,
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
    /// # Arguments
    /// * `name` - The name of the path parameter.
    /// * `description` - Optional description for the parameter.
    /// * `deprecated` - Whether the parameter is deprecated.
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
    /// # Arguments
    /// * `name` - The name of the cookie parameter.
    /// * `description` - Optional description for the parameter.
    /// * `deprecated` - Whether the parameter is deprecated.
    /// * `required` - Whether the parameter is required.
    ///
    /// # Errors
    ///
    /// Returns an error if parameter description fails due to invalid type information or builder-specific errors.
    fn describe_cookie_parameter<'a>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        required: bool,
    ) -> Result<Self::ParameterSchemaBuilder<'a>, Self::Error>;

    fn collect_cookie_parameter<'a, D, E: Iterator<Item: Serialize + 'static>>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        required: bool,
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
    /// # Arguments
    /// * `description` - Optional description for the request body.
    /// * `deprecated` - Whether the request body is deprecated.
    /// * `required` - Whether the request body is required.
    ///
    /// # Errors
    ///
    /// Returns an error if request body description fails due to invalid type information or builder-specific errors.
    fn describe_request_body<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
        required: bool,
    ) -> Result<Self::RequestBodySchemaBuilder<'a>, Self::Error>;

    fn collect_request_body<'a, D>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        required: bool,
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
    /// # Arguments
    /// * `id` - The operation identifier.
    /// * `method` - The HTTP method (e.g., "GET", "POST").
    /// * `path` - The path for the operation (e.g., "/users/{id}").
    /// * `tags` - Optional iterator over tags for the operation.
    /// * `description` - Optional description for the operation.
    /// * `deprecated` - Whether the operation is deprecated.
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
    /// # Arguments
    /// * `operation_builder` - A builder that constructs the HTTP operation description.
    ///
    /// # Errors
    ///
    /// Returns an error if operation description fails due to invalid type information or builder-specific errors.
    fn describe<B>(operation_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpOperationBuilder;
}
