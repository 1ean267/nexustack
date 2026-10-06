/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use std::marker::PhantomData;

use crate::{
    http::{
        ConfigureRouter, HttpMethod, decoding, decoding::Decoder,
        response::IntoResponseWithContext, session, user,
    },
    inject::{FromInjector, ServiceScope},
};
use nonempty_collections::IntoNonEmptyIterator;

/// Trait representing an HTTP endpoint.
///
/// Implementors define the request and response types, the HTTP method, route, and handler logic.
///
/// # Type Parameters
/// - `C` - The type of endpoint context for the http feature.
pub trait HttpEndpoint<C: HttpEndpointContext> {
    /// The request type, which must implement [`FromRequest`].
    type Request: axum::extract::FromRequest<()> + Send;
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

    impl<DefaultDecoder, User, SessionExtractor> Seal
        for super::HttpEndpointContextContainer<DefaultDecoder, User, SessionExtractor>
    {
    }
}

pub trait HttpEndpointContext: seal::Seal + Send + Sync + 'static {
    type DefaultDecoder: Decoder;
    type User: Send + Sync;
    type Session: axum::extract::FromRequestParts<(), Rejection: std::error::Error + Send + Sync + 'static>;
    type SessionExtractor: session::SessionExtractor<Session<()> = Self::Session>; // TODO: Generic State Arg

    fn default_decoder(&self) -> &Self::DefaultDecoder;
}

pub struct HttpEndpointContextContainer<DefaultDecoder, User, SessionExtractor> {
    default_decoder: DefaultDecoder,
    _user: PhantomData<User>,
    _session_extractor: PhantomData<SessionExtractor>,
}

impl<DefaultDecoder, User, SessionExtractor>
    HttpEndpointContextContainer<DefaultDecoder, User, SessionExtractor>
{
    pub(crate) fn new(default_decoder: DefaultDecoder) -> Self {
        Self {
            default_decoder,
            _user: PhantomData,
            _session_extractor: PhantomData,
        }
    }
}

impl<DefaultDecoder, User, SessionExtractor> HttpEndpointContext
    for HttpEndpointContextContainer<DefaultDecoder, User, SessionExtractor>
where
    DefaultDecoder: Decoder + Send + Sync + 'static,
    User: Send + Sync + 'static,
    SessionExtractor: session::SessionExtractor,
{
    type DefaultDecoder = DefaultDecoder;
    type User = User;
    type Session = SessionExtractor::Session<()>; // TODO: Generic State Arg
    type SessionExtractor = SessionExtractor;

    fn default_decoder(&self) -> &Self::DefaultDecoder {
        &self.default_decoder
    }
}

pub struct ConfiguredHttpEndpoint<Endpoint, DefaultDecoderFactory, UserExtractor, SessionManager> {
    _endpoint: PhantomData<Endpoint>,
    default_decoder_factory: DefaultDecoderFactory,
    _user_extractor: PhantomData<UserExtractor>,
    _session_manager: PhantomData<SessionManager>,
}

impl<Endpoint, DefaultDecoderFactory, UserExtractor, SessionManager>
    ConfiguredHttpEndpoint<Endpoint, DefaultDecoderFactory, UserExtractor, SessionManager>
{
    pub(crate) fn new(default_decoder_factory: DefaultDecoderFactory) -> Self {
        Self {
            _endpoint: PhantomData,
            default_decoder_factory,
            _user_extractor: PhantomData,
            _session_manager: PhantomData,
        }
    }
}

impl<Endpoint, DefaultDecoderFactory, UserExtractor, SessionManager> Clone
    for ConfiguredHttpEndpoint<Endpoint, DefaultDecoderFactory, UserExtractor, SessionManager>
where
    DefaultDecoderFactory: Clone,
{
    fn clone(&self) -> Self {
        Self {
            _endpoint: PhantomData,
            default_decoder_factory: self.default_decoder_factory.clone(),
            _user_extractor: PhantomData,
            _session_manager: PhantomData,
        }
    }
}

impl<Endpoint, DefaultDecoder, DefaultDecoderFactory, UserExtractor, SessionManager> ConfigureRouter
    for ConfiguredHttpEndpoint<Endpoint, DefaultDecoderFactory, UserExtractor, SessionManager>
where
    Endpoint: HttpEndpoint<
            HttpEndpointContextContainer<
                DefaultDecoder,
                UserExtractor::User,
                SessionManager::Extractor,
            >,
        > + FromInjector
        + Send
        + Sync
        + 'static,
    DefaultDecoder: decoding::Decoder + Send + Sync + 'static,
    DefaultDecoderFactory: Fn() -> DefaultDecoder + Clone + Send + Sync + 'static,
    UserExtractor: user::UserExtractor,
    SessionManager: session::SessionManager,
    Self: Clone,
{
    fn configure_router(&self, mut router: axum::Router) -> axum::Router {
        for route in Endpoint::routes() {
            router = router.route(
                route,
                Endpoint::method().route({
                    let default_decoder_factory = self.default_decoder_factory.clone();

                    async move |request: axum::extract::Request| {
                        let service_scope = request
                            .extensions()
                            .get::<ServiceScope>()
                            .expect("ServiceScope is registered in a middleware for each request");

                        let mut endpoint =
                            match service_scope.service_provider().construct::<Endpoint>() {
                                Ok(endpoint) => endpoint,
                                Err(err) => {
                                    tracing::error!("Failed to construct endpoint {err}");
                                    return axum::response::IntoResponse::into_response((
                                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                        format!("Failed to construct endpoint: {err}"),
                                    ));
                                }
                            };

                        let (response_state, request) = match <(
                            <Endpoint::Response as IntoResponseWithContext<()>>::Context,
                            Endpoint::Request,
                        ) as axum::extract::FromRequest<()>>::from_request(
                            request, &()
                        )
                        .await
                        {
                            Ok((response_state, request)) => (response_state, request),
                            Err(err) => {
                                return axum::response::IntoResponse::into_response(err);
                            }
                        };

                        endpoint
                            .handle(
                                request,
                                HttpEndpointContextContainer::<
                                    _,
                                    UserExtractor::User,
                                    SessionManager::Extractor,
                                >::new((default_decoder_factory)()),
                            )
                            .await
                            .into_response(response_state)
                    }
                }),
            );
        }

        router
    }
}
