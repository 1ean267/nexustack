/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{http::ConfigureRouter, openapi};
use std::{borrow::Cow, time::Duration};

#[derive(Debug, Copy, Clone)]
pub enum SessionManagerCookieSameSite {
    None,
    Lax,
    Strict,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SessionManagerCookie {
    name: &'static str,
    description: Option<&'static str>,
    domain: Option<Cow<'static, str>>,
    path: Option<Cow<'static, str>>,
    max_age: Option<Duration>,
    same_site: SessionManagerCookieSameSite,
    required: bool,
    http_only: bool,
    secure: bool,
    partitioned: bool,
}

pub trait SessionExtractor: Clone + Send + Sync + 'static {
    type Session<S: Send + Sync>: axum::extract::FromRequestParts<S, Rejection: std::error::Error + Send + Sync + 'static>;
}

pub trait SessionManager: ConfigureRouter + Clone + 'static {
    type Extractor: SessionExtractor;

    fn cookie(&self) -> Option<&SessionManagerCookie>;

    fn extractor(&self) -> Self::Extractor;
}

#[derive(Default, Copy, Clone)]
pub struct NoSessionManager;

impl ConfigureRouter for NoSessionManager {
    fn configure_router(&self, router: axum::Router) -> axum::Router {
        router
    }
}

impl SessionManager for NoSessionManager {
    type Extractor = Self;

    fn cookie(&self) -> Option<&SessionManagerCookie> {
        None
    }

    fn extractor(&self) -> Self::Extractor {
        *self
    }
}

impl SessionExtractor for NoSessionManager {
    type Session<S: Send + Sync> = ();
}

#[cfg(feature = "tower_sessions")]
pub struct TowerSessionsSessionManager<S, C>
where
    S: tower_sessions::SessionStore,
    C: tower_sessions::service::CookieController,
{
    cookie: SessionManagerCookie,
    layer: tower_sessions::SessionManagerLayer<S, C>,
}

#[cfg(feature = "tower_sessions")]
impl<S, C> SessionManager for TowerSessionsSessionManager<S, C>
where
    S: tower_sessions::SessionStore + Clone,
    C: tower_sessions::service::CookieController + Clone + Sync,
{
    type Extractor = TowerSessionExtractor;

    fn cookie(&self) -> Option<&SessionManagerCookie> {
        Some(&self.cookie)
    }

    fn configure_router<L>(&mut self, router: axum::Router<L>) -> axum::Router<L>
    where
        L: Clone + Send + Sync + 'static,
    {
        router.layer(self.layer.clone())
    }

    fn extractor(&self) -> Self::Extractor {
        TowerSessionExtractor
    }
}

#[cfg(feature = "tower_sessions")]
#[derive(Default, Clone, Copy)]
pub struct TowerSessionExtractor;

#[cfg(feature = "tower_sessions")]
impl SessionExtractor for TowerSessionExtractor {
    type Session<S: Send + Sync> = tower_sessions::Session;
}

pub struct WithSessionCookie<'c, Endpoint>(
    std::marker::PhantomData<&'c ()>,
    std::marker::PhantomData<Endpoint>,
);

impl<'c, Endpoint> openapi::http::HttpOperation for WithSessionCookie<'c, Endpoint>
where
    Endpoint: openapi::http::HttpOperation<Context = ()>,
{
    type Context = &'c SessionManagerCookie;

    fn describe<B>(operation_builder: B, context: Self::Context) -> Result<B::Ok, B::Error>
    where
        B: openapi::http::HttpOperationBuilder,
    {
        let operation_builder = WithSessionCookieOperationBuilder {
            wrapped: operation_builder,
            cookie: context,
        };

        <Endpoint as openapi::http::HttpOperation>::describe(operation_builder, ())
    }
}

struct WithSessionCookieOperationBuilder<'c, OperationBuilder> {
    wrapped: OperationBuilder,
    cookie: &'c SessionManagerCookie,
}

impl<'c, OperationBuilder> openapi::http::HttpOperationBuilder
    for WithSessionCookieOperationBuilder<'c, OperationBuilder>
where
    OperationBuilder: openapi::http::HttpOperationBuilder,
{
    type Ok = OperationBuilder::Ok;
    type Error = OperationBuilder::Error;

    type SchemaBuilderError = OperationBuilder::SchemaBuilderError;

    type ParameterSchemaBuilder<'a>
        = OperationBuilder::ParameterSchemaBuilder<'a>
    where
        Self: 'a;

    type RequestBodySchemaBuilder<'a>
        = OperationBuilder::RequestBodySchemaBuilder<'a>
    where
        Self: 'a;

    type SecurityRequirementBuilder<'a>
        = OperationBuilder::SecurityRequirementBuilder<'a>
    where
        Self: 'a;

    type HttpResponseBuilder =
        WithSessionCookieResponseBuilder<'c, OperationBuilder::HttpResponseBuilder>;

    fn describe_query_parameter<'a>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
    ) -> Result<Self::ParameterSchemaBuilder<'a>, Self::Error> {
        self.wrapped
            .describe_query_parameter(name, description, deprecated, required)
    }

    fn describe_header_parameter<'a>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
    ) -> Result<Self::ParameterSchemaBuilder<'a>, Self::Error> {
        self.wrapped
            .describe_header_parameter(name, description, deprecated, required)
    }

    fn describe_path_parameter<'a>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::ParameterSchemaBuilder<'a>, Self::Error> {
        self.wrapped
            .describe_path_parameter(name, description, deprecated)
    }

    fn describe_cookie_parameter<'a>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
    ) -> Result<Self::ParameterSchemaBuilder<'a>, Self::Error> {
        self.wrapped
            .describe_cookie_parameter(name, description, deprecated, required)
    }

    fn describe_request_body<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
    ) -> Result<Self::RequestBodySchemaBuilder<'a>, Self::Error> {
        self.wrapped
            .describe_request_body(description, deprecated, required)
    }

    fn describe_security_requirement(
        &mut self,
    ) -> Result<Self::SecurityRequirementBuilder<'_>, Self::Error> {
        self.wrapped.describe_security_requirement()
    }

    fn describe_response<T>(
        mut self,
        id: openapi::http::HttpOperationId,
        method: &'static str,
        path: &'static str,
        tags: Option<T>,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::HttpResponseBuilder, Self::Error>
    where
        T: IntoIterator<Item = &'static str>,
    {
        self.wrapped.collect_cookie_parameter(
            self.cookie.name,
            self.cookie.description,
            false,
            Some(self.cookie.required),
            <String as openapi::Schema>::describe,
        )?;

        Ok(WithSessionCookieResponseBuilder {
            wrapped: self.wrapped.describe_response(
                id,
                method,
                path,
                tags,
                description,
                deprecated,
            )?,
            cookie: self.cookie,
        })
    }
}

