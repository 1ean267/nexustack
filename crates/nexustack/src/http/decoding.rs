/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use axum::{
    Json,
    body::Body,
    extract::{FromRequest, FromRequestParts as _, rejection::JsonRejection},
    http::Request,
    response::IntoResponse,
};
use mime::Mime;
use serde::de::DeserializeOwned;

#[cfg(feature = "yaml")]
use crate::http::yaml::{Yaml, YamlRejection};
#[cfg(feature = "openapi")]
use crate::openapi;

/// A trait for decoding HTTP requests.
///
/// Implementors of this trait define how to deserialize a request body into a specific type (e.g., JSON, YAML).
/// The `Decoder` trait is generic over the type to be deserialized and the associated rejection type.
///
/// # Associated Types
/// - `Rejection` - The type of error returned when decoding fails.
///
/// # Example
/// ```
/// use axum::http::Request;
/// use nexustack::http::decoding::{Decoder, DefaultDecoder};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct MyData {
///     field: String,
/// }
///
/// let decoder = DefaultDecoder;
/// let request = Request::builder()
///     .header("Content-Type", "application/json")
///     .body(Body::from("{\"field\":\"value\"}"))
///     .unwrap();
///
/// let data: MyData = decoder.from_request(request).await.unwrap();
/// assert_eq!(data.field, "value");
/// ```
pub trait Decoder {
    /// The type of error returned when decoding fails.
    type Rejection: IntoResponse;

    // TODO: Support non-owned deserialization
    /// Decodes a request body into the specified type.
    ///
    /// # Parameters
    /// - `request` - The HTTP request to decode.
    ///
    /// # Returns
    ///
    /// A future that resolves to the decoded value or a rejection error.
    fn decode_request<T: DeserializeOwned>(
        request: Request<Body>,
    ) -> impl Future<Output = Result<T, Self::Rejection>> + Send;
}

/// Represents the `ContentType` header in HTTP requests.
///
/// This struct parses the `ContentType` header into a MIME type, which can be used to determine the request format.
pub struct ContentType(Mime);

/// Represents errors that can occur during the decoding of HTTP requests.
///
/// This enum provides detailed error variants for different failure scenarios, such as invalid media types,
/// unsupported media types, or deserialization errors.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DecodeError {
    /// Indicates that the request body could not be deserialized as JSON.
    ///
    /// This variant wraps the `JsonRejection` error from Axum.
    #[error(transparent)]
    JsonError(#[from] JsonRejection),

    /// Indicates that the request body could not be deserialized as YAML.
    ///
    /// This variant is only available when the `yaml` feature is enabled and wraps the `YamlRejection` error.
    #[cfg(feature = "yaml")]
    #[error(transparent)]
    YamlError(#[from] YamlRejection),

    /// Indicates that the `Content-Type` header contains an invalid media type.
    ///
    /// This variant includes an optional string representing the invalid media type.
    #[error("Invalid media type: {0:?}")]
    InvalidMediaType(Option<String>),

    /// Indicates that the `Content-Type` header is missing from the request.
    #[error("Media type unspecified")]
    MediaTypeUnspecified,

    /// Indicates that the `Content-Type` header specifies a media type that is not supported.
    ///
    /// This variant includes the unsupported MIME type.
    #[error("Unsupported media type: {0}")]
    UnsupportedMediaType(Mime),
}

impl<S> axum::extract::FromRequestParts<S> for ContentType {
    type Rejection = DecodeError;

    #[allow(clippy::manual_async_fn)]
    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async {
            // TODO: Can we implement this without heap allocation (at least in common cases)?
            if let Some(content_type_header) = parts.headers.get(axum::http::header::CONTENT_TYPE) {
                let raw_mime_type = content_type_header
                    .to_str()
                    .map_err(|_| DecodeError::InvalidMediaType(None))?;

                let mime_type = raw_mime_type
                    .parse()
                    .map_err(|_| DecodeError::InvalidMediaType(Some(raw_mime_type.to_string())))?;

                Ok(Self(mime_type))
            } else {
                Err(DecodeError::MediaTypeUnspecified)
            }
        }
    }
}

