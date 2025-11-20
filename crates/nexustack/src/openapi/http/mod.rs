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
use crate::openapi::{
    HttpOperation, SpecificationVersion, schema::generator::SchemaCollection, spec,
};
use generator::{add_http_operation_to_paths, build_http_operation_with_collection};
use serde::Serialize;
use std::{borrow::Cow, cell::RefCell, collections::HashMap, rc::Rc};

pub(crate) mod content_type;
mod generator;
mod impls;
pub(crate) mod operation;
pub(crate) mod response;

/// Represents a variable for an HTTP server in the `OpenAPI` specification.
///
/// This struct is used to define variables that can be substituted into server URLs.
/// Variables can have default values, optional descriptions, and a list of allowed values.
///
/// # Examples
/// ```rust
/// use std::borrow::Cow;
/// use nexustack::openapi::HttpServerVariable;
///
/// let variable = HttpServerVariable::new("port".into(), "8080".into())
///     .with_description("The port number to connect to".into())
///     .with_enum_values(vec!["8080".into(), "9090".into()].into());
/// ```
pub struct HttpServerVariable {
    name: Cow<'static, str>,
    default: Cow<'static, str>,
    allowed_values: Option<Cow<'static, [Cow<'static, str>]>>,
    description: Option<Cow<'static, str>>,
}

impl HttpServerVariable {
    /// Creates a new `HttpServerVariable` with the specified name and default value.
    ///
    /// # Paramaters
    /// - `name` - The name of the variable.
    /// - `default` - The default value of the variable.
    ///
    /// # Examples
    /// ```rust
    /// use std::borrow::Cow;
    /// use nexustack::openapi::HttpServerVariable;
    ///
    /// let variable = HttpServerVariable::new("port".into(), "8080".into());
    /// ```
    #[must_use]
    pub const fn new(name: Cow<'static, str>, default: Cow<'static, str>) -> Self {
        Self {
            name,
            default,
            allowed_values: None,
            description: None,
        }
    }

    /// Returns the name of the variable.
    ///
    /// # Examples
    /// ```rust
    /// let name = variable.name();
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Returns the default value of the variable.
    ///
    /// # Examples
    /// ```rust
    /// let default = variable.default();
    /// ```
    #[must_use]
    pub fn default(&self) -> &str {
        self.default.as_ref()
    }

    /// Returns the allowed values for the variable, if any.
    ///
    /// # Examples
    /// ```rust
    /// let allowed_values = variable.enum_values();
    /// ```
    #[must_use]
    pub fn enum_values(&self) -> Option<&[Cow<'static, str>]> {
        self.allowed_values.as_deref()
    }

    /// Sets the allowed values for the variable.
    ///
    /// # Paramaters
    /// - `enum_values` - A list of allowed values for the variable.
    ///
    /// # Examples
    /// ```rust
    /// variable.with_enum_values(vec!["8080".into(), "9090".into()].into());
    /// ```
    pub fn with_enum_values(
        &mut self,
        enum_values: Cow<'static, [Cow<'static, str>]>,
    ) -> &mut Self {
        self.allowed_values = Some(enum_values);
        self
    }

    /// Returns the description of the variable, if any.
    ///
    /// # Examples
    /// ```rust
    /// let description = variable.description();
    /// ```
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Sets the description for the variable.
    ///
    /// # Paramaters
    /// - `description` - A description for the variable.
    ///
    /// # Examples
    /// ```rust
    /// variable.with_description("The port number to connect to".into());
    /// ```
    pub fn with_description(&mut self, description: Cow<'static, str>) -> &mut Self {
        self.description = Some(description);
        self
    }
}

/// Represents an HTTP server in the `OpenAPI` specification.
///
/// This struct is used to define an HTTP server, including its URL, optional description,
/// and optional variables that can be substituted into the URL.
///
/// # Examples
/// ```rust
/// use std::borrow::Cow;
/// use nexustack::openapi::{HttpServer, HttpServerVariable};
///
/// let server = HttpServer::new("https://api.example.com".into())
///     .with_description("The main API server".into())
///     .with_variables(vec![
///         HttpServerVariable::new("port".into(), "443".into())
///             .with_description("The port number to connect to".into()),
///     ]);
/// ```
pub struct HttpServer {
    url: Cow<'static, str>,
    description: Option<Cow<'static, str>>,
    variables: Option<Vec<HttpServerVariable>>,
}

