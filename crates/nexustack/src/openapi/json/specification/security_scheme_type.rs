/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use serde::{Deserialize, Serialize};

/// Represents the type of security scheme used in `OpenAPI` specification.

#[derive(Serialize, Debug, Clone)]
pub enum SecuritySchemeType {
    /// API key authentication.
    #[serde(rename = "apiKey")]
    ApiKey,
    /// HTTP authentication schemes (e.g., Basic, Bearer).
    #[serde(rename = "http")]
    Http,
    /// OAuth 2.0 authentication.
    #[serde(rename = "oauth2")]
    Oauth2,
    /// `OpenID Connect` authentication.
    #[serde(rename = "openIdConnect")]
    OpenIdConnect,
}

impl<'de> Deserialize<'de> for SecuritySchemeType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;

        if s.eq_ignore_ascii_case("apiKey") {
            Ok(Self::ApiKey)
        } else if s.eq_ignore_ascii_case("http") {
            Ok(Self::Http)
        } else if s.eq_ignore_ascii_case("oauth2") {
            Ok(Self::Oauth2)
        } else if s.eq_ignore_ascii_case("openIdConnect") {
            Ok(Self::OpenIdConnect)
        } else {
            Err(serde::de::Error::custom("Unknown security scheme type."))
        }
    }
}
