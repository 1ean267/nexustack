/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use std::marker::PhantomData;

use crate::http::{HttpMethod, response::IntoResponseWithContext, session};
use axum::extract::FromRequest;
use nonempty_collections::IntoNonEmptyIterator;

/// Trait representing an HTTP endpoint.
///
/// Implementors define the request and response types, the HTTP method, route, and handler logic.
///
/// # Type Parameters
/// - `C` - The type of endpoint context for the http feature.
pub trait HttpEndpoint<C: HttpEndpointContext> {
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
    /// - `context` - The endpoint conext object for this request.
    fn handle(
        &mut self,
        request: Self::Request,
        context: C,
    ) -> impl Future<Output = Self::Response> + Send;
}

mod seal {
    /// A sealed trait to prevent external implementations of `HttpEndpointContext`.
    pub trait Seal {}

    impl<SessionExtractor> Seal for super::HttpEndpointContextContainer<SessionExtractor> {}
}

pub trait HttpEndpointContext: seal::Seal + Send + Sync + 'static {
    type Session: axum::extract::FromRequestParts<(), Rejection: std::error::Error + Send + Sync + 'static>;
    type SessionExtractor: session::SessionExtractor<Session<()> = Self::Session>; // TODO: Generic State Arg
}

pub struct HttpEndpointContextContainer<SessionExtractor> {
    _session_extractor: PhantomData<SessionExtractor>,
}

impl<SessionExtractor> HttpEndpointContextContainer<SessionExtractor> {
    pub(crate) fn new() -> Self {
        Self {
            _session_extractor: PhantomData,
        }
    }
}

impl<SessionExtractor> HttpEndpointContext for HttpEndpointContextContainer<SessionExtractor>
where
    SessionExtractor: session::SessionExtractor,
{
    type Session = SessionExtractor::Session<()>; // TODO: Generic State Arg
    type SessionExtractor = SessionExtractor;
}
