/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// License information for the exposed API.
/// See <https://swagger.io/specification/#license-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LicenseObject {
    /// REQUIRED. The license name used for the API.
    #[serde(rename = "name")]
    pub name: Cow<'static, str>,

    /// An SPDX license expression for the API. The identifier field is mutually exclusive of the url field.
    #[serde(
        rename = "identifier",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub identifier: Option<Cow<'static, str>>,

    /// A URL to the license used for the API. This MUST be in the form of a URL.
    /// The url field is mutually exclusive of the identifier field.
    #[serde(rename = "url", default, skip_serializing_if = "Option::is_none")]
    pub url: Option<Cow<'static, str>>,
}
