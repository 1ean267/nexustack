/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use axum::http::header::ToStrError;
use axum_extra::typed_header::{TypedHeaderRejection, TypedHeaderRejectionReason};
use headers::authorization::Credentials;
use serde::Deserializer;
use std::fmt::Display;

pub use headers::*;

/// Represents an error that occurred while extracting a typed header from an HTTP request.
///
/// This struct contains information about which header failed to be extracted and the
/// specific reason for the failure. It is typically returned by header extraction functions
/// when a required header is missing or cannot be parsed correctly.
///
/// # Fields
///
/// * `name` - The name of the header that failed to be extracted
/// * `reason` - The specific reason why the extraction failed
///
/// # Examples
///
/// This error is usually handled by the framework's error handling mechanisms,
/// but can be inspected manually:
///
/// ```rust,ignore
/// match extract_header(request) {
///     Ok(header) => println!("Header: {:?}", header),
///     Err(ExtractHeaderError { name, reason }) => {
///         println!("Failed to extract header '{}': {:?}", name, reason);
///     }
/// }
/// ```
#[derive(Debug, thiserror::Error)]
#[error("Failed to extract header '{name}': {reason:?}")]
pub struct ExtractHeaderError {
    /// The name of the header that failed to be extracted.
    name: HeaderName,
    /// The specific reason why the header extraction failed.
    reason: ExtractHeaderErrorReason,
}

/// The specific reason why a header extraction failed.
///
/// This enum provides detailed information about what went wrong during
/// the header extraction process, allowing for better error handling and debugging.
#[derive(Debug)]
#[non_exhaustive]
pub enum ExtractHeaderErrorReason {
    /// The header was missing from the HTTP request.
    ///
    /// This occurs when a required header is not present in the request headers.
    Missing,
    /// An error occurred while parsing or processing the header value.
    ///
    /// This wraps any underlying error that occurred during header parsing,
    /// such as invalid format or unsupported values.
    Error(anyhow::Error),
}

impl ExtractHeaderError {
    /// Creates a new `ExtractHeaderError` with the given header name and reason.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the header that failed to be extracted
    /// * `reason` - The reason why the extraction failed
    ///
    /// # Returns
    ///
    /// A new `ExtractHeaderError` instance
    pub const fn new(name: HeaderName, reason: ExtractHeaderErrorReason) -> Self {
        Self { name, reason }
    }

    /// Returns the name of the header that failed to be extracted.
    ///
    /// # Returns
    ///
    /// A reference to the `HeaderName`
    pub const fn name(&self) -> &HeaderName {
        &self.name
    }

    /// Returns the reason why the header extraction failed.
    ///
    /// # Returns
    ///
    /// A reference to the `ExtractHeaderErrorReason`
    pub const fn reason(&self) -> &ExtractHeaderErrorReason {
        &self.reason
    }
}

impl From<TypedHeaderRejection> for ExtractHeaderError {
    fn from(rejection: TypedHeaderRejection) -> Self {
        let name = rejection.name().clone();

        if matches!(rejection.reason(), TypedHeaderRejectionReason::Missing) {
            return Self {
                name,
                reason: ExtractHeaderErrorReason::Missing,
            };
        }

        Self {
            name,
            reason: ExtractHeaderErrorReason::Error(anyhow::format_err!("{rejection}")),
        }
    }
}

/// An error that occurs during deserialization of a header value.
///
/// This error type is used when attempting to deserialize HTTP header values
/// into Rust types using Serde. It covers various failure modes that can occur
/// during the deserialization process.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum HeaderValueDeserializeError {
    /// The header value type is not supported for deserialization.
    ///
    /// This error occurs when trying to deserialize a header value into a type
    /// that is not supported by the `HeaderValueDeserializer`, such as numeric
    /// types or complex data structures.
    #[error("Invalid type for header value deserialization")]
    InvalidType,

    /// Failed to convert the header value to a string.
    ///
    /// This happens when the header value contains invalid UTF-8 bytes that
    /// cannot be converted to a string representation.
    #[error("Header value stringify error: {0}")]
    StringifyError(#[from] ToStrError),

    /// A custom deserialization error occurred.
    ///
    /// This variant wraps custom error messages that may be generated during
    /// the deserialization process by Serde visitors.
    #[error("Header value deserialization error: {0}")]
    Custom(String),
}

impl serde::de::Error for HeaderValueDeserializeError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Custom(msg.to_string())
    }
}

