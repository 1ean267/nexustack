/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::response::{EmptyResponse, GetOneHttpResponse, InternalServerError};
use axum_extra::headers::UserAgent;
use nexustack::{
    ApplicationBuilder,
    http::{
        Http, HttpApplicationBuilder, HttpEndpointContext, headers::Origin, http_controller,
        session::SessionExtractor, user::NoUser,
    },
    inject::injectable,
    module,
};
use std::{
    borrow::Cow,
    convert::Infallible,
    net::{IpAddr, SocketAddr},
};

/// Extension trait to add the `ClientInfo` module to the application builder.
#[module(features = "Http<Session = (), User = NoUser>")] // or features = "Http<Session: Into<()>>"
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

    /// A test endpoint to demonstrate header extraction.
    /// # Parameters
    /// - `user_agent`: The value of the 'User-Agent' header.
    #[get(route = "/api/client_info/header_test")]
    pub async fn header_test(
        &self,
        #[header] user_agent: UserAgent,
    ) -> Result<GetOneHttpResponse<String>, InternalServerError> {
        Ok(GetOneHttpResponse(
            user_agent.to_string(), // user_agent.map_or("[None]".to_string(), user_agent.to_string()),
        ))
    }

    /// A test endpoint to demonstrate optional header extraction.
    /// # Parameters
    /// - `user_agent`: The value of the 'User-Agent' header.
    #[get(route = "/api/client_info/optional_header_test")]
    pub async fn optional_header_test(
        &self,
        #[header(default)] user_agent: Option<UserAgent>,
    ) -> Result<GetOneHttpResponse<String>, InternalServerError> {
        Ok(GetOneHttpResponse(
            user_agent.map_or("[None]".to_string(), |user_agent| user_agent.to_string()),
        ))
    }

    /// A test endpoint to demonstrate header extraction.
    /// # Parameters
    /// - `custom_header`: The value of the custom header.
    /// - `other_custom_header`: The optional value of the other custom header.
    #[get(route = "/api/client_info/custom_header_test")]
    pub async fn custom_header_test(
        &self,
        #[header(rename = "X-Custom-Header", default)] custom_header: Option<String>,
        #[header(rename = "X-Other-Custom-Header")] other_custom_header: String,
    ) -> Result<GetOneHttpResponse<String>, InternalServerError> {
        Ok(GetOneHttpResponse(format!(
            "X-Custom-Header: {}, X-Other-Custom-Header: {other_custom_header}",
            custom_header.map_or("[NONE]".to_string(), |custom_header| custom_header
                .to_string())
        )))
    }

    /// A test endpoint to demonstrate session usage
    ///# Parameters
    /// - `session`: The current session
    #[get(route = "/api/client_info/test_session")]
    pub async fn test_session(
        &self,
        #[session] session: (),
    ) -> Result<EmptyResponse, InternalServerError> {
        Ok(EmptyResponse)
    }

    /// A test endpoint to demonstrate user usage
    ///# Parameters
    /// - `user`: The current user
    #[get(route = "/api/client_info/test_user")]
    pub async fn test_user(
        &self,
        #[user] user: NoUser,
    ) -> Result<EmptyResponse, InternalServerError> {
        Ok(EmptyResponse)
    }
}
