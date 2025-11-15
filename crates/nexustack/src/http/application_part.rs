/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    ApplicationPart, ApplicationPartBuilder,
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
    serve::Listener,
};
use std::{net::SocketAddr, path::PathBuf};
use tokio_util::sync::CancellationToken;

#[cfg(feature = "openapi")]
use crate::{http::swagger::SwaggerRouter, openapi};

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

/// Builder for the HTTP application part.
pub struct HttpApplicationPartBuilder {
    /// The address to bind the HTTP server to.
    bind_address: HttpBindAddress,
    /// The `OpenAPI` document builder.
    #[cfg(feature = "openapi")]
    openapi_document_builder: Option<openapi::HttpDocumentBuilder>,
    /// The router instance (wrapped in Option for internal mutability).
    router: Option<axum::Router>,
}

struct HttpEndpointsBuilderImpl<'a> {
    builder: &'a mut HttpApplicationPartBuilder,
}

impl HttpEndpointsBuilder for HttpEndpointsBuilderImpl<'_> {
    #[cfg(feature = "openapi")]
    fn add_endpoint<E>(&mut self)
    where
        E: HttpEndpoint + FromInjector + openapi::HttpOperation + Send + Sync + 'static,
        E::Request: Send,
        <<E as HttpEndpoint>::Response as IntoResponseWithContext<()>>::Context: Send,
    {
        self.builder.add_endpoint::<E>();
    }

    fn add_hidden_endpoint<E>(&mut self)
    where
        E: HttpEndpoint + FromInjector + Send + Sync + 'static,
        E::Request: Send,
        <<E as HttpEndpoint>::Response as IntoResponseWithContext<()>>::Context: Send,
    {
        self.builder.add_hidden_endpoint::<E>();
    }
}

impl HttpApplicationPartBuilder {
    /// Create a new `HttpApplicationPartBuilder`.
    ///
    /// # Arguments
    /// * `bind_address` - The address to bind the HTTP server to.
    /// * `openapi_document_builder` - The `OpenAPI` document builder.
    pub(crate) fn new(
        bind_address: HttpBindAddress,
        #[cfg(feature = "openapi")] openapi_document_builder: Option<openapi::HttpDocumentBuilder>,
    ) -> Self {
        Self {
            bind_address,
            #[cfg(feature = "openapi")]
            openapi_document_builder,
            router: Some(Router::new()),
        }
    }

    /// Configure the router using the provided function.
    ///
    /// # Arguments
    /// * `f` - A function that takes the current router and returns a new router.
    #[allow(clippy::missing_panics_doc)]
    pub fn configure_router(&mut self, f: impl FnOnce(Router) -> Router) -> &mut Self {
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
    /// * `E` - The endpoint type implementing `HttpEndpoint` and `FromInjector`.
    #[allow(clippy::missing_panics_doc)]
    pub fn add_hidden_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint + FromInjector + Send + Sync + 'static,
        E::Request: Send,
        <<E as HttpEndpoint>::Response as IntoResponseWithContext<()>>::Context: Send,
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
    /// * `E` - The endpoint type implementing `HttpEndpoint` and `FromInjector`.
    #[allow(clippy::missing_panics_doc)]
    #[cfg(feature = "openapi")]
    pub fn add_endpoint<E>(&mut self) -> &mut Self
    where
        E: HttpEndpoint + FromInjector + openapi::HttpOperation + Send + Sync + 'static,
        E::Request: Send,
        <<E as HttpEndpoint>::Response as IntoResponseWithContext<()>>::Context: Send,
    {
        if let Some(openapi_document_builder) = &mut self.openapi_document_builder {
            openapi_document_builder.add_operation::<E>();
        }

        self.add_hidden_endpoint::<E>()
    }

    pub fn add_controller<C>(&mut self) -> &mut Self
    where
        C: HttpController,
    {
        let builder = HttpEndpointsBuilderImpl { builder: self };
        C::build_endpoints(builder);

        self
    }
}

impl ApplicationPartBuilder for HttpApplicationPartBuilder {
    type ApplicationPart = HttpApplicationPart;

    fn build(self, service_provider: ServiceProvider) -> ConstructionResult<Self::ApplicationPart> {
        #[cfg(feature = "openapi")]
        let openapi_document = self.openapi_document_builder.map(|builder| builder.build());

        Ok(HttpApplicationPart::new(
            service_provider,
            self.bind_address,
            self.router
                .expect("router not taken as we have a mut ref to self so no one else can take it"),
            #[cfg(feature = "openapi")]
            openapi_document,
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
    openapi_document: Option<std::sync::Arc<openapi::json::specification::OpenAPIObject>>,
}

impl HttpApplicationPart {
    /// Create a new `HttpApplicationPart`.
    ///
    /// # Arguments
    /// * `service_provider` - The service provider for dependency injection.
    /// * `bind_address` - The address to bind the HTTP server to.
    /// * `router` - The router instance.
    pub(crate) fn new(
        service_provider: ServiceProvider,
        bind_address: HttpBindAddress,
        router: Router,
        #[cfg(feature = "openapi")] openapi_document: Option<
            openapi::json::specification::OpenAPIObject,
        >,
    ) -> Self {
        Self {
            service_provider,
            bind_address,
            router: Some(router),
            #[cfg(feature = "openapi")]
            openapi_document: openapi_document.map(std::sync::Arc::new),
        }
    }
}

/// Middleware to insert a `ServiceScope` into each request's extensions.
///
/// # Arguments
/// * `service_provider` - The service provider for dependency injection.
/// * `request` - The incoming HTTP request.
/// * `next` - The next middleware or handler in the chain.
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
            #[cfg(feature = "openapi")]
            let title = &openapi_document.as_ref().info.title;
            // TODO: Make configurable
            #[cfg(feature = "openapi")]
            let openapi_path = "/api";

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
                self.run_with_listener(router, listener, cancellation_token)
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
                self.run_with_listener(router, listener, cancellation_token)
                    .await?;

                Ok(())
            }
        }
    }
}

impl HttpApplicationPart {
    /// Run the HTTP server with the given listener.
    ///
    /// # Type Parameters
    /// * `L` - The listener type implementing `Listener`.
    ///
    /// # Arguments
    /// * `app` - The Axum router to serve.
    /// * `listener` - The listener to accept connections from.
    /// * `cancellation_token` - A token to signal graceful shutdown.
    async fn run_with_listener<L>(
        &self,
        app: Router,
        listener: L,
        cancellation_token: CancellationToken,
    ) -> Result<(), <Self as ApplicationPart>::Error>
    where
        L: Listener,
        L::Addr: std::fmt::Debug,
    {
        axum::serve(listener, app)
            .with_graceful_shutdown(cancellation_token.cancelled_owned())
            .await
            .map_err(HttpError::ServeError)
    }
}
