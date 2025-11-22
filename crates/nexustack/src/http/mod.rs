/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    ApplicationBuilder, ApplicationPartBuilder,
    application::{Index, Node},
};
use axum::{handler::Handler, routing::MethodRouter};
use serde::Deserialize;
use std::{
    borrow::Cow,
    convert::Infallible,
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
};

#[cfg(feature = "openapi")]
use crate::openapi;

mod endpoint;
mod feature;

#[cfg(feature = "openapi")]
mod swagger;

mod controller;
pub mod encoding;
pub mod response;

#[cfg(feature = "yaml")]
pub mod yaml;

pub use controller::{HttpController, HttpEndpointsBuilder};
pub use endpoint::HttpEndpoint;
pub use feature::{Http, HttpApplicationPart, HttpApplicationPartBuilder};

#[cfg(feature = "derive")]
pub use nexustack_macros::http_response;

/// Enum representing supported HTTP methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HttpMethod {
    /// HTTP GET method.
    Get,
    /// HTTP POST method.
    Post,
    /// HTTP PUT method.
    Put,
    /// HTTP DELETE method.
    Delete,
    /// HTTP PATCH method.
    Patch,
    /// HTTP OPTIONS method.
    Options,
    /// HTTP HEAD method.
    Head,
    /// HTTP TRACE method.
    Trace,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl HttpMethod {
    /// Returns the string representation of the HTTP method.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Options => "OPTIONS",
            Self::Head => "HEAD",
            Self::Trace => "TRACE",
        }
    }

    /// Returns an Axum `MethodRouter` for the given handler and HTTP method.
    ///
    /// # Type Parameters
    /// * `H` - The handler type.
    /// * `T` - The handler's input type.
    /// * `S` - The application state type.
    ///
    /// # Arguments
    /// * `handler` - The handler function or closure.
    pub(crate) fn route<H, T, S>(self, handler: H) -> MethodRouter<S, Infallible>
    where
        H: Handler<T, S>,
        T: 'static,
        S: Clone + Send + Sync + 'static,
    {
        match self {
            Self::Get => axum::routing::get(handler),
            Self::Post => axum::routing::post(handler),
            Self::Put => axum::routing::put(handler),
            Self::Delete => axum::routing::delete(handler),
            Self::Patch => axum::routing::patch(handler),
            Self::Options => axum::routing::options(handler),
            Self::Head => axum::routing::head(handler),
            Self::Trace => axum::routing::trace(handler),
        }
    }
}

/// Represents errors that can occur while parsing an HTTP bind address.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum HttpBindAddressParseErr {
    /// Error indicating that the Unix socket path is empty.
    #[error("Unix socket path cannot be empty")]
    EmptyUnixPath,
    /// Error indicating that no TCP address was provided.
    #[error("No TCP address provided")]
    NoTcpAddressProvided,
    /// Error indicating that the TCP address is invalid.
    #[error("Invalid TCP address: {0}")]
    InvalidTcpAddress(String),
}

/// Represents an HTTP bind address, either Unix socket or TCP.
#[derive(Debug, Clone)]
pub enum HttpBindAddress {
    /// Unix socket address.
    Unix(Cow<'static, Path>),
    /// TCP socket address(es).
    Tcp(Cow<'static, [SocketAddr]>),
}

impl FromStr for HttpBindAddress {
    type Err = HttpBindAddressParseErr;

    /// Parses a string into an `HttpBindAddress`.
    ///
    /// # Arguments
    /// * `s` - The string to parse as a bind address.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(path) = s.strip_prefix("unix:") {
            if path.is_empty() {
                return Err(HttpBindAddressParseErr::EmptyUnixPath);
            }
            Ok(Self::Unix(Cow::Owned(PathBuf::from(path.to_string()))))
        } else {
            let addrs: Result<Vec<SocketAddr>, _> = s
                .split(',')
                .map(|s| {
                    s.trim()
                        .parse()
                        .map_err(|_| HttpBindAddressParseErr::InvalidTcpAddress(s.to_string()))
                })
                .collect();
            match addrs {
                Ok(addrs) if !addrs.is_empty() => Ok(Self::Tcp(Cow::Owned(addrs))),
                Ok(_) => Err(HttpBindAddressParseErr::NoTcpAddressProvided),
                Err(e) => Err(e),
            }
        }
    }
}

