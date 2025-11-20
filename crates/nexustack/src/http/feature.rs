/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    ApplicationPart, ApplicationPartBuilder, Here, InHead, InTail, Index as ChainIndex, Node,
    http::{
        ConfigureRouter, HttpBindAddress, HttpEndpoint, HttpEndpointContext,
        controller::HttpController,
        decoding,
        endpoint::{ConfiguredHttpEndpoint, HttpEndpointContextContainer},
        session,
        user::{self, UserExtractorConfigureRouter},
    },
    inject::{ConstructionResult, FromInjector, ServiceProvider},
};
use std::{net::SocketAddr, path::PathBuf};
use tokio_util::sync::CancellationToken;

#[cfg(feature = "openapi")]
use crate::{
    http::swagger::{OpenApiConfiguration, OpenApiConfigurationBuilder},
    inject::ConstructionError,
    openapi,
};
#[cfg(feature = "openapi")]
use std::borrow::Cow;

/// Errors that can occur in the HTTP application part.
#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    /// Failed to bind to a TCP address.
    #[error("Failed to bind to TCP address {addrs:?}: {source}")]
    TcpBindError {
        /// The addresses attempted.
        addrs: Vec<SocketAddr>,
        /// The underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// Failed to bind to a Unix socket.
    #[error("Failed to bind to Unix socket {path:?}: {source}")]
    UnixBindError {
        /// The path attempted.
        path: PathBuf,
        /// The underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// Error while serving HTTP.
    #[error("Error while serving HTTP: {0}")]
    ServeError(#[source] std::io::Error),
}

/// The `Http` trait represents the HTTP feature in the application.
///
/// It provides methods for configuring HTTP routers, adding endpoints, and controllers.
pub trait Http<Index> {
    type User;
    type Session;
    type Context: HttpEndpointContext<Session = Self::Session, User = Self::User>;

    /// Configure the router using the provided function.
    ///
    /// # Paramaters
    /// - `configure` - A function that takes the current router and returns a new router.
    #[allow(clippy::missing_panics_doc)]
    fn configure_router<F>(&mut self, configure: F) -> &mut Self
    where
        F: ConfigureRouter + 'static;

    /// Add a hidden HTTP endpoint to the router.
    ///
    /// # Type Parameters
    /// - `E` - The endpoint type implementing `HttpEndpoint` and `FromInjector`.
    #[allow(clippy::missing_panics_doc)]
    fn add_hidden_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint<Self::Context> + FromInjector + Send + Sync + 'static;

    /// Add an HTTP endpoint to the router.
    ///
    /// # Type Parameters
    /// - `E` - The endpoint type implementing `HttpEndpoint` and `FromInjector`.
    #[allow(clippy::missing_panics_doc)]
    #[cfg(feature = "openapi")]
    fn add_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint<Self::Context>
            + FromInjector
            + openapi::http::HttpOperation<Context = ()>
            + Send
            + Sync
            + 'static;

    /// Add an HTTP endpoint to the router.
    ///
    /// # Type Parameters
    /// - `E` - The endpoint type implementing `HttpEndpoint` and `FromInjector`.
    #[allow(clippy::missing_panics_doc)]
    #[cfg(not(feature = "openapi"))]
    fn add_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint<Self::Context> + FromInjector + Send + Sync + 'static;

    /// Add an HTTP controller to the router.
    ///
    /// # Type Parameters
    /// - `C` - The controller type implementing `HttpController`.
    ///
    /// This method allows adding a controller that defines multiple endpoints to the HTTP router.
    /// Controllers are responsible for building their endpoints using the provided builder.
    fn add_controller<C>(&mut self) -> &mut Self
    where
        C: HttpController<Self::Context>;

    /// Set the client IP source configuration.
    ///
    /// Parameters
    /// - `client_ip_source` - The source from which to extract the client IP address.
    #[cfg(feature = "axum-client-ip")]
    fn with_client_ip_source(
        &mut self,
        client_ip_source: axum_client_ip::ClientIpSource,
    ) -> &mut Self;
}

impl<Head, Tail, HeadIndex> Http<InHead<HeadIndex>> for Node<Head, Tail>
where
    HeadIndex: ChainIndex,
    Head: Http<HeadIndex>,
{
    type User = Head::User;
    type Session = Head::Session;
    type Context = Head::Context;

    fn configure_router<F>(&mut self, configure: F) -> &mut Self
    where
        F: ConfigureRouter + 'static,
    {
        self.head.configure_router(configure);
        self
    }

    fn add_hidden_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint<Self::Context> + FromInjector + Send + Sync + 'static,
    {
        self.head.add_hidden_endpoint::<E>();
        self
    }

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
        self.head.add_endpoint::<E>();
        self
    }

    #[cfg(not(feature = "openapi"))]
    fn add_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint<Self::Context> + FromInjector + Send + Sync + 'static,
    {
        self.head.add_endpoint::<E>();
        self
    }

    fn add_controller<C>(&mut self) -> &mut Self
    where
        C: HttpController<Self::Context>,
    {
        self.head.add_controller::<C>();
        self
    }

    #[cfg(feature = "axum-client-ip")]
    fn with_client_ip_source(
        &mut self,
        client_ip_source: axum_client_ip::ClientIpSource,
    ) -> &mut Self {
        self.head.with_client_ip_source(client_ip_source);
        self
    }
}

impl<Head, Tail, TailIndex> Http<InTail<TailIndex>> for Node<Head, Tail>
where
    Tail: Http<TailIndex>,
{
    type User = Tail::User;
    type Session = Tail::Session;
    type Context = Tail::Context;

    fn configure_router<F>(&mut self, configure: F) -> &mut Self
    where
        F: ConfigureRouter + 'static,
    {
        self.tail.configure_router(configure);
        self
    }

    fn add_hidden_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint<Self::Context> + FromInjector + Send + Sync + 'static,
    {
        self.tail.add_hidden_endpoint::<E>();
        self
    }

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
        self.tail.add_endpoint::<E>();
        self
    }

    #[cfg(not(feature = "openapi"))]
    fn add_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint<Self::Context> + FromInjector + Send + Sync + 'static,
    {
        self.tail.add_endpoint::<E>();
        self
    }

    fn add_controller<C>(&mut self) -> &mut Self
    where
        C: HttpController<Self::Context>,
    {
        self.tail.add_controller::<C>();
        self
    }

    #[cfg(feature = "axum-client-ip")]
    fn with_client_ip_source(
        &mut self,
        client_ip_source: axum_client_ip::ClientIpSource,
    ) -> &mut Self {
        self.tail.with_client_ip_source(client_ip_source);
        self
    }
}