impl IntoResponse for DecodeError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::JsonError(err) => err.into_response(),
            #[cfg(feature = "yaml")]
            Self::YamlError(err) => err.into_response(),
            Self::InvalidMediaType(Some(media_type)) => axum::response::Response::builder()
                .status(axum::http::StatusCode::BAD_REQUEST)
                .body(format!("Invalid Media Type: {media_type}").into())
                .unwrap(),
            Self::InvalidMediaType(None) => axum::response::Response::builder()
                .status(axum::http::StatusCode::BAD_REQUEST)
                .body("Invalid Media Type".into())
                .unwrap(),
            Self::MediaTypeUnspecified => axum::response::Response::builder()
                .status(axum::http::StatusCode::BAD_REQUEST)
                .body("Media Type Unspecified".into())
                .unwrap(),
            Self::UnsupportedMediaType(mime) => axum::response::Response::builder()
                .status(axum::http::StatusCode::UNSUPPORTED_MEDIA_TYPE)
                .body(format!("Unsupported Media Type: {mime}").into())
                .unwrap(),
        }
    }
}

/// The default decoder for HTTP requests.
///
/// This decoder supports multiple MIME types, such as JSON and YAML (if enabled).
/// It uses the `ContentType` header to determine the format of the incoming request.
///
/// # Example
/// ```
/// use axum::http::Request;
/// use nexustack::http::decoding::{DefaultDecoder, Decoder};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct MyData {
///     field: String,
/// }
///
/// let decoder = DefaultDecoder;
/// let request = Request::builder()
///     .header("Content-Type", "application/json")
///     .body(Body::from("{\"field\":\"value\"}"))
///     .unwrap();
///
/// let data: MyData = decoder.from_request(request).await.unwrap();
/// assert_eq!(data.field, "value");
/// ```
#[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
pub struct DefaultDecoder;

impl Decoder for DefaultDecoder {
    type Rejection = DecodeError;

    async fn decode_request<T: DeserializeOwned>(
        request: Request<Body>,
    ) -> Result<T, Self::Rejection> {
        let (mut parts, body) = request.into_parts();
        let content_type = ContentType::from_request_parts(&mut parts, &()).await?.0;

        Ok(match content_type.essence_str() {
            "application/json" => {
                Json::<T>::from_request(Request::<Body>::from_parts(parts, body), &())
                    .await
                    .map_err(DecodeError::JsonError)?
                    .0
            }
            #[cfg(feature = "yaml")]
            "application/yaml" | "application/x-yaml" => {
                Yaml::<T>::from_request(Request::<Body>::from_parts(parts, body), &())
                    .await
                    .map_err(DecodeError::YamlError)?
                    .0
            }
            _ => return Err(DecodeError::UnsupportedMediaType(content_type)),
        })
    }
}

#[cfg(feature = "openapi")]
impl<T: openapi::Schema> openapi::HttpContentType<T> for DefaultDecoder {
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

/// A JSON decoder for HTTP requests.
///
/// This decoder is used to deserialize requests encoded in JSON format. It does not require any additional context.
///
/// # Example
/// ```
/// use axum::http::Request;
/// use nexustack::http::decoding::{JsonDecoder, Decoder};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct MyData {
///     field: String,
/// }
///
/// let decoder = JsonDecoder;
/// let request = Request::builder()
///     .header("Content-Type", "application/json")
///     .body(Body::from("{\"field\":\"value\"}"))
///     .unwrap();
///
/// let data: MyData = decoder.from_request(request).await.unwrap();
/// assert_eq!(data.field, "value");
/// ```
#[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
pub struct JsonDecoder;

impl Decoder for JsonDecoder {
    type Rejection = JsonRejection;

    async fn decode_request<T: DeserializeOwned>(
        request: Request<Body>,
    ) -> Result<T, Self::Rejection> {
        Json::<T>::from_request(request, &())
            .await
            .map(|json| json.0)
    }
}

#[cfg(feature = "openapi")]
impl<T: openapi::Schema> openapi::HttpContentType<T> for JsonDecoder {
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