impl<'de> Deserialize<'de> for HttpBindAddress {
    /// Deserializes an `HttpBindAddress` from a string.
    ///
    /// # Arguments
    /// * `deserializer` - The deserializer to use.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = HttpBindAddress;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a Unix or TCP bind address")
            }

            /// Visit a string value and attempt to parse it as an `HttpBindAddress`.
            ///
            /// # Arguments
            /// * `v` - The string value to visit.
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v.starts_with("unix:") {
                    let mut path = v;
                    let _ = path.drain(..5);
                    if path.is_empty() {
                        return Err(E::custom("Unix socket path cannot be empty"));
                    }
                    Ok(HttpBindAddress::Unix(Cow::Owned(PathBuf::from(path))))
                } else {
                    let addrs = v
                        .split(',')
                        .map(|s| s.trim().parse())
                        .collect::<Result<Vec<_>, _>>();
                    match addrs {
                        Ok(addrs) if !addrs.is_empty() => {
                            Ok(HttpBindAddress::Tcp(Cow::Owned(addrs)))
                        }
                        Ok(_) => Err(E::custom("At least one TCP address must be provided")),
                        Err(e) => Err(E::custom(format!("Invalid TCP address: {e}"))),
                    }
                }
            }

            /// Visit a string slice and attempt to parse it as an `HttpBindAddress`.
            ///
            /// # Arguments
            /// * `v` - The string slice to visit.
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if let Some(path) = v.strip_prefix("unix:") {
                    if path.is_empty() {
                        return Err(E::custom("Unix socket path cannot be empty"));
                    }
                    Ok(HttpBindAddress::Unix(Cow::Owned(PathBuf::from(
                        path.to_string(),
                    ))))
                } else {
                    let addrs: Result<Vec<SocketAddr>, _> =
                        v.split(',').map(|s| s.trim().parse()).collect();
                    match addrs {
                        Ok(addrs) if !addrs.is_empty() => {
                            Ok(HttpBindAddress::Tcp(Cow::Owned(addrs)))
                        }
                        Ok(_) => Err(E::custom("At least one TCP address must be provided")),
                        Err(e) => Err(E::custom(format!("Invalid TCP address: {e}"))),
                    }
                }
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

/// Trait for building HTTP application parts.
pub trait HttpApplicationBuilder {
    /// The application chain type.
    type Chain: ApplicationPartBuilder;

    /// Adds an HTTP application part to the builder.
    ///
    /// # Arguments
    /// * `bind_address` - The address to bind the HTTP server to.
    /// * `openapi_document_builder` - The `OpenAPI` document builder.
    #[cfg(feature = "openapi")]
    fn add_http_with_openapi(
        self,
        bind_address: HttpBindAddress,
        openapi_document_builder: openapi::HttpDocumentBuilder,
    ) -> impl ApplicationBuilder<Chain = Node<HttpApplicationPartBuilder, Self::Chain>>;

    /// Adds an HTTP application part to the builder.
    ///
    /// # Arguments
    /// * `bind_address` - The address to bind the HTTP server to.
    /// * `openapi_document_builder` - The `OpenAPI` document builder.
    fn add_http(
        self,
        bind_address: HttpBindAddress,
    ) -> impl ApplicationBuilder<Chain = Node<HttpApplicationPartBuilder, Self::Chain>>;

    /// Configures the HTTP application part.
    ///
    /// # Arguments
    /// * `configure` - A closure to configure the HTTP application part builder.
    fn configure_http<I, F>(self, configure: F) -> impl ApplicationBuilder<Chain = Self::Chain>
    where
        I: Index,
        F: FnOnce(&mut Self::Chain),
        Self::Chain: Http<I>;
}

impl<B: ApplicationBuilder> HttpApplicationBuilder for B {
    type Chain = B::Chain;

    /// Adds an HTTP application part to the builder.
    ///
    /// # Arguments
    /// * `bind_address` - The address to bind the HTTP server to.
    /// * `openapi_document_builder` - The `OpenAPI` document builder.
    #[cfg(feature = "openapi")]
    fn add_http_with_openapi(
        self,
        bind_address: HttpBindAddress,
        openapi_document_builder: openapi::HttpDocumentBuilder,
    ) -> impl ApplicationBuilder<Chain = Node<HttpApplicationPartBuilder, Self::Chain>> {
        self.add_application_part_with_factory(|| {
            HttpApplicationPartBuilder::new(bind_address, Some(openapi_document_builder))
        })
    }

    /// Adds an HTTP application part to the builder.
    ///
    /// # Arguments
    /// * `bind_address` - The address to bind the HTTP server to.
    /// * `openapi_document_builder` - The `OpenAPI` document builder.
    fn add_http(
        self,
        bind_address: HttpBindAddress,
    ) -> impl ApplicationBuilder<Chain = Node<HttpApplicationPartBuilder, Self::Chain>> {
        self.add_application_part_with_factory(|| {
            HttpApplicationPartBuilder::new(
                bind_address,
                #[cfg(feature = "openapi")]
                None,
            )
        })
    }

    /// Configures the HTTP application part.
    ///
    /// # Arguments
    /// * `configure` - A closure to configure the HTTP application part builder.
    fn configure_http<I, F>(self, configure: F) -> impl ApplicationBuilder<Chain = Self::Chain>
    where
        I: Index,
        F: FnOnce(&mut Self::Chain),
        Self::Chain: Http<I>,
    {
        self.configure_application_part(configure)
    }
}