/// Builder for the HTTP application part.
pub struct HttpApplicationPartBuilder<DefaultDecoderFactory, UserExtractor, SessionManager> {
    /// The address to bind the HTTP server to.
    bind_address: HttpBindAddress,
    /// The router builder.
    router_builders: Vec<Box<dyn ConfigureRouter>>,
    /// The `OpenAPI` configuration builder.
    #[cfg(feature = "openapi")]
    openapi_configuration_builder: OpenApiConfigurationBuilder,
    /// The client IP source configuration.
    #[cfg(feature = "axum-client-ip")]
    client_ip_source: axum_client_ip::ClientIpSource,
    default_decoder_factory: DefaultDecoderFactory,
    user_extractor: UserExtractor,
    session_manager: SessionManager,
}

impl
    HttpApplicationPartBuilder<
        fn() -> decoding::DefaultDecoder,
        user::NoUserExtractor,
        session::NoSessionManager,
    >
{
    /// Create a new `HttpApplicationPartBuilder`.
    ///
    /// # Paramaters
    /// - `bind_address` - The address to bind the HTTP server to.
    /// - `openapi_document_builder` - The `OpenAPI` document builder.
    pub(crate) fn new(
        bind_address: HttpBindAddress,
        #[cfg(feature = "openapi")] openapi_document_builder: Option<
            openapi::http::HttpDocumentBuilder,
        >,
    ) -> Self {
        Self {
            bind_address,
            router_builders: Vec::new(),
            #[cfg(feature = "openapi")]
            openapi_configuration_builder: OpenApiConfigurationBuilder::new(
                openapi_document_builder,
            ),
            #[cfg(feature = "axum-client-ip")]
            client_ip_source: axum_client_ip::ClientIpSource::ConnectInfo,
            default_decoder_factory: || decoding::DefaultDecoder,
            user_extractor: user::NoUserExtractor,
            session_manager: session::NoSessionManager,
        }
    }
}

