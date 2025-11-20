/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use std::borrow::Cow;

use axum::extract::rejection::JsonRejection;
use nexustack::{
    http::{decoding::DecodeError, encoding::JsonEncoder, http_response},
    openapi,
};

/// A test type to verify that the macro is working.
#[http_response(status_code = "NO_CONTENT")]
pub struct EmptyResponse;

/// Represents a generic HTTP response for a "get one" operation.
#[derive(Debug)]
#[http_response(status_code = "OK")]
pub struct GetOneHttpResponse<T>(pub T);

/// Represents an HTTP error response for a "get one" operation.
#[derive(Debug)]
#[http_response(encoder = "JsonEncoder")]
pub enum GetOneHttpError {
    /// The requested entity was not found.
    #[http_response::variant(status_code = "NOT_FOUND")]
    NotFound,

    /// The request data was invalid.
    #[http_response::variant(status_code = "BAD_REQUEST")]
    ValidationError(Cow<'static, str>),

    /// An internal server error occurred.
    #[http_response::variant(status_code = "INTERNAL_SERVER_ERROR")]
    InternalServerError(SerializableError),
}

impl From<axum::extract::rejection::PathRejection> for GetOneHttpError {
    fn from(value: axum::extract::rejection::PathRejection) -> Self {
        use axum::extract::path::ErrorKind;

        // TODO: logging

        match value {
            axum::extract::rejection::PathRejection::FailedToDeserializePathParams(
                failed_to_deserialize_path_params,
            ) => match failed_to_deserialize_path_params.kind() {
                ErrorKind::Message(_)
                | ErrorKind::DeserializeError { .. }
                | ErrorKind::InvalidUtf8InPathParam { .. }
                | ErrorKind::ParseError { .. }
                | ErrorKind::ParseErrorAtIndex { .. }
                | ErrorKind::ParseErrorAtKey { .. } => Self::ValidationError(
                    format!("Invalid URL: {}", failed_to_deserialize_path_params.kind()).into(),
                ),
                ErrorKind::WrongNumberOfParameters { .. } | ErrorKind::UnsupportedType { .. } => {
                    Self::InternalServerError(SerializableError(anyhow::Error::msg(
                        failed_to_deserialize_path_params.kind().to_string(),
                    )))
                }
                _ => todo!(),
            },
            axum::extract::rejection::PathRejection::MissingPathParams(_) => {
                Self::InternalServerError(SerializableError(anyhow::Error::msg(
                    "No paths parameters found for matched route",
                )))
            }
            _ => Self::InternalServerError(SerializableError(anyhow::Error::msg(
                "Failed to extract path parameters for an unknown reason",
            ))),
        }
    }
}

impl From<DataAccessError> for GetOneHttpError {
    fn from(value: DataAccessError) -> Self {
        Self::InternalServerError(SerializableError(value.into()))
    }
}

/// Represents a generic HTTP response for a "get many" operation.
#[derive(Debug)]
#[http_response(status_code = "OK")]
pub struct GetManyHttpResponse<T>(pub Vec<T>);

/// Represents an HTTP error response for a "get one" operation.
#[derive(Debug)]
#[http_response(encoder = "JsonEncoder")]
pub enum GetManyHttpError {
    /// The request data was invalid.
    #[http_response::variant(status_code = "BAD_REQUEST")]
    ValidationError(Cow<'static, str>),

    /// An internal server error occurred.
    #[http_response::variant(status_code = "INTERNAL_SERVER_ERROR")]
    InternalServerError(SerializableError),
}

impl From<axum::extract::rejection::PathRejection> for GetManyHttpError {
    fn from(value: axum::extract::rejection::PathRejection) -> Self {
        use axum::extract::path::ErrorKind;

        // TODO: logging

        match value {
            axum::extract::rejection::PathRejection::FailedToDeserializePathParams(
                failed_to_deserialize_path_params,
            ) => match failed_to_deserialize_path_params.kind() {
                ErrorKind::Message(_)
                | ErrorKind::DeserializeError { .. }
                | ErrorKind::InvalidUtf8InPathParam { .. }
                | ErrorKind::ParseError { .. }
                | ErrorKind::ParseErrorAtIndex { .. }
                | ErrorKind::ParseErrorAtKey { .. } => Self::ValidationError(
                    format!("Invalid URL: {}", failed_to_deserialize_path_params.kind()).into(),
                ),
                ErrorKind::WrongNumberOfParameters { .. } | ErrorKind::UnsupportedType { .. } => {
                    Self::InternalServerError(SerializableError(anyhow::Error::msg(
                        failed_to_deserialize_path_params.kind().to_string(),
                    )))
                }
                _ => todo!(),
            },
            axum::extract::rejection::PathRejection::MissingPathParams(_) => {
                Self::InternalServerError(SerializableError(anyhow::Error::msg(
                    "No paths parameters found for matched route",
                )))
            }
            _ => Self::InternalServerError(SerializableError(anyhow::Error::msg(
                "Failed to extract path parameters for an unknown reason",
            ))),
        }
    }
}

impl From<axum::extract::rejection::QueryRejection> for GetManyHttpError {
    fn from(value: axum::extract::rejection::QueryRejection) -> Self {
        match value {
            axum::extract::rejection::QueryRejection::FailedToDeserializeQueryString(
                failed_to_deserialize_query_string,
            ) => Self::ValidationError(
                format!("Invalid query parameters: {failed_to_deserialize_query_string}").into(),
            ),
            _ => Self::InternalServerError(SerializableError(anyhow::Error::msg(
                "Failed to extract query parameters for an unknown reason",
            ))),
        }
    }
}

impl From<axum_extra::extract::QueryRejection> for GetManyHttpError {
    fn from(value: axum_extra::extract::QueryRejection) -> Self {
        match value {
            axum_extra::extract::QueryRejection::FailedToDeserializeQueryString(
                failed_to_deserialize_query_string,
            ) => Self::ValidationError(
                format!("Invalid query parameters: {failed_to_deserialize_query_string}").into(),
            ),
            _ => Self::InternalServerError(SerializableError(anyhow::Error::msg(
                "Failed to extract query parameters for an unknown reason",
            ))),
        }
    }
}

impl From<DataAccessError> for GetManyHttpError {
    fn from(value: DataAccessError) -> Self {
        Self::InternalServerError(SerializableError(value.into()))
    }
}

/// Represents a generic HTTP response for a "create" operation.
#[derive(Debug)]
#[http_response(status_code = "CREATED")]
pub struct HttpCreateResponse<T>(T);

/// Represents a generic HTTP response for an "update" or "delete" operation.
#[derive(Debug)]
#[http_response(status_code = "OK")]
pub struct HttpOperationResponse<T>(T);

/// Represents an HTTP error response for a "create", "update" or "delete" operation.
#[derive(Debug)]
#[http_response(encoder = "JsonEncoder")]
pub enum HttpOperationError {
    /// An entity was not found.
    #[http_response::variant(status_code = "NOT_FOUND")]
    NotFound,

