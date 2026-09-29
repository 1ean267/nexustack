/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    http::headers::{
        AcceptRanges, AccessControlAllowCredentials, AccessControlAllowHeaders,
        AccessControlAllowMethods, AccessControlAllowOrigin, AccessControlExposeHeaders,
        AccessControlMaxAge, AccessControlRequestHeaders, AccessControlRequestMethod, Age, Allow,
        CacheControl, Connection, ContentDisposition, ContentEncoding, ContentLength,
        ContentLocation, ContentRange, ContentType, Cookie, Date, ETag, Origin, UserAgent, Vary,
    },
    openapi::Schema,
};
use std::ops::Bound;

// TODO: Stolen from net.rs
const U16_REGEX: &str =
    r"((6553[0-5])|(655[0-2]\d)|(65[0-4]\d{2})|(6[0-4]\d{3})|([0-5]\d{4})|(\d{1,4}))";

impl Schema for AcceptRanges {
    type Example = &'static str;
    type Examples = <[Self::Example; 2] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            None,
            None,
            Some(&["none", "bytes"]),
            Some("An HTTP Accept-Ranges response header value."),
            || Ok(["none", "bytes"]),
            false,
        )
    }
}

impl Schema for AccessControlAllowCredentials {
    type Example = &'static str;
    type Examples = <[Self::Example; 1] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            None,
            None,
            Some(&["true"]),
            Some("An HTTP Access-Control-Allow-Credentials response header value."),
            || Ok(["true"]),
            false,
        )
    }
}

impl Schema for AccessControlAllowHeaders {
    type Example = &'static str;
    type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            Some(r"^(\*)|([^,\s]+(,\s?[^,\s]+)*)$"),
            None,
            None,
            Some("An HTTP Access-Control-Allow-Headers response header value."),
            || Ok(["Accept-Encoding", "Accept-Language, Content-Type", "*"]),
            false,
        )
    }
}

impl Schema for AccessControlAllowMethods {
    type Example = &'static str;
    type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            Some(r"^(\*)|(((GET)|(POST)|(PUT)|(DELETE)|(PATCH)|(OPTIONS)|(HEAD)|(TRACE))(,\s?((GET)|(POST)|(PUT)|(DELETE)|(PATCH)|(OPTIONS)|(HEAD)|(TRACE)))*)$"),
            None,
            None,
            Some("An HTTP Access-Control-Allow-Methods response header value."),
            || Ok(["PUT", "PUT, DELETE", "*"]),
            false,
        )
    }
}

impl Schema for AccessControlAllowOrigin {
    type Example = &'static str;
    type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            Some(const_format::formatcp!(
                r"^(null)|(\*)|(([^:\/?#\r\n\s]+):\/\/[^\/?#:\r\n\s]+(:{U16_REGEX})?)$"
            )),
            None,
            None,
            Some("An HTTP Access-Control-Allow-Origin response header value."),
            || Ok(["https://developer.mozilla.org", "null", "*"]),
            false,
        )
    }
}

impl Schema for AccessControlExposeHeaders {
    type Example = &'static str;
    type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            Some(r"^(\*)|([^,\s]+(,\s?[^,\s]+)*)?|$"),
            None,
            None,
            Some("An HTTP Access-Control-Expose-Headers response header value."),
            || Ok(["Content-Encoding", "Content-Encoding, Kuma-Revision", "*"]),
            false,
        )
    }
}

impl Schema for AccessControlMaxAge {
    type Example = u64;
    type Examples = <[Self::Example; 1] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_u64(
            Bound::Unbounded,
            Bound::Unbounded,
            None,
            None,
            None,
            Some("An HTTP Access-Control-Max-Age response header value."),
            || Ok([600]),
            false,
        )
    }
}