impl<DefaultDecoderFactory, UserExtractor, SessionManager>
    HttpApplicationPartBuilder<DefaultDecoderFactory, UserExtractor, SessionManager>
{
    #[cfg(feature = "openapi")]
    pub(crate) fn with_open_api(
        mut self,
        openapi_document_builder: openapi::http::HttpDocumentBuilder,
    ) -> Self {
        self.openapi_configuration_builder
            .with_openapi_document_builder(Some(openapi_document_builder));
        self
    }

    #[cfg(feature = "openapi")]
    pub(crate) fn with_open_api_at_path(
        mut self,
        path: Cow<'static, str>,
        openapi_document_builder: openapi::http::HttpDocumentBuilder,
    ) -> Self {
        self.openapi_configuration_builder
            .with_openapi_document_builder(Some(openapi_document_builder));
        self.openapi_configuration_builder.with_openapi_path(path);
        self
    }

    pub(crate) fn with_default_decoder<D, F>(
        self,
        default_decoder_factory: F,
    ) -> HttpApplicationPartBuilder<F, UserExtractor, SessionManager>
    where
        D: decoding::Decoder + Send + Sync + 'static,
        F: Fn() -> D,
    {
        HttpApplicationPartBuilder {
            bind_address: self.bind_address,
            router_builders: self.router_builders,
            #[cfg(feature = "openapi")]
            openapi_configuration_builder: self.openapi_configuration_builder,
            #[cfg(feature = "axum-client-ip")]
            client_ip_source: self.client_ip_source,
            default_decoder_factory,
            user_extractor: self.user_extractor,
            session_manager: self.session_manager,
        }
    }

    pub(crate) fn with_session_manager<S>(
        self,
        session_manager: S,
    ) -> HttpApplicationPartBuilder<DefaultDecoderFactory, UserExtractor, S>
    where
        S: session::SessionManager,
    {
        HttpApplicationPartBuilder {
            bind_address: self.bind_address,
            router_builders: self.router_builders,
            #[cfg(feature = "openapi")]
            openapi_configuration_builder: self.openapi_configuration_builder,
            #[cfg(feature = "axum-client-ip")]
            client_ip_source: self.client_ip_source,
            default_decoder_factory: self.default_decoder_factory,
            user_extractor: self.user_extractor,
            session_manager,
        }
    }

    pub(crate) fn with_user_extractor<E>(
        self,
        user_extractor: E,
    ) -> HttpApplicationPartBuilder<DefaultDecoderFactory, E, SessionManager>
    where
        E: user::UserExtractor,
    {
        HttpApplicationPartBuilder {
            bind_address: self.bind_address,
            router_builders: self.router_builders,
            #[cfg(feature = "openapi")]
            openapi_configuration_builder: self.openapi_configuration_builder,
            #[cfg(feature = "axum-client-ip")]
            client_ip_source: self.client_ip_source,
            default_decoder_factory: self.default_decoder_factory,
            user_extractor: user_extractor,
            session_manager: self.session_manager,
        }
    }
}

impl<DefaultDecoder, DefaultDecoderFactory, UserExtractor, SessionManager> Http<Here>
    for HttpApplicationPartBuilder<DefaultDecoderFactory, UserExtractor, SessionManager>
