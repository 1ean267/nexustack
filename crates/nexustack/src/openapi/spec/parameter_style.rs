/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use serde::{Deserialize, Serialize};

/// Represents the style of an `OpenAPI` parameter.
///
/// The style determines how a parameter value is serialized in the request.
/// See [OpenAPI Specification](https://spec.openapis.org/oas/v3.1.0#style) for details.
#[derive(Serialize, Debug, Clone)]
pub enum ParameterStyle {
    /// Matrix style, e.g. `;color=blue`.
    #[serde(rename = "matrix")]
    Matrix = 0,
    /// Label style, e.g. `.color=blue`.
    #[serde(rename = "label")]
    Label = 1,
    /// Form style, e.g. `color=blue`.
    #[serde(rename = "form")]
    Form = 2,
    /// Simple style, e.g. `blue,black,red`.
    #[serde(rename = "simple")]
    Simple = 3,
    /// Space delimited style, e.g. `blue%20black%20red`.
    #[serde(rename = "spaceDelimited")]
    SpaceDelimited = 4,
    /// Pipe delimited style, e.g. `blue|black|red`.
    #[serde(rename = "pipeDelimited")]
    PipeDelimited = 5,
    /// Deep object style, e.g. `color[primary]=blue&color[secondary]=black`.
    #[serde(rename = "deepObject")]
    DeepObject = 6,
}

impl<'de> Deserialize<'de> for ParameterStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;

        if s.eq_ignore_ascii_case("matrix") {
            Ok(Self::Matrix)
        } else if s.eq_ignore_ascii_case("label") {
            Ok(Self::Label)
        } else if s.eq_ignore_ascii_case("form") {
            Ok(Self::Form)
        } else if s.eq_ignore_ascii_case("simple") {
            Ok(Self::Simple)
        } else if s.eq_ignore_ascii_case("spaceDelimited") {
            Ok(Self::SpaceDelimited)
        } else if s.eq_ignore_ascii_case("pipeDelimited") {
            Ok(Self::PipeDelimited)
        } else if s.eq_ignore_ascii_case("deepObject") {
            Ok(Self::DeepObject)
        } else {
            Err(serde::de::Error::custom("Unknown parameter style."))
        }
    }
}