impl Schema for AccessControlRequestHeaders {
    type Example = &'static str;
    type Examples = <[Self::Example; 1] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            Some(r"^([^,]+(,[^,]+)*)$"),
            None,
            None,
            Some("An HTTP Access-Control-Request-Headers request header value."),
            || Ok(["content-type,x-pingother"]),
            false,
        )
    }
}

impl Schema for AccessControlRequestMethod {
    type Example = &'static str;
    type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            Some(r"^((GET)|(POST)|(PUT)|(DELETE)|(PATCH)|(OPTIONS)|(HEAD)|(TRACE))$"),
            None,
            None,
            Some("An HTTP Access-Control-Request-Method request header value."),
            || Ok(["GET", "POST", "DELETE"]),
            false,
        )
    }
}

impl Schema for Age {
    type Example = u64;
    type Examples = <[Self::Example; 1] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_u64(
            Bound::Unbounded,
            Bound::Unbounded,
            None,
            None,
            None,
            Some("An HTTP Age response header value."),
            || Ok([24]),
            false,
        )
    }
}

impl Schema for Allow {
    type Example = &'static str;
    type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            Some(r"^(((GET)|(POST)|(PUT)|(DELETE)|(PATCH)|(OPTIONS)|(HEAD)|(TRACE))(,\s?((GET)|(POST)|(PUT)|(DELETE)|(PATCH)|(OPTIONS)|(HEAD)|(TRACE)))*)$"),
            None,
            None,
            Some("An HTTP Allow response header value."),
            || Ok(["GET, POST, HEAD", "POST, HEAD", "HEAD"]),
            false,
        )
    }
}

// TODO: Pattern
impl Schema for CacheControl {
    type Example = &'static str;
    type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            None,
            None,
            None,
            Some("An HTTP CacheControl request or response header value."),
            || Ok(["no-cache", "private, community=\"UCI\"", "max-age=30"]),
            false,
        )
    }
}

// TODO: Pattern
impl Schema for Connection {
    type Example = &'static str;
    type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            None,
            None,
            None,
            Some("An HTTP Connection request or response header value."),
            || Ok(["close", "keep-alive", "upgrade"]),
            false,
        )
    }
}

// TODO: Pattern
impl Schema for ContentDisposition {
    type Example = &'static str;
    type Examples = <[Self::Example; 2] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            None,
            None,
            None,
            Some("An HTTP ContentDisposition request or response header value."),
            || {
                Ok([
                    "form-data; name=\"field1\"",
                    "form-data; name=\"field2\"; filename=\"example.txt\"",
                ])
            },
            false,
        )
    }
}

impl Schema for ContentEncoding {
    type Example = &'static str;
    type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            Some(r"^(((gzip)|(compress)|(deflate)|(br)|(zstd)|(dcb)|(dcz))(,\s?((gzip)|(compress)|(deflate)|(br)|(zstd)|(dcb)|(dcz)))*)$"),
            None,
            None,
            Some("An HTTP Content-Encoding header value."),
            || Ok(["deflate, gzip", "br", "dcb"]),
            false,
        )
    }
}

impl Schema for ContentLength {
    type Example = u64;
    type Examples = <[Self::Example; 1] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_u64(
            Bound::Unbounded,
            Bound::Unbounded,
            None,
            None,
            None,
            Some("An HTTP Content-Length header value."),
            || Ok([3495]),
            false,
        )
    }
}

impl Schema for ContentLocation {
    type Example = &'static str;
    type Examples = <[Self::Example; 2] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            Some(r"^((([^:\/?#\r\n\s]+):(\/\/([^\/?#\r\n\s]*))?)?([^?#\r\n\s]*)(\?([^#\r\n\s]*))?(#([^\r\n\s]*))?)$"),
            Some("uri"),
            None,
            Some("An HTTP Content-Location header value."),
            || Ok(["/hypertext/Overview.html", "http://www.example.org/hypertext/Overview.html"]),
            false,
        )
    }
}

