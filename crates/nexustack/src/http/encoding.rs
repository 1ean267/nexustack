/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use std::convert::Infallible;

use mime::Mime;
use serde::Serialize;

#[cfg(feature = "openapi")]
use crate::openapi;

// TODO: Rename
pub trait Encoder {
    type Context;

    fn into_response<T: Serialize>(
        self,
        status_code: axum::http::StatusCode,
        value: T,
        context: Self::Context,
    ) -> axum::response::Response;
}

// TODO: Rename
#[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
pub struct DefaultEncoder;

pub struct Accept(Vec<Mime>);

impl<S> axum::extract::FromRequestParts<S> for Accept {
    type Rejection = axum::http::StatusCode;

    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async {
            // TODO: Can we implement this without heap alloocation (at least in common cases)?
            if let Some(accept_header) = parts.headers.get(axum::http::header::ACCEPT) {
                let accept_str = accept_header
                    .to_str()
                    .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;

                let mime_types: Vec<Mime> = accept_str
                    .split(',')
                    // TODO: Is it the right strategy to ignore invalid mime types here?
                    .filter_map(|s| s.trim().parse::<Mime>().ok())
                    .collect();

                Ok(Accept(mime_types))
            } else {
                Ok(Accept(Vec::new()))
            }
        }
    }
}

impl Encoder for DefaultEncoder {
    type Context = Accept;

    fn into_response<T: Serialize>(
        self,
        status_code: axum::http::StatusCode,
        value: T,
        context: Self::Context,
    ) -> axum::response::Response {
        let mut expected_mime_types: Vec<Mime> = context.0;

        // Sort the expected_mim_types vector by quality value
        expected_mime_types.sort_by(|a, b| {
            // TODO: Is it the right strategy to ignore invalid quality values here?
            let quality_a = a
                .get_param("q")
                .and_then(|q| q.as_str().parse::<f32>().ok())
                .unwrap_or(1.0);
            let quality_b = b
                .get_param("q")
                .and_then(|q| q.as_str().parse::<f32>().ok())
                .unwrap_or(1.0);

            // Reverse the order to sort in descending order of quality
            quality_b
                .partial_cmp(&quality_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        fn encode_json<Q: Serialize>(
            status_code: axum::http::StatusCode,
            value: Q,
        ) -> axum::response::Response {
            axum::response::IntoResponse::into_response((status_code, axum::Json(value)))
        }

        #[cfg(feature = "yaml")]
        fn encode_yaml<Q: Serialize>(
            status_code: axum::http::StatusCode,
            value: Q,
        ) -> axum::response::Response {
            axum::response::IntoResponse::into_response((
                status_code,
                crate::http::yaml::Yaml(value),
            ))
        }

        fn encode_unsupported<Q>(
            _status_code: axum::http::StatusCode,
            _value: Q,
        ) -> axum::response::Response {
            axum::response::IntoResponse::into_response(UnsupportedMediaType)
        }

        let encode: fn(status_code: axum::http::StatusCode, value: T) -> axum::response::Response =
            expected_mime_types
                .iter()
                .filter_map(|expected_mime| match expected_mime.essence_str() {
                    "application/json" => Some(encode_json::<T> as _),
                    #[cfg(feature = "yaml")]
                    "application/yaml" | "application/x-yaml" => Some(encode_yaml::<T> as _),
                    "application/*" => Some(encode_json::<T> as _),
                    "*/*" => Some(encode_json::<T> as _),
                    _ => None,
                })
                .next()
                .unwrap_or(encode_unsupported as _);

        encode(status_code, value)
    }
}

#[cfg(feature = "openapi")]
impl openapi::HttpContentType for DefaultEncoder {
    fn describe<T: openapi::Schema, B>(mut content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: openapi::HttpContentTypeBuilder,
    {
        content_type_builder.collect_content_type(
            "application/json",
            None,
            false,
            <T as openapi::Schema>::describe,
        );

        #[cfg(feature = "yaml")]
        content_type_builder.collect_content_type(
            "application/yaml",
            None,
            false,
            <T as openapi::Schema>::describe,
        );

        content_type_builder.end()
    }
}

pub struct UnsupportedMediaType;

impl axum::response::IntoResponse for UnsupportedMediaType {
    fn into_response(self) -> axum::response::Response {
        axum::response::Response::builder()
            .status(axum::http::StatusCode::NOT_ACCEPTABLE)
            .body("Unsupported Media Type".into())
            .unwrap()
    }
}

// TODO: Rename
#[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
pub struct JsonEncoder;

pub struct NoContext;

impl<S> axum::extract::FromRequestParts<S> for NoContext {
    type Rejection = Infallible;

    fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async { Ok(Self) }
    }
}

impl Encoder for JsonEncoder {
    type Context = NoContext;

    fn into_response<T: Serialize>(
        self,
        status_code: axum::http::StatusCode,
        value: T,
        _context: Self::Context,
    ) -> axum::response::Response {
        axum::response::IntoResponse::into_response((status_code, axum::Json(value)))
    }
}

#[cfg(feature = "openapi")]
impl openapi::HttpContentType for JsonEncoder {
    fn describe<T: openapi::Schema, B>(mut content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: openapi::HttpContentTypeBuilder,
    {
        content_type_builder.collect_content_type(
            "application/json",
            None,
            false,
            <T as openapi::Schema>::describe,
        );

        content_type_builder.end()
    }
}