/// A deserializer for HTTP header values that implements the Serde `Deserializer` trait.
///
/// This struct allows HTTP header values to be deserialized into Rust types using Serde.
/// It supports deserializing header values into strings, byte sequences, and some basic types.
/// The deserializer is designed to work with the constraints of HTTP header values,
/// which are typically ASCII or UTF-8 encoded byte sequences.
///
/// # Type Parameters
///
/// * `'de` - The lifetime of the header value being deserialized
///
/// # Examples
///
/// ```rust
/// use axum::http::HeaderValue;
/// use serde::Deserialize;
///
/// let header_value = HeaderValue::from_static("hello world");
/// let deserializer = HeaderValueDeserializer { value: &header_value };
/// let result: String = Deserialize::deserialize(deserializer)?;
/// assert_eq!(result, "hello world");
/// ```
pub struct HeaderValueDeserializer<'de> {
    /// The header value to deserialize.
    ///
    /// This is a reference to the `HeaderValue` that will be deserialized
    /// into the target Rust type.
    pub value: &'de HeaderValue,
}

impl<'de> Deserializer<'de> for HeaderValueDeserializer<'de> {
    type Error = HeaderValueDeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(HeaderValueDeserializeError::InvalidType)
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(HeaderValueDeserializeError::InvalidType)
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(HeaderValueDeserializeError::InvalidType)
    }

    fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(HeaderValueDeserializeError::InvalidType)
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(HeaderValueDeserializeError::InvalidType)
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(HeaderValueDeserializeError::InvalidType)
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(HeaderValueDeserializeError::InvalidType)
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(HeaderValueDeserializeError::InvalidType)
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(HeaderValueDeserializeError::InvalidType)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(HeaderValueDeserializeError::InvalidType)
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(HeaderValueDeserializeError::InvalidType)
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(HeaderValueDeserializeError::InvalidType)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let str = self
            .value
            .to_str()
            .map_err(HeaderValueDeserializeError::StringifyError)?;

        visitor.visit_borrowed_str(str)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let bytes = self.value.as_bytes();
        visitor.visit_borrowed_bytes(bytes)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.value.is_empty() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        // TODO: Support this for seq of bytes
        Err(HeaderValueDeserializeError::InvalidType)
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(HeaderValueDeserializeError::InvalidType)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(HeaderValueDeserializeError::InvalidType)
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(HeaderValueDeserializeError::InvalidType)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(HeaderValueDeserializeError::InvalidType)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(HeaderValueDeserializeError::InvalidType)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

/// A trait for types that represent optional HTTP headers.
///
/// This trait is used to mark header types that can be optional in HTTP requests.
/// It provides an associated type `Header` that specifies the concrete header type
/// being made optional. This is primarily used by the framework's dependency injection
/// system to handle optional headers in request handlers.
///
/// # Associated Types
///
/// * `Header` - The concrete header type that this optional header represents
///
/// # Examples
///
/// ```rust
/// use headers::ContentType;
///
/// // ContentType implements OptionalHeader
/// fn handle_request(content_type: Option<ContentType>) {
///     // content_type is optional
/// }
/// ```
pub trait OptionalHeader {
    /// The concrete header type that this optional header represents.
    ///
    /// This associated type specifies the underlying header type that can be
    /// optionally present in HTTP requests. It must implement the `Header` trait
    /// from the `headers` crate, ensuring it can be properly parsed from and
    /// serialized to HTTP header values.
    type Header: Header;
}

impl<H: Header> OptionalHeader for Option<H> {
    type Header = H;
}

/// A helper macro to implement the `OptionalHeader` trait for concrete header types.
///
/// This macro generates implementations of `OptionalHeader` for the specified header types,
/// allowing them to be used as optional headers in request handling. Each header type
/// that implements `Header` can be made optional by including it in this macro.
///
/// # Parameters
///
/// * `$header` - A header type that implements the `Header` trait
///
macro_rules! impl_optional_header {
    ($($header:ty),* $(,)?) => {
        $(
            impl OptionalHeader for $header {
                type Header = $header;
            }
        )*
    };
}

impl_optional_header! {
    AcceptRanges,
    AccessControlAllowCredentials,
    AccessControlAllowHeaders,
    AccessControlAllowMethods,
    AccessControlAllowOrigin,
    AccessControlExposeHeaders,
    AccessControlMaxAge,
    AccessControlRequestHeaders,
    AccessControlRequestMethod,
    Age,
    Allow,
    CacheControl,
    Connection,
    ContentDisposition,
    ContentEncoding,
    ContentLength,
    ContentLocation,
    ContentRange,
    ContentType,
    Cookie,
    Date,
    ETag,
    Expect,
    Expires,
    Host,
    IfMatch,
    IfModifiedSince,
    IfNoneMatch,
    IfRange,
    IfUnmodifiedSince,
    LastModified,
    Location,
    Origin,
    Pragma,
    Range,
    Referer,
    ReferrerPolicy,
    RetryAfter,
    SecWebsocketAccept,
    SecWebsocketKey,
    SecWebsocketVersion,
    Server,
    SetCookie,
    StrictTransportSecurity,
    Te,
    TransferEncoding,
    Upgrade,
    UserAgent,
    Vary,
}

impl<C: Credentials> OptionalHeader for Authorization<C> {
    type Header = Self;
}

impl<C: Credentials> OptionalHeader for ProxyAuthorization<C> {
    type Header = Self;
}