impl Schema for ContentRange {
    type Example = &'static str;
    type Examples = <[Self::Example; 2] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            Some(r"^bytes (\*|([0-9]+-[0-9]+))\/(\*|[0-9]+)$"),
            None,
            None,
            Some("An HTTP Content-Range header value."),
            || Ok(["bytes 0-1023/146515", "bytes */67589"]),
            false,
        )
    }
}

impl Schema for ContentType {
    type Example = <mime::Mime as Schema>::Example;
    type Examples = <mime::Mime as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        <mime::Mime as Schema>::describe(schema_builder)
    }
}

impl Schema for Cookie {
    type Example = &'static str;
    type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            Some(r"^\w+=\w+(; \w+=\w+)*$"),
            None,
            None,
            Some("An HTTP Cookie header value."),
            || {
                Ok([
                    "SID=31d4d96e407aad42",
                    "SID=31d4d96e407aad42; lang=en-US",
                    "PHPSESSID=298zf09hf012fh2; csrftoken=u32t4o3tb3gg43; _gat=1",
                ])
            },
            false,
        )
    }
}

impl Schema for Date {
    type Example = &'static str;
    type Examples = <[Self::Example; 2] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            Some(r"^(((Mon)|(Tue)|(Wed)|(Thu)|(Fri)|(Sat)|(Sun)), [0-9]{2} ((Jan)|(Feb)|(Mar)|(Apr)|(May)|(Jun)|(Jul)|(Aug)|(Sep)|(Oct)|(Nov)|(Dec)) [0-9]{4} [0-9]{2}:[0-9]{2}:[0-9]{2} GMT)$"),
            None,
            None,
            Some("An HTTP Date header value."),
            || {
                Ok([
                    "Tue, 29 Oct 2024 16:56:32 GMT",
                    "Tue, 15 Nov 1994 08:12:31 GMT",
                ])
            },
            false,
        )
    }
}

impl Schema for ETag {
    type Example = &'static str;
    type Examples = <[Self::Example; 2] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            Some(r#"^(W\/)?"[ -~]+"$"#),
            None,
            None,
            Some("An HTTP ETag header value."),
            || Ok(["\"33a64df551425fcc55e4d42a148795d9f25f89d4\"", "W/\"0815\""]),
            false,
        )
    }
}

// Expect
// Expires
// Host
// IfMatch
// IfModifiedSince
// IfNoneMatch
// IfRange
// IfUnmodifiedSince
// LastModified
// Location

impl Schema for Origin {
    type Example = &'static str;
    type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            Some(const_format::formatcp!(
                r"^(([^:\/?#\r\n\s]+):\/\/[^\/?#:\r\n\s]+(:{U16_REGEX})?)$"
            )),
            None,
            None,
            Some("An HTTP Origin request header value."),
            || {
                Ok([
                    "null",
                    "https://developer.mozilla.org",
                    "https://developer.mozilla.org:80",
                ])
            },
            false,
        )
    }
}

// Pragma
// Range
// Referer
// ReferrerPolicy
// RetryAfter
// SecWebsocketAccept
// SecWebsocketKey
// SecWebsocketVersion
// Server
// SetCookie
// StrictTransportSecurity
// Te
// TransferEncoding
// Upgrade

impl Schema for UserAgent {
    type Example = &'static str;
    type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            Some(r"\((?<info>.*?)\)(\s|$)|(?<name>.*?)\/(?<version>.*?)(\s|$)"),
            None,
            None,
            Some("An HTTP User-Agent request header value."),
            || {
                Ok([
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3",
                    "curl/7.64.1",
                    "PostmanRuntime/7.26.8",
                ])
            },
            false,
        )
    }
}

impl Schema for Vary {
    type Example = &'static str;
    type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            Some(r"^(\*)|([^,\s]+(,\s?[^,\s]+)*)$"),
            None,
            None,
            Some("An HTTP Vary response header value."),
            || Ok(["Accept-Encoding", "Accept-Language, Content-Type", "*"]),
            false,
        )
    }
}