where
    DefaultDecoder: decoding::Decoder + Send + Sync + 'static,
    DefaultDecoderFactory: Fn() -> DefaultDecoder + Clone + Send + Sync + 'static,
    UserExtractor: user::UserExtractor,
    SessionManager: session::SessionManager,
{
    type User = <Self::Context as HttpEndpointContext>::User;
    type Session = <Self::Context as HttpEndpointContext>::Session;
    type Context = HttpEndpointContextContainer<
        DefaultDecoder,
        UserExtractor::User,
        SessionManager::Extractor,
    >;

    /// Configure the router using the provided function.
    ///
    /// # Paramaters
    /// - `configure` - A function that takes the current router and returns a new router.
    #[allow(clippy::missing_panics_doc)]
    fn configure_router<F>(&mut self, configure: F) -> &mut Self
    where
        F: ConfigureRouter + 'static,
    {
        self.router_builders.push(Box::new(configure));
        self
    }

    /// Add an HTTP endpoint to the router.
    ///
    /// # Type Parameters
    /// - `E` - The endpoint type implementing `HttpEndpoint` and `FromInjector`.
    #[allow(clippy::missing_panics_doc)]
    fn add_hidden_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint<Self::Context> + FromInjector + Send + Sync + 'static,
    {
        self.configure_router(
            ConfiguredHttpEndpoint::<E, _, UserExtractor, SessionManager>::new(
                self.default_decoder_factory.clone(),
            ),
        )
    }

    /// Add an HTTP endpoint to the router.
    ///
    /// # Type Parameters
    /// - `E` - The endpoint type implementing `HttpEndpoint` and `FromInjector`.
    #[allow(clippy::missing_panics_doc)]
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
        self.openapi_configuration_builder
            .configure_openapi_document(|openapi_document_builder| {
                if let Some(cookie) = self.session_manager.cookie() {
                    openapi_document_builder
                        .add_operation_with_context::<session::WithSessionCookie<'_, E>>(cookie);
                } else {
                    openapi_document_builder.add_operation::<E>();
                }
            });

        Http::<Here>::add_hidden_endpoint::<E>(self)
    }

    /// Add an HTTP endpoint to the router.
    ///
    /// # Type Parameters
    /// - `E` - The endpoint type implementing `HttpEndpoint` and `FromInjector`.
    #[allow(clippy::missing_panics_doc)]
    #[cfg(not(feature = "openapi"))]
    fn add_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint<Self::Context> + FromInjector + Send + Sync + 'static,
    {
        Http::<Here>::add_hidden_endpoint::<E>(self)
    }

    fn add_controller<C>(&mut self) -> &mut Self
    where
        C: HttpController<Self::Context>,
    {
        C::build_endpoints(self);
        self
    }

    #[cfg(feature = "axum-client-ip")]
    fn with_client_ip_source(
        &mut self,
        client_ip_source: axum_client_ip::ClientIpSource,
    ) -> &mut Self {
        self.client_ip_source = client_ip_source;
        self
    }
}

impl<DefaultDecoderFactory, UserExtractor, SessionManager> ApplicationPartBuilder
    for HttpApplicationPartBuilder<DefaultDecoderFactory, UserExtractor, SessionManager>
where
    UserExtractor: user::UserExtractor,
    SessionManager: session::SessionManager,
{
    type ApplicationPart = HttpApplicationPart<UserExtractor, SessionManager>;

    fn build(self, service_provider: ServiceProvider) -> ConstructionResult<Self::ApplicationPart> {
        #[cfg(feature = "openapi")]
        let openapi_configuration = self
            .openapi_configuration_builder
            .build()
            .map_err(|err| ConstructionError::Custom(err.into()))?;

        Ok(HttpApplicationPart::new(
            service_provider,
            self.bind_address,
            self.router_builders,
            #[cfg(feature = "openapi")]
            openapi_configuration,
            #[cfg(feature = "axum-client-ip")]
            self.client_ip_source,
            self.user_extractor,
            self.session_manager,
        ))
    }
}

/// The HTTP application part.
pub struct HttpApplicationPart<UserExtractor, SessionManager> {
    /// The service provider for dependency injection.
    service_provider: ServiceProvider,
    /// The address to bind the HTTP server to.
    bind_address: HttpBindAddress,
    /// The router builder.
    router_builders: Vec<Box<dyn ConfigureRouter>>,
    #[cfg(feature = "openapi")]
    // The built `OpenAPI` configuration.
    open_api_configuration: OpenApiConfiguration,
    /// The client IP source configuration.
    #[cfg(feature = "axum-client-ip")]
    client_ip_source: axum_client_ip::ClientIpSource,
    user_extractor: UserExtractorConfigureRouter<UserExtractor>,
    session_manager: SessionManager,
}

