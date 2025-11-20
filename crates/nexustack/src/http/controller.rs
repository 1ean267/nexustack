/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{http::HttpEndpoint, inject::FromInjector};

#[cfg(feature = "openapi")]
use crate::openapi;

/// A trait for building HTTP endpoints.
///
/// This trait provides methods to add HTTP endpoints to a builder, either as visible or hidden endpoints.
pub trait HttpEndpointsBuilder {
    /// Adds an HTTP endpoint to the builder.
    ///
    /// This endpoint will be included in the `OpenAPI` documentation if the `openapi` feature is enabled.
    ///
    /// # Type Parameters
    /// - `E` - The endpoint type implementing `HttpEndpoint`, `FromInjector`, and `openapi::HttpOperation`.
    #[cfg(feature = "openapi")]
    fn add_endpoint<E>(&mut self)
    where
        E: HttpEndpoint + FromInjector + openapi::HttpOperation + Send + Sync + 'static;

    /// Adds an HTTP endpoint to the builder.
    ///
    /// This endpoint will be included in the `OpenAPI` documentation if the `openapi` feature is enabled.
    ///
    /// # Type Parameters
    /// - `E` - The endpoint type implementing `HttpEndpoint`, `FromInjector`, and `openapi::HttpOperation`.
    #[cfg(not(feature = "openapi"))]
    fn add_endpoint<E>(&mut self)
    where
        E: HttpEndpoint + FromInjector + Send + Sync + 'static;

    /// Adds an HTTP endpoint to the builder as a hidden endpoint.
    ///
    /// Hidden endpoints are not included in the `OpenAPI` documentation.
    ///
    /// # Type Parameters
    /// - `E` - The endpoint type implementing `HttpEndpoint` and `FromInjector`.
    fn add_hidden_endpoint<E>(&mut self)
    where
        E: HttpEndpoint + FromInjector + Send + Sync + 'static;
}

/// A trait for defining HTTP controllers.
///
/// HTTP controllers are responsible for building and registering multiple endpoints using a provided builder.
pub trait HttpController {
    /// Builds and registers HTTP endpoints using the provided builder.
    ///
    /// # Type Parameters
    /// - `B` - The builder type implementing `HttpEndpointsBuilder`.
    fn build_endpoints<B>(builder: B)
    where
        B: HttpEndpointsBuilder;
}
