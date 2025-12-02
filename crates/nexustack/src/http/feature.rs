/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    ApplicationPart, ApplicationPartBuilder, Here, InHead, InTail, Index, Node,
    http::{
        HttpBindAddress, HttpEndpoint,
        controller::{HttpController, HttpEndpointsBuilder},
        response::IntoResponseWithContext,
    },
    inject::{ConstructionResult, FromInjector, ServiceProvider, ServiceScope},
};
use axum::{
    Router,
    extract::{FromRequest, Request, State},
    middleware::Next,
    response::Response,
};
#[cfg(feature = "openapi")]
use std::borrow::Cow;
use std::{net::SocketAddr, path::PathBuf};
use tokio_util::sync::CancellationToken;

#[cfg(feature = "openapi")]
use crate::{
    http::swagger::SwaggerRouter,
    inject::ConstructionError,
    openapi::{self, HttpDocument, HttpDocumentBuilder},
};

#[cfg(feature = "yaml")]
use crate::http::yaml::Yaml;

/// Errors that can occur in the HTTP application part.
#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    /// The HTTP application is already running.
    #[error("HTTP application is already running")]
    ApplicationAlreadyRunning,
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
    /// Configure the router using the provided function.
    ///
    /// # Paramaters
    /// - `f` - A function that takes the current router and returns a new router.
    #[allow(clippy::missing_panics_doc)]
    fn configure_router(&mut self, f: impl FnOnce(Router) -> Router) -> &mut Self;

    /// Add a hidden HTTP endpoint to the router.
    ///
    /// # Type Parameters
    /// - `E` - The endpoint type implementing `HttpEndpoint` and `FromInjector`.
    #[allow(clippy::missing_panics_doc)]
    fn add_hidden_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint + FromInjector + Send + Sync + 'static;

    /// Add an HTTP endpoint to the router.
    ///
    /// # Type Parameters
    /// - `E` - The endpoint type implementing `HttpEndpoint` and `FromInjector`.
    #[allow(clippy::missing_panics_doc)]
    #[cfg(feature = "openapi")]
    fn add_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint + FromInjector + openapi::HttpOperation + Send + Sync + 'static;

    /// Add an HTTP endpoint to the router.
    ///
    /// # Type Parameters
    /// - `E` - The endpoint type implementing `HttpEndpoint` and `FromInjector`.
    #[allow(clippy::missing_panics_doc)]
    #[cfg(not(feature = "openapi"))]
    fn add_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint + FromInjector + Send + Sync + 'static;

    /// Add an HTTP controller to the router.
    ///
    /// # Type Parameters
    /// - `C` - The controller type implementing `HttpController`.
    ///
    /// This method allows adding a controller that defines multiple endpoints to the HTTP router.
    /// Controllers are responsible for building their endpoints using the provided builder.
    fn add_controller<C>(&mut self) -> &mut Self
    where
        C: HttpController;

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
    HeadIndex: Index,
    Head: Http<HeadIndex>,
{
    fn configure_router(&mut self, f: impl FnOnce(Router) -> Router) -> &mut Self {
        self.head.configure_router(f);
        self
    }

    fn add_hidden_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint + FromInjector + Send + Sync + 'static,
    {
        self.head.add_hidden_endpoint::<E>();
        self
    }

    #[cfg(feature = "openapi")]
    fn add_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint + FromInjector + openapi::HttpOperation + Send + Sync + 'static,
    {
        self.head.add_endpoint::<E>();
        self
    }

    #[cfg(not(feature = "openapi"))]
    fn add_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint + FromInjector + Send + Sync + 'static,
    {
        self.head.add_endpoint::<E>();
        self
    }

    fn add_controller<C>(&mut self) -> &mut Self
    where
        C: HttpController,
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
    TailIndex: Index,
    Tail: Http<TailIndex>,
{
    fn configure_router(&mut self, f: impl FnOnce(Router) -> Router) -> &mut Self {
        self.tail.configure_router(f);
        self
    }

    fn add_hidden_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint + FromInjector + Send + Sync + 'static,
    {
        self.tail.add_hidden_endpoint::<E>();
        self
    }

    #[cfg(feature = "openapi")]
    fn add_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint + FromInjector + openapi::HttpOperation + Send + Sync + 'static,
    {
        self.tail.add_endpoint::<E>();
        self
    }

    #[cfg(not(feature = "openapi"))]
    fn add_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint + FromInjector + Send + Sync + 'static,
    {
        self.tail.add_endpoint::<E>();
        self
    }

    fn add_controller<C>(&mut self) -> &mut Self
    where
        C: HttpController,
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
pub struct HttpApplicationPartBuilder {
    /// The address to bind the HTTP server to.
    bind_address: HttpBindAddress,
    /// The `OpenAPI` document builder.
    #[cfg(feature = "openapi")]
    openapi_document_builder: Option<openapi::HttpDocumentBuilder>,
    /// The path to serve the `OpenAPI` document and Swagger UI at.
    #[cfg(feature = "openapi")]
    openapi_path: Cow<'static, str>,
    /// The router instance (wrapped in Option for internal mutability).
    router: Option<axum::Router>,
    /// The client IP source configuration.
    #[cfg(feature = "axum-client-ip")]
    client_ip_source: axum_client_ip::ClientIpSource,
}

struct HttpEndpointsBuilderImpl<'a> {
    builder: &'a mut HttpApplicationPartBuilder,
}

impl HttpEndpointsBuilder for HttpEndpointsBuilderImpl<'_> {
    #[cfg(feature = "openapi")]
    fn add_endpoint<E>(&mut self)
    where
        E: HttpEndpoint + FromInjector + openapi::HttpOperation + Send + Sync + 'static,
    {
        self.builder.add_endpoint::<E>();
    }

    #[cfg(not(feature = "openapi"))]
    fn add_endpoint<E>(&mut self)
    where
        E: HttpEndpoint + FromInjector + Send + Sync + 'static,
    {
        self.builder.add_endpoint::<E>();
    }

    fn add_hidden_endpoint<E>(&mut self)
    where
        E: HttpEndpoint + FromInjector + Send + Sync + 'static,
    {
        self.builder.add_hidden_endpoint::<E>();
    }
}