impl<UserExtractor, SessionManager> HttpApplicationPart<UserExtractor, SessionManager> {
    /// Create a new `HttpApplicationPart`.
    ///
    /// # Paramaters
    /// - `service_provider` - The service provider for dependency injection.
    /// - `bind_address` - The address to bind the HTTP server to.
    /// - `router_builder` - The router builder.
    /// - `open_api_configuration` - The built `OpenAPI` configuration.
    /// - `client_ip_source` - The client IP source configuration.
    pub(crate) fn new(
        service_provider: ServiceProvider,
        bind_address: HttpBindAddress,
        router_builders: Vec<Box<dyn ConfigureRouter>>,
        #[cfg(feature = "openapi")] open_api_configuration: OpenApiConfiguration,
        #[cfg(feature = "axum-client-ip")] client_ip_source: axum_client_ip::ClientIpSource,
        user_extractor: UserExtractor,
        session_manager: SessionManager,
    ) -> Self {
        Self {
            service_provider,
            bind_address,
            router_builders,
            #[cfg(feature = "openapi")]
            open_api_configuration,
            #[cfg(feature = "axum-client-ip")]
            client_ip_source,
            user_extractor: UserExtractorConfigureRouter(user_extractor),
            session_manager,
        }
    }
}

impl<UserExtractor, SessionManager> ApplicationPart
    for HttpApplicationPart<UserExtractor, SessionManager>
where
    UserExtractor: user::UserExtractor,
    SessionManager: session::SessionManager,
{
    type Error = HttpError;

    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<(), Self::Error> {
        // TODO: configure openapi to include the user extraction layer rejections

        let mut router = axum::Router::new();

        router = self.service_provider.configure_router(router);

        for router_builder in self.router_builders.iter() {
            router = router_builder.configure_router(router);
        }

        #[cfg(feature = "openapi")]
        {
            router = self.open_api_configuration.configure_router(router);
        }
        router = self.user_extractor.configure_router(router);
        router = self.session_manager.configure_router(router);

        match &self.bind_address {
            HttpBindAddress::Unix(path) => {
                let listener = tokio::net::UnixListener::bind(path).map_err(|err| {
                    HttpError::UnixBindError {
                        path: path.to_path_buf(),
                        source: err,
                    }
                })?;
                self.run_with_unix_listener(router, listener, cancellation_token)
                    .await?;

                Ok(())
            }
            HttpBindAddress::Tcp(addrs) => {
                let listener = tokio::net::TcpListener::bind(addrs.as_ref())
                    .await
                    .map_err(|err| HttpError::TcpBindError {
                        addrs: addrs.to_vec(),
                        source: err,
                    })?;
                self.run_with_tcp_listener(router, listener, cancellation_token)
                    .await?;

                Ok(())
            }
        }
    }
}

impl<UserExtractor, SessionManager> HttpApplicationPart<UserExtractor, SessionManager>
where
    UserExtractor: user::UserExtractor,
    SessionManager: session::SessionManager,
{
    /// Run the HTTP server with the given TCP listener.
    ///
    /// # Paramaters
    /// - `app` - The Axum router to serve.
    /// - `listener` - The TCP listener to accept connections from.
    /// - `cancellation_token` - A token to signal graceful shutdown.
    ///
    /// # Errors
    /// Returns an `HttpError::ServeError` if the server encounters an error while serving HTTP.
    async fn run_with_tcp_listener(
        &self,
        app: axum::Router,
        listener: tokio::net::TcpListener,
        cancellation_token: CancellationToken,
    ) -> Result<(), <Self as ApplicationPart>::Error> {
        let make_service = app;
        #[cfg(feature = "axum-client-ip")]
        let make_service = make_service.layer(self.client_ip_source.clone().into_extension());
        let make_service = make_service.into_make_service_with_connect_info::<SocketAddr>();

        axum::serve(listener, make_service)
            .with_graceful_shutdown(cancellation_token.cancelled_owned())
            .await
            .map_err(HttpError::ServeError)
    }

    /// Run the HTTP server with the given Unix listener.
    ///
    /// # Paramaters
    /// - `app` - The Axum router to serve.
    /// - `listener` - The Unix listener to accept connections from.
    /// - `cancellation_token` - A token to signal graceful shutdown.
    async fn run_with_unix_listener(
        &self,
        app: axum::Router,
        listener: tokio::net::UnixListener,
        cancellation_token: CancellationToken,
    ) -> Result<(), <Self as ApplicationPart>::Error> {
        let make_service = app;
        #[cfg(feature = "axum-client-ip")]
        let make_service = make_service.layer(self.client_ip_source.clone().into_extension());

        axum::serve(listener, make_service)
            .with_graceful_shutdown(cancellation_token.cancelled_owned())
            .await
            .map_err(HttpError::ServeError)
    }
}
