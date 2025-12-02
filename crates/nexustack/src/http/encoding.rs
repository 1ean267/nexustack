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

/// A trait for encoding HTTP responses.
///
/// Implementors of this trait define how to serialize a response into a specific format (e.g., JSON, YAML).
/// The `Encoder` trait is generic over a `Context` type, which provides additional information required during encoding.
///
/// # Associated Types
/// - `Context` - The type of context required for encoding.
///
/// # Example
/// ```
/// struct MyEncoder;
///
/// impl Encoder for MyEncoder {
///     type Context = ();
///
///     fn into_response<T: Serialize>(
///         self,
///         status_code: axum::http::StatusCode,
///         value: T,
///         _context: Self::Context,
///     ) -> axum::response::Response {
///         axum::response::IntoResponse::into_response((status_code, axum::Json(value)))
///     }
/// }
/// ```
pub trait Encoder {
    /// The type of context required for encoding.
    type Context;

    /// Converts a value into an HTTP response.
    ///
    /// # Paramaters
    /// - `status_code` - The HTTP status code for the response.
    /// - `value` - The value to encode into the response.
    /// - `context` - The context required for encoding.
    fn into_response<T: Serialize>(
        // TODO: Rename to `encode_response`
        self,
        status_code: axum::http::StatusCode,
        value: T,
        context: Self::Context,
    ) -> axum::response::Response;
}

/// Represents the `Accept` header in HTTP requests.
///
/// This struct parses the `Accept` header into a list of MIME types, which can be used to determine the preferred response format.
pub struct Accept(Vec<Mime>);

impl<S> axum::extract::FromRequestParts<S> for Accept {
    type Rejection = axum::http::StatusCode;

    #[allow(clippy::manual_async_fn)]
    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async {
            // TODO: Can we implement this without heap allocation (at least in common cases)?
            if let Some(accept_header) = parts.headers.get(axum::http::header::ACCEPT) {
                let mime_types = accept_header
                    .to_str()
                    .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?
                    .split(',')
                    // TODO: Is it the right strategy to ignore invalid mime types here?
                    .filter_map(|s| s.trim().parse::<Mime>().ok())
                    .collect();

                Ok(Self(mime_types))
            } else {
                Ok(Self(Vec::new()))
            }
        }
    }
}

// TODO: Rename
/// The default encoder for HTTP responses.
///
/// This encoder supports multiple MIME types, such as JSON and YAML (if enabled).
/// It uses the `Accept` context to determine the preferred response format.
#[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
pub struct DefaultEncoder;

impl Encoder for DefaultEncoder {
    type Context = Accept;

    fn into_response<T: Serialize>(
        self,
        status_code: axum::http::StatusCode,
        value: T,
        context: Self::Context,
    ) -> axum::response::Response {
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
            axum::response::IntoResponse::into_response(NotAcceptable)
        }

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

        let encode: fn(status_code: axum::http::StatusCode, value: T) -> axum::response::Response =
            expected_mime_types
                .iter()
                .find_map(|expected_mime| match expected_mime.essence_str() {
                    "application/json" => Some(encode_json::<T> as _),
                    #[cfg(feature = "yaml")]
                    "application/yaml" | "application/x-yaml" => Some(encode_yaml::<T> as _),
                    "application/*" => Some(encode_json::<T> as _),
                    "*/*" => Some(encode_json::<T> as _),
                    _ => None,
                })
                .unwrap_or(encode_unsupported as _);

        encode(status_code, value)
    }
}

#[cfg(feature = "openapi")]
impl<T: openapi::Schema> openapi::HttpContentType<T> for DefaultEncoder {
    fn describe<B>(mut content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: openapi::HttpContentTypeBuilder,
    {
        content_type_builder.collect_content_type(
            "application/json",
            None,
            false,
            <T as openapi::Schema>::describe,
        )?;

        #[cfg(feature = "yaml")]
        content_type_builder.collect_content_type(
            "application/yaml",
            None,
            false,
            <T as openapi::Schema>::describe,
        )?;

        content_type_builder.end()
    }
}

/// A response type indicating that the requested media type is not supported.
///
/// This type is used to generate an HTTP `406 Not Acceptable` response when the server cannot produce a response matching the list of acceptable values defined in the request's `Accept` header.
pub struct NotAcceptable;

impl axum::response::IntoResponse for NotAcceptable {
    fn into_response(self) -> axum::response::Response {
        axum::response::Response::builder()
            .status(axum::http::StatusCode::NOT_ACCEPTABLE)
            .body("Not Acceptable".into())
            .unwrap()
    }
}

// TODO: Rename

/// A JSON encoder for serializing responses.
///
/// This encoder is used to serialize responses into JSON format. It does not require any additional context.
#[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
pub struct JsonEncoder;

/// A placeholder context type for encoders that do not require any context.
///
/// This type is used when an encoder does not need any additional information to process requests or responses.
pub struct NoContext;

impl<S> axum::extract::FromRequestParts<S> for NoContext {
    type Rejection = Infallible;

    #[allow(clippy::manual_async_fn)]
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
impl<T: openapi::Schema> openapi::HttpContentType<T> for JsonEncoder {
    fn describe<B>(mut content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: openapi::HttpContentTypeBuilder,
    {
        content_type_builder.collect_content_type(
            "application/json",
            None,
            false,
            <T as openapi::Schema>::describe,
        )?;

        content_type_builder.end()
    }
}