    /// The request data was invalid.
    #[http_response::variant(status_code = "BAD_REQUEST")]
    ValidationError(Cow<'static, str>),

    /// An internal server error occurred.
    #[http_response::variant(status_code = "INTERNAL_SERVER_ERROR")]
    InternalServerError(SerializableError),
}

impl From<DecodeError> for HttpOperationError {
    fn from(value: DecodeError) -> Self {
        match value {
            DecodeError::JsonError(json_rejection) => {
                Self::ValidationError(format!("Deserialization error: {json_rejection}").into())
            }
            DecodeError::YamlError(yaml_rejection) => {
                Self::ValidationError(format!("Deserialization error: {yaml_rejection}").into())
            }
            DecodeError::InvalidMediaType(mime) => {
                Self::ValidationError(format!("Invalid media type: {mime:?}").into())
            }
            DecodeError::MediaTypeUnspecified => {
                Self::ValidationError("Media type unspecified".into())
            }
            DecodeError::UnsupportedMediaType(mime) => {
                Self::ValidationError(format!("Unsupported media type: {mime}").into())
            }
            err => Self::InternalServerError(err.into()),
        }
    }
}

impl From<JsonRejection> for HttpOperationError {
    fn from(value: JsonRejection) -> Self {
        Self::ValidationError(format!("Deserialization error: {value}").into())
    }
}

/// An internal server error occurred.
#[http_response(encoder = "JsonEncoder", status_code = "INTERNAL_SERVER_ERROR")]
pub struct InternalServerError(pub SerializableError);

impl<E> From<E> for InternalServerError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(error: E) -> Self {
        Self(error.into())
    }
}

#[derive(Debug)]
pub struct SerializableError(pub anyhow::Error);

impl<E> From<E> for SerializableError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(error: E) -> Self {
        Self(error.into())
    }
}

impl From<SerializableError> for Box<dyn std::error::Error + Send + Sync + 'static> {
    fn from(error: SerializableError) -> Self {
        error.0.into()
    }
}

impl From<SerializableError> for Box<dyn std::error::Error + Send + 'static> {
    fn from(error: SerializableError) -> Self {
        error.0.into()
    }
}

impl From<SerializableError> for Box<dyn std::error::Error + 'static> {
    fn from(error: SerializableError) -> Self {
        error.0.into()
    }
}

impl serde::Serialize for SerializableError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl openapi::Schema for SerializableError {
    type Example = &'static str;
    type Examples = <[&'static str; 1] as IntoIterator>::IntoIter;

    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            None,
            None,
            None,
            Some("An internal server error message."),
            || Ok(["An unexpected error occurred."]),
            false,
        )
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DataAccessError {
    #[error("Entity not found")]
    NotFound,
    #[error("Database error occurred")]
    DatabaseError(anyhow::Error),
}
