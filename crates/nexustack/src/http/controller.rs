/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    Here,
    http::{
        Http, HttpApplicationPartBuilder, HttpEndpoint, HttpEndpointContext, decoding, session,
        user,
    },
    inject::FromInjector,
};

#[cfg(feature = "openapi")]
use crate::openapi;

/// A trait for building HTTP endpoints.
///
/// This trait provides methods to add HTTP endpoints to a builder, either as visible or hidden endpoints.
pub trait HttpEndpointsBuilder {
    type Context: HttpEndpointContext;

    /// Adds an HTTP endpoint to the builder.
    ///
    /// This endpoint will be included in the `OpenAPI` documentation if the `openapi` feature is enabled.
    ///
    /// # Type Parameters
    /// - `E` - The endpoint type implementing `HttpEndpoint`, `FromInjector`, and `openapi::HttpOperation`.
    #[cfg(feature = "openapi")]
    fn add_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint<Self::Context>
            + FromInjector
            + openapi::http::HttpOperation<Context = ()>
            + Send
            + Sync
            + 'static;

    /// Adds an HTTP endpoint to the builder.
    ///
    /// This endpoint will be included in the `OpenAPI` documentation if the `openapi` feature is enabled.
    ///
    /// # Type Parameters
    /// - `E` - The endpoint type implementing `HttpEndpoint`, `FromInjector`, and `openapi::HttpOperation`.
    #[cfg(not(feature = "openapi"))]
    fn add_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint<Self::Context> + FromInjector + Send + Sync + 'static;

    /// Adds an HTTP endpoint to the builder as a hidden endpoint.
    ///
    /// Hidden endpoints are not included in the `OpenAPI` documentation.
    ///
    /// # Type Parameters
    /// - `E` - The endpoint type implementing `HttpEndpoint` and `FromInjector`.
    fn add_hidden_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint<Self::Context> + FromInjector + Send + Sync + 'static;
}

impl<DefaultDecoder, DefaultDecoderFactory, UserExtractor, SessionManager> HttpEndpointsBuilder
    for HttpApplicationPartBuilder<DefaultDecoderFactory, UserExtractor, SessionManager>
where
    DefaultDecoder: decoding::Decoder + Send + Sync + 'static,
    DefaultDecoderFactory: Fn() -> DefaultDecoder + Clone + Send + Sync + 'static,
    UserExtractor: user::UserExtractor,
    SessionManager: session::SessionManager,
{
    type Context = <Self as Http<Here>>::Context;

    #[cfg(feature = "openapi")]
    fn add_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint<Self::Context>
            + FromInjector
            + openapi::http::HttpOperation<Context = ()>
            + Send
            + Sync
            + 'static,
    {
        Http::<Here>::add_endpoint::<E>(self)
    }

    #[cfg(not(feature = "openapi"))]
    fn add_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint<Self::Context> + FromInjector + Send + Sync + 'static,
    {
        Http::<Here>::add_endpoint::<E>(self)
    }

    fn add_hidden_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint<Self::Context> + FromInjector + Send + Sync + 'static,
    {
        Http::<Here>::add_hidden_endpoint::<E>(self)
    }
}

/// A trait for defining HTTP controllers.
///
/// HTTP controllers are responsible for building and registering multiple endpoints using a provided builder.
///
/// # Type Parameters
/// - `C` - The type of endpoint context for the http feature.
pub trait HttpController<C: HttpEndpointContext> {
    /// Builds and registers HTTP endpoints using the provided builder.
    ///
    /// # Type Parameters
    /// - `B` - The builder type implementing `HttpEndpointsBuilder`.
    fn build_endpoints<B>(builder: &mut B)
    where
        B: HttpEndpointsBuilder<Context = C>;
}
