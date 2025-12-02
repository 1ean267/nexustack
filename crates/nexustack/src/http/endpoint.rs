/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::http::{HttpMethod, response::IntoResponseWithContext};
use axum::extract::FromRequest;
use nonempty_collections::IntoNonEmptyIterator;

/// Trait representing an HTTP endpoint.
///
/// Implementors define the request and response types, the HTTP method, route, and handler logic.
pub trait HttpEndpoint {
    /// The request type, which must implement [`FromRequest`].
    type Request: FromRequest<()> + Send;
    /// The response type, which must implement [`IntoResponseWithContext`].
    type Response: IntoResponseWithContext<()>;

    /// The type representing the routes for this endpoint.
    ///
    /// This type must implement [`IntoNonEmptyIterator`], ensuring that at least one route is defined.
    /// Each route is represented as a static string slice (`&'static str`).
    type Routes: IntoNonEmptyIterator<Item = &'static str>;

    /// Returns the HTTP method for this endpoint.
    fn method() -> HttpMethod;

    /// Returns the route paths for this endpoint.
    fn routes() -> Self::Routes;

    /// Handles the endpoint logic.
    ///
    /// # Paramaters
    /// - `request` - The request object for this endpoint.
    fn handle(&mut self, request: Self::Request) -> impl Future<Output = Self::Response> + Send;
}
