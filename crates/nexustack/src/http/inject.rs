/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    http::ConfigureRouter,
    inject::{ServiceProvider, ServiceScope},
};

impl ConfigureRouter for ServiceProvider {
    fn configure_router(&self, router: axum::Router) -> axum::Router {
        router.layer(axum::middleware::from_fn_with_state(
            self.clone(),
            service_scope_middleware,
        ))
    }
}

/// Middleware to insert a `ServiceScope` into each request's extensions.
///
/// # Paramaters
/// - `service_provider` - The service provider for dependency injection.
/// - `request` - The incoming HTTP request.
/// - `next` - The next middleware or handler in the chain.
async fn service_scope_middleware(
    axum::extract::State(service_provider): axum::extract::State<ServiceProvider>,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let scope = service_provider
        .resolve::<ServiceScope>()
        .expect("ServiceScope is always registered as scoped service in the ServiceProvider");

    request.extensions_mut().insert(scope);
    next.run(request).await
}
