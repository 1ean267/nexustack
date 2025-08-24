/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use serde::{Deserialize, Serialize};

/// Represents the location of a security scheme in an `OpenAPI` specification.
///
/// This enum is used to specify where the security credentials are expected to be found
/// in the HTTP request.
#[derive(Serialize, Debug, Clone)]
pub enum SecuritySchemeLocation {
    /// Credentials are passed as query parameters.
    #[serde(rename = "query")]
    Query,
    /// Credentials are passed in HTTP headers.
    #[serde(rename = "header")]
    Header,
    /// Credentials are passed in cookies.
    #[serde(rename = "cookie")]
    Cookie,
}

impl<'de> Deserialize<'de> for SecuritySchemeLocation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;

        if s.eq_ignore_ascii_case("query") {
            Ok(Self::Query)
        } else if s.eq_ignore_ascii_case("header") {
            Ok(Self::Header)
        } else if s.eq_ignore_ascii_case("cookie") {
            Ok(Self::Cookie)
        } else {
            Err(serde::de::Error::custom(
                "Unknown security scheme location.",
            ))
        }
    }
}