impl HttpApplicationPartBuilder {
    /// Create a new `HttpApplicationPartBuilder`.
    ///
    /// # Paramaters
    /// - `bind_address` - The address to bind the HTTP server to.
    /// - `openapi_document_builder` - The `OpenAPI` document builder.
    pub(crate) fn new(
        bind_address: HttpBindAddress,
        #[cfg(feature = "openapi")] openapi_document_builder: Option<openapi::HttpDocumentBuilder>,
    ) -> Self {
        Self {
            bind_address,
            #[cfg(feature = "openapi")]
            openapi_document_builder,
            #[cfg(feature = "openapi")]
            openapi_path: Cow::Borrowed("/api"),
            router: Some(Router::new()),
            #[cfg(feature = "axum-client-ip")]
            client_ip_source: axum_client_ip::ClientIpSource::ConnectInfo,
        }
    }

    #[cfg(feature = "openapi")]
    pub(crate) fn with_open_api(&mut self, openapi_document_builder: openapi::HttpDocumentBuilder) {
        self.openapi_document_builder = Some(openapi_document_builder);
    }

    #[cfg(feature = "openapi")]
    pub(crate) fn with_open_api_at_path(
        &mut self,
        path: Cow<'static, str>,
        openapi_document_builder: openapi::HttpDocumentBuilder,
    ) {
        self.with_open_api(openapi_document_builder);
        self.openapi_path = path;
    }
}

impl Http<Here> for HttpApplicationPartBuilder {
    /// Configure the router using the provided function.
    ///
    /// # Paramaters
    /// - `f` - A function that takes the current router and returns a new router.
    #[allow(clippy::missing_panics_doc)]
    fn configure_router(&mut self, f: impl FnOnce(Router) -> Router) -> &mut Self {
        let router = self
            .router
            .take()
            .expect("router not taken as we have a mut ref to self so no one else can take it");
        self.router = Some(f(router));
        self
    }

    /// Add an HTTP endpoint to the router.
    ///
    /// # Type Parameters
    /// - `E` - The endpoint type implementing `HttpEndpoint` and `FromInjector`.
    #[allow(clippy::missing_panics_doc)]
    fn add_hidden_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint + FromInjector + Send + Sync + 'static,
    {
        let mut router = self
            .router
            .take()
            .expect("router not taken as we have a mut ref to self so no one else can take it");

        for route in E::routes() {
            router = router.route(
                route,
                E::method().route(async |request: Request| {
                    let service_scope = request
                        .extensions()
                        .get::<ServiceScope>()
                        .expect("ServiceScope is registered in a middleware for each request");

                    let mut endpoint = match service_scope.service_provider().construct::<E>() {
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
                        <E::Response as IntoResponseWithContext<()>>::Context,
                        E::Request,
                    ) as FromRequest<()>>::from_request(
                        request, &()
                    )
                    .await
                    {
                        Ok((response_state, request)) => (response_state, request),
                        Err(err) => {
                            return axum::response::IntoResponse::into_response(err);
                        }
                    };

                    endpoint.handle(request).await.into_response(response_state)
                }),
            );
        }

        self.router = Some(router);
        self
    }

    /// Add an HTTP endpoint to the router.
    ///
    /// # Type Parameters
    /// - `E` - The endpoint type implementing `HttpEndpoint` and `FromInjector`.
    #[allow(clippy::missing_panics_doc)]
    #[cfg(feature = "openapi")]
    fn add_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint + FromInjector + openapi::HttpOperation + Send + Sync + 'static,
    {
        if let Some(openapi_document_builder) = &mut self.openapi_document_builder {
            openapi_document_builder.add_operation::<E>();
        }

        self.add_hidden_endpoint::<E>()
    }