struct WithSessionCookieResponseBuilder<'c, ResponseBuilder> {
    wrapped: ResponseBuilder,
    cookie: &'c SessionManagerCookie,
}

impl<'c, ResponseBuilder> openapi::http::HttpResponseBuilder
    for WithSessionCookieResponseBuilder<'c, ResponseBuilder>
where
    ResponseBuilder: openapi::http::HttpResponseBuilder,
{
    type Ok = ResponseBuilder::Ok;
    type Error = ResponseBuilder::Error;

    type StatusResponseBuilder<'a>
        = WithSessionCookieStatusResponseBuilder<'c, ResponseBuilder::StatusResponseBuilder<'a>>
    where
        Self: 'a;

    fn describe_status_response<'a>(
        &'a mut self,
        status_code: u16,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::StatusResponseBuilder<'a>, Self::Error> {
        Ok(WithSessionCookieStatusResponseBuilder {
            wrapped: self
                .wrapped
                .describe_status_response(status_code, description, deprecated)?,
            cookie: self.cookie,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.wrapped.end()
    }
}

struct WithSessionCookieStatusResponseBuilder<'c, ResponseBuilder> {
    wrapped: ResponseBuilder,
    cookie: &'c SessionManagerCookie,
}

impl<'c, ResponseBuilder> openapi::http::HttpStatusResponseBuilder
    for WithSessionCookieStatusResponseBuilder<'c, ResponseBuilder>
where
    ResponseBuilder: openapi::http::HttpStatusResponseBuilder,
{
    type ResponseCookieSchemaBuilder<'a>
        = ResponseBuilder::ResponseCookieSchemaBuilder<'a>
    where
        Self: 'a;

    type ResponseHeaderSchemaBuilder<'a>
        = ResponseBuilder::ResponseHeaderSchemaBuilder<'a>
    where
        Self: 'a;

    fn describe_response_cookie<'a>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        domain: Option<Cow<'static, str>>,
        path: Option<Cow<'static, str>>,
        max_age: Option<Duration>,
        same_site: openapi::http::CookieSameSite,
        deprecated: bool,
        required: bool,
        http_only: bool,
        secure: bool,
        partitioned: bool,
    ) -> Result<Self::ResponseCookieSchemaBuilder<'a>, Self::Error> {
        self.wrapped.describe_response_cookie(
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
        )
    }

    fn describe_response_header<'a>(
        &'a mut self,
        name: http::header::HeaderName,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
    ) -> Result<Self::ResponseHeaderSchemaBuilder<'a>, Self::Error> {
        self.wrapped
            .describe_response_header(name, description, deprecated, required)
    }
}

impl<'c, ContentTypeBuilder> openapi::http::HttpContentTypeBuilder
    for WithSessionCookieStatusResponseBuilder<'c, ContentTypeBuilder>
where
    ContentTypeBuilder: openapi::http::HttpStatusResponseBuilder,
{
    type Ok = ContentTypeBuilder::Ok;
    type Error = ContentTypeBuilder::Error;

    type SchemaBuilderError = ContentTypeBuilder::SchemaBuilderError;

    type SchemaBuilder<'a>
        = ContentTypeBuilder::SchemaBuilder<'a>
    where
        Self: 'a;

    fn describe_content_type<'a>(
        &'a mut self,
        content_type: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::SchemaBuilder<'a>, Self::Error> {
        self.wrapped
            .describe_content_type(content_type, description, deprecated)
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.wrapped.collect_response_cookie(
            self.cookie.name,
            self.cookie.description,
            self.cookie.domain.clone(),
            self.cookie.path.clone(),
            self.cookie.max_age,
            match self.cookie.same_site {
                SessionManagerCookieSameSite::None => openapi::http::CookieSameSite::None,
                SessionManagerCookieSameSite::Lax => openapi::http::CookieSameSite::Lax,
                SessionManagerCookieSameSite::Strict => openapi::http::CookieSameSite::Strict,
            },
            false,
            self.cookie.required,
            self.cookie.http_only,
            self.cookie.secure,
            self.cookie.partitioned,
            <String as openapi::Schema>::describe,
        )?;

        self.wrapped.end()
    }
}
