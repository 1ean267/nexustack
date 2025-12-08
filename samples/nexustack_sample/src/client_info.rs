/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::response::{GetOneHttpResponse, InternalServerError};
use nexustack::{
    ApplicationBuilder,
    http::{Http, HttpApplicationBuilder, http_controller},
    inject::injectable,
    module,
};
use std::{
    convert::Infallible,
    net::{IpAddr, SocketAddr},
};

/// Extension trait to add the `ClientInfo` module to the application builder.
#[module(features = "Http")]
pub trait ClientInfoModule {
    /// Adds the `ClientInfo` module to the application builder.
    fn add_client_info(self) -> impl ApplicationBuilder {
        self.configure_http(|http_builder| {
            http_builder.add_controller::<ClientInfoController>();
        })
    }
}

#[derive(Debug, Clone)]
pub struct ClientInfoController;

/// HTTP controller for retrieving client information.
#[http_controller(tags = "ClientInfo")]
impl ClientInfoController {
    #[ctor]
    const fn new() -> Self {
        Self {}
    }

    /// Retrieves client information.
    ///
    /// # Parameters
    /// - `ip_address`: The IP address of the client making the request.
    #[get(route = "/api/client_info")]
    pub async fn get(
        &self,
        #[ip_address] ip_address: Option<IpAddr>,
    ) -> Result<GetOneHttpResponse<String>, Infallible> {
        let ip_address = if let Some(ip_address) = ip_address {
            either::Either::Left(ip_address)
        } else {
            either::Either::Right("Unknown")
        };

        Ok(GetOneHttpResponse(format!(
            "Your IP Address: {}",
            ip_address
        )))
    }

    /// A test endpoint to demonstrate path parameters.
    /// # Parameters
    /// - `a`: The first path parameter.
    /// - `b`: The second path parameter.
    #[get(route = "/api/client_info/{a}/test/{b}")]
    pub async fn test(
        &self,
        #[param] a: String,
        #[param] b: String,
    ) -> Result<GetOneHttpResponse<(String, String)>, InternalServerError> {
        Ok(GetOneHttpResponse((a, b)))
    }
}
