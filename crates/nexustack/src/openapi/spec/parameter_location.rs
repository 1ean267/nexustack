/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Represents the location of an `OpenAPI` parameter.
///
/// This enum corresponds to the possible values for the `in` field of a parameter object
/// in the `OpenAPI` specification. It indicates where the parameter is expected to be found.
///
#[derive(Serialize, Debug, Clone)]
pub enum ParameterLocation {
    /// Parameter is located in the query Cow<'static, str>.
    #[serde(rename = "query")]
    Query,
    /// Parameter is located in the request header.
    #[serde(rename = "header")]
    Header,
    /// Parameter is located in the path.
    #[serde(rename = "path")]
    Path,
    /// Parameter is located in a cookie.
    #[serde(rename = "cookie")]
    Cookie,
}

impl<'de> Deserialize<'de> for ParameterLocation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: Cow<'static, str> = Deserialize::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "query" => Ok(Self::Query),
            "header" => Ok(Self::Header),
            "path" => Ok(Self::Path),
            "cookie" => Ok(Self::Cookie),
            _ => Err(serde::de::Error::custom(
                "Unknown security scheme location.",
            )),
        }
    }
}