    /// Add an HTTP endpoint to the router.
    ///
    /// # Type Parameters
    /// - `E` - The endpoint type implementing `HttpEndpoint` and `FromInjector`.
    #[allow(clippy::missing_panics_doc)]
    #[cfg(not(feature = "openapi"))]
    fn add_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint + FromInjector + Send + Sync + 'static,
    {
        self.add_hidden_endpoint::<E>()
    }

    fn add_controller<C>(&mut self) -> &mut Self
    where
        C: HttpController,
    {
        let builder = HttpEndpointsBuilderImpl { builder: self };
        C::build_endpoints(builder);

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

impl ApplicationPartBuilder for HttpApplicationPartBuilder {
    type ApplicationPart = HttpApplicationPart;

    fn build(self, service_provider: ServiceProvider) -> ConstructionResult<Self::ApplicationPart> {
        #[cfg(feature = "openapi")]
        let openapi_document = self
            .openapi_document_builder
            .map(HttpDocumentBuilder::build)
            .transpose()
            .map_err(ConstructionError::Custom)?;

        Ok(HttpApplicationPart::new(
            service_provider,
            self.bind_address,
            self.router
                .expect("router not taken as we have a mut ref to self so no one else can take it"),
            #[cfg(feature = "openapi")]
            openapi_document,
            #[cfg(feature = "openapi")]
            self.openapi_path,
            #[cfg(feature = "axum-client-ip")]
            self.client_ip_source,
        ))
    }
}

/// The HTTP application part.
pub struct HttpApplicationPart {
    /// The service provider for dependency injection.
    service_provider: ServiceProvider,
    /// The address to bind the HTTP server to.
    bind_address: HttpBindAddress,
    /// The router instance.
    router: Option<Router>,
    /// The built `OpenAPI` document.
    #[cfg(feature = "openapi")]
    openapi_document: Option<std::sync::Arc<HttpDocument>>,
    /// The path to serve the `OpenAPI` document and Swagger UI at.
    #[cfg(feature = "openapi")]
    openapi_path: Cow<'static, str>,
    /// The client IP source configuration.
    #[cfg(feature = "axum-client-ip")]
    client_ip_source: axum_client_ip::ClientIpSource,
}

impl HttpApplicationPart {
    /// Create a new `HttpApplicationPart`.
    ///
    /// # Paramaters
    /// - `service_provider` - The service provider for dependency injection.
    /// - `bind_address` - The address to bind the HTTP server to.
    /// - `router` - The router instance.
    /// - `openapi_document` - The built `OpenAPI` document.
    /// - `client_ip_source` - The client IP source configuration.
    pub(crate) fn new(
        service_provider: ServiceProvider,
        bind_address: HttpBindAddress,
        router: Router,
        #[cfg(feature = "openapi")] openapi_document: Option<HttpDocument>,
        #[cfg(feature = "openapi")] openapi_path: Cow<'static, str>,
        #[cfg(feature = "axum-client-ip")] client_ip_source: axum_client_ip::ClientIpSource,
    ) -> Self {
        Self {
            service_provider,
            bind_address,
            router: Some(router),
            #[cfg(feature = "openapi")]
            openapi_document: openapi_document.map(std::sync::Arc::new),
            #[cfg(feature = "openapi")]
            openapi_path,
            #[cfg(feature = "axum-client-ip")]
            client_ip_source,
        }
    }
}

/// Middleware to insert a `ServiceScope` into each request's extensions.
///
/// # Paramaters
/// - `service_provider` - The service provider for dependency injection.
/// - `request` - The incoming HTTP request.
/// - `next` - The next middleware or handler in the chain.
async fn service_scope_middleware(
    State(service_provider): State<ServiceProvider>,
    mut request: Request,
    next: Next,
) -> Response {
    let scope = service_provider
        .resolve::<ServiceScope>()
        .expect("ServiceScope is always registered as scoped service in the ServiceProvider");

    request.extensions_mut().insert(scope);
    next.run(request).await
}

impl ApplicationPart for HttpApplicationPart {
    type Error = HttpError;

    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<(), Self::Error> {
        let mut router = self
            .router
            .take()
            .ok_or(HttpError::ApplicationAlreadyRunning)?;

        #[cfg(feature = "openapi")]
        if let Some(openapi_document) = &self.openapi_document {
            let title = &openapi_document.title();
            let openapi_path = self.openapi_path.as_ref();

            router = router.route(
                &format!("{openapi_path}-json"),
                axum::routing::get({
                    let openapi_document = openapi_document.clone();
                    async move || axum::Json(openapi_document)
                }),
            );

            #[cfg(feature = "yaml")]
            {
                router = router.route(
                    &format!("{openapi_path}-yaml"),
                    axum::routing::get({
                        let openapi_document = openapi_document.clone();
                        async move || Yaml(openapi_document)
                    }),
                );
            }

            router = router.serve_swagger_ui(openapi_path, &format!("{openapi_path}-json"), title);
        }

        router = router.layer(axum::middleware::from_fn_with_state(
            self.service_provider.clone(),
            service_scope_middleware,
        ));

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

impl HttpApplicationPart {
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
        app: Router,
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
        app: Router,
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