impl HttpServer {
    /// Creates a new `HttpServer` with the specified URL.
    ///
    /// # Paramaters
    /// - `url` - The URL of the server.
    ///
    /// # Examples
    /// ```rust
    /// use std::borrow::Cow;
    /// use nexustack::openapi::HttpServer;
    ///
    /// let server = HttpServer::new("https://api.example.com".into());
    /// ```
    #[must_use]
    pub const fn new(url: Cow<'static, str>) -> Self {
        Self {
            url,
            description: None,
            variables: None,
        }
    }

    /// Returns the URL of the server.
    ///
    /// # Examples
    /// ```rust
    /// let url = server.url();
    /// ```
    #[must_use]
    pub fn url(&self) -> &str {
        self.url.as_ref()
    }

    /// Returns the description of the server, if any.
    ///
    /// # Examples
    /// ```rust
    /// let description = server.description();
    /// ```
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Sets the description for the server.
    ///
    /// # Paramaters
    /// - `description` - A description for the server.
    ///
    /// # Examples
    /// ```rust
    /// server.with_description("The main API server".into());
    /// ```
    pub fn with_description(&mut self, description: Cow<'static, str>) -> &mut Self {
        self.description = Some(description);
        self
    }

    /// Returns the variables for the server, if any.
    ///
    /// # Examples
    /// ```rust
    /// let variables = server.variables();
    /// ```
    #[must_use]
    pub fn variables(&self) -> Option<&[HttpServerVariable]> {
        self.variables.as_deref()
    }

    /// Sets the variables for the server.
    ///
    /// # Paramaters
    /// - `variables` - An iterator of `HttpServerVariable` instances.
    ///
    /// # Examples
    /// ```rust
    /// server.with_variables(vec![
    ///     HttpServerVariable::new("port".into(), "443".into())
    ///         .with_description("The port number to connect to".into()),
    /// ]);
    /// ```
    pub fn with_variables<V>(&mut self, variables: V) -> &mut Self
    where
        V: IntoIterator<Item = HttpServerVariable>,
    {
        self.variables = Some(variables.into_iter().collect());
        self
    }
}

