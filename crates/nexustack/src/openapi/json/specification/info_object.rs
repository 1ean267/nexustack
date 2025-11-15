/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::{ContactObject, LicenseObject};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// The object provides metadata about the API.
///
/// The metadata MAY be used by the clients if needed, and MAY be presented in editing or
/// documentation generation tools for convenience.
/// See <https://swagger.io/specification/#info-object>
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InfoObject {
    /// REQUIRED. The title of the API.
    #[serde(rename = "title")]
    pub title: Cow<'static, str>,

    /// A short summary of the API.
    #[serde(rename = "summary", default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<Cow<'static, str>>,

    /// A description of the API. `CommonMark` syntax MAY be used for rich text representation.
    #[serde(
        rename = "description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<Cow<'static, str>>,

    /// A URL to the Terms of Service for the API. This MUST be in the form of a URL.
    #[serde(
        rename = "termsOfService",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub terms_of_service: Option<Cow<'static, str>>,

    /// The contact information for the exposed API.
    #[serde(rename = "contact", default, skip_serializing_if = "Option::is_none")]
    pub contact: Option<ContactObject>,

    /// The license information for the exposed API.
    #[serde(rename = "license", default, skip_serializing_if = "Option::is_none")]
    pub license: Option<LicenseObject>,

    /// REQUIRED.
    /// The version of the `OpenAPI` document
    /// (which is distinct from the `OpenAPI` Specification version or the API implementation version).
    #[serde(rename = "version")]
    pub version: Cow<'static, str>,
}