impl From<HttpServer> for spec::ServerObject {
    fn from(server: HttpServer) -> Self {
        Self {
            url: server.url,
            description: server.description,
            variables: server.variables.map(|vars| {
                vars.into_iter()
                    .map(|var| {
                        (
                            var.name,
                            spec::ServerVariableObject {
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

/// Represents a tag in the `OpenAPI` specification.
///
/// This struct is used to define tags that can be associated with HTTP operations.
/// Tags provide a way to group operations in the `OpenAPI` documentation.
///
/// # Examples
/// ```rust
/// use std::borrow::Cow;
/// use nexustack::openapi::Tag;
///
/// let tag = Tag::new("user".into())
///     .with_description("Operations related to user management".into());
/// ```
pub struct Tag {
    name: Cow<'static, str>,
    description: Option<Cow<'static, str>>,
}

impl Tag {
    /// Creates a new `Tag` with the specified name.
    ///
    /// # Paramaters
    /// - `name` - The name of the tag.
    ///
    /// # Examples
    /// ```rust
    /// use std::borrow::Cow;
    /// use nexustack::openapi::Tag;
    ///
    /// let tag = Tag::new("user".into());
    /// ```
    #[must_use]
    pub const fn new(name: Cow<'static, str>) -> Self {
        Self {
            name,
            description: None,
        }
    }

    /// Returns the name of the tag.
    ///
    /// # Examples
    /// ```rust
    /// let name = tag.name();
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Returns the description of the tag, if any.
    ///
    /// # Examples
    /// ```rust
    /// let description = tag.description();
    /// ```
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Sets the description for the tag.
    ///
    /// # Paramaters
    /// - `description` - A description for the tag.
    ///
    /// # Examples
    /// ```rust
    /// tag.with_description("Operations related to user management".into());
    /// ```
    pub fn with_description(&mut self, description: Cow<'static, str>) -> &mut Self {
        self.description = Some(description);
        self
    }
}

impl From<Tag> for spec::TagObject {
    fn from(tag: Tag) -> Self {
        Self {
            name: tag.name,
            description: tag.description,
            external_docs: None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct HttpDocumentBuildError(&'static str);

/// Represents an `OpenAPI` document.
#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct HttpDocument(spec::OpenAPIObject);

impl HttpDocument {
    /// Returns the title of the `OpenAPI` document.
    #[must_use]
    pub fn title(&self) -> &str {
        self.0.info.title.as_ref()
    }
}

/// Builder for creating an `OpenAPI` document.
///
/// This struct provides methods to configure the `OpenAPI` document, including metadata, servers, tags, and operations.
/// It is designed to simplify the process of building a complete `OpenAPI` specification.
///
/// # Examples
///
/// ```rust
/// use nexustack::openapi::{HttpDocumentBuilder, HttpServer, Tag};
///
/// let builder = HttpDocumentBuilder::new("My API", "1.0")
///     .with_summary("A summary of the API")
///     .with_description("A detailed description of the API")
///     .with_terms_of_service("https://example.com/terms")
///     .with_contact(Some("Support"), Some("https://example.com/contact"), Some("support@example.com"))
///     .with_license_url("MIT", "https://opensource.org/licenses/MIT")
///     .with_servers(vec![HttpServer::new("https://api.example.com".into())])
///     .with_tags(vec![Tag::new("user".into())]);
/// ```
pub struct HttpDocumentBuilder {
    info: spec::InfoObject,
    paths: spec::PathsObject,
    schema_collection: Rc<RefCell<SchemaCollection>>,
    servers: Option<Vec<spec::ServerObject>>,
    tags: Option<Vec<spec::TagObject>>,
    operation_error: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl HttpDocumentBuilder {
    /// Creates a new `HttpDocumentBuilder` with the specified title and version.
    ///
    /// # Paramaters
    ///
    /// - `title` - The title of the API.
    /// - `version` - The version of the API.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let builder = HttpDocumentBuilder::new("My API", "1.0");
    /// ```
    #[must_use]
    pub fn new(title: &'static str, version: &'static str) -> Self {
        Self {
            info: spec::InfoObject {
                title: title.into(),
                version: version.into(),
                summary: None,
                description: None,
                terms_of_service: None,
                contact: None,
                license: None,
            },
            paths: spec::PathsObject(HashMap::new()),
            schema_collection: Rc::new(RefCell::new(SchemaCollection::new())),
            servers: None,
            tags: None,
            operation_error: None,
        }
    }

    /// Sets a summary for the `OpenAPI` document.
    ///
    /// # Parameters
    ///
    /// - `summary` - A short summary of the API.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let builder = HttpDocumentBuilder::new("My API", "1.0")
    ///     .with_summary("A summary of the API");
    /// ```
    #[must_use]
    pub fn with_summary(&mut self, summary: &'static str) -> &mut Self {
        self.info.summary = Some(summary.into());
        self
    }

    /// Sets a detailed description for the `OpenAPI` document.
    ///
    /// # Parameters
    ///
    /// - `description` - A detailed description of the API.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let builder = HttpDocumentBuilder::new("My API", "1.0")
    ///     .with_description("A detailed description of the API");
    /// ```
    #[must_use]
    pub fn with_description(&mut self, description: &'static str) -> &mut Self {
        self.info.description = Some(description.into());
        self
    }

    /// Sets the terms of service URL for the `OpenAPI` document.
    ///
    /// # Parameters
    ///
    /// - `terms_of_service` - The URL to the terms of service.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let builder = HttpDocumentBuilder::new("My API", "1.0")
    ///     .with_terms_of_service("https://example.com/terms");
    /// ```
    #[must_use]
    pub fn with_terms_of_service(&mut self, terms_of_service: &'static str) -> &mut Self {
        self.info.terms_of_service = Some(terms_of_service.into());
        self
    }

    /// Sets the contact information for the `OpenAPI` document.
    ///
    /// # Parameters
    ///
    /// - `name` - The name of the contact person or organization.
    /// - `url` - The URL to the contact information.
    /// - `email` - The email address for contacting support.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let builder = HttpDocumentBuilder::new("My API", "1.0")
    ///     .with_contact(Some("Support"), Some("https://example.com/contact"), Some("support@example.com"));
    /// ```
    #[must_use]
    pub fn with_contact(
        &mut self,
        name: Option<&'static str>,
        url: Option<&'static str>,
        email: Option<&'static str>,
    ) -> &mut Self {
        self.info.contact = Some(spec::ContactObject {
            name: name.map(Into::into),
            url: url.map(Into::into),
            email: email.map(Into::into),
        });
        self
    }

    /// Sets the license information for the `OpenAPI` document using a URL.
    ///
    /// # Parameters
    ///
    /// - `name` - The name of the license.
    /// - `url` - The URL to the license.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let builder = HttpDocumentBuilder::new("My API", "1.0")
    ///     .with_license_url("MIT", "https://opensource.org/licenses/MIT");
    /// ```
    #[must_use]
    pub fn with_license_url(&mut self, name: &'static str, url: &'static str) -> &mut Self {
        self.info.license = Some(spec::LicenseObject {
            name: name.into(),
            identifier: None,
            url: Some(url.into()),
        });
        self
    }

    /// Sets the license information for the `OpenAPI` document using an SPDX identifier.
    ///
    /// # Parameters
    ///
    /// - `name` - The name of the license.
    /// - `identifier` - The SPDX identifier for the license.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let builder = HttpDocumentBuilder::new("My API", "1.0")
    ///     .with_spdx_license("MIT", "MIT");
    /// ```
    #[must_use]
    pub fn with_spdx_license(&mut self, name: &'static str, identifier: &'static str) -> &mut Self {
        self.info.license = Some(spec::LicenseObject {
            name: name.into(),
            identifier: Some(identifier.into()),
            url: None,
        });
        self
    }

    /// Adds a list of servers to the `OpenAPI` document.
    ///
    /// # Parameters
    ///
    /// - `servers` - An iterator of `HttpServer` instances.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexustack::openapi::HttpServer;
    ///
    /// let builder = HttpDocumentBuilder::new("My API", "1.0")
    ///     .with_servers(vec![HttpServer::new("https://api.example.com".into())]);
    /// ```
    #[must_use]
    pub fn with_servers<S>(&mut self, servers: S) -> &mut Self
    where
        S: IntoIterator<Item = HttpServer>,
    {
        self.servers = Some(servers.into_iter().map(Into::into).collect());
        self
    }

    /// Adds a list of tags to the `OpenAPI` document.
    ///
    /// # Parameters
    ///
    /// - `tags` - An iterator of `Tag` instances.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexustack::openapi::Tag;
    ///
    /// let builder = HttpDocumentBuilder::new("My API", "1.0")
    ///     .with_tags(vec![Tag::new("user".into())]);
    /// ```
    #[must_use]
    pub fn with_tags<T>(&mut self, tags: T) -> &mut Self
    where
        T: IntoIterator<Item = Tag>,
    {
        self.tags = Some(tags.into_iter().map(Into::into).collect());
        self
    }

    /// Adds an HTTP operation to the `OpenAPI` document.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation cannot be added due to schema or path conflicts.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexustack::openapi::HttpDocumentBuilder;
    ///
    /// let mut builder = HttpDocumentBuilder::new("My API", "1.0");
    /// builder.add_operation::<MyOperation>().unwrap();
    /// ```
    pub fn add_operation<T>(&mut self) -> &mut Self
    where
        T: HttpOperation + 'static,
    {
        // TODO: This should not error but collect errors instead and raise them on build

        let keyed_operation_result = build_http_operation_with_collection::<T>(
            SpecificationVersion::OpenAPI3_1,
            self.schema_collection.clone(),
        );

        let keyed_operation = match keyed_operation_result {
            Ok(op) => op,
            Err(err) => {
                self.operation_error = Some(Box::new(err));
                return self;
            }
        };

        add_http_operation_to_paths(&mut self.paths, keyed_operation).unwrap_or_else(|err| {
            self.operation_error = Some(Box::new(err));
        });

        self
    }

    /// Finalizes the `OpenAPI` document and returns the resulting `HttpDocument`.
    ///
    /// # Errors
    ///
    /// This method returns an error if:
    /// - Any operation added to the document failed due to schema or path conflicts.
    /// - The schema collection cannot be unwrapped due to multiple references.
    ///
    /// # Panics
    ///
    /// This method panics if the schema collection cannot be unwrapped, which should not happen
    /// unless there is an internal logic error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexustack::openapi::HttpDocumentBuilder;
    ///
    /// let builder = HttpDocumentBuilder::new("My API", "1.0");
    /// let openapi = builder.build().unwrap();
    /// ```
    pub fn build(self) -> Result<HttpDocument, Box<dyn std::error::Error + Send + Sync + 'static>> {
        if let Some(operation_error) = self.operation_error {
            return Err(operation_error);
        }

        let schemas = Rc::try_unwrap(self.schema_collection)
            .map_err(|_| HttpDocumentBuildError("Should be the only Rc strong reference"))?
            .into_inner()
            .to_schemas_object();

        // TODO: security schemas

        Ok(HttpDocument(spec::OpenAPIObject {
            openapi: "3.1.0".into(),
            info: self.info,
            paths: self.paths,
            components: Some(spec::ComponentsObject {
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
        }))
    }
}
