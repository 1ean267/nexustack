/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use serde::{Deserialize, Serialize};

/// A simple object to allow referencing other components in the `OpenAPI` document, internally and externally.
///
/// The $ref string value contains a URI RFC3986, which identifies the location of the value being referenced.
/// See <https://swagger.io/specification/#reference-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReferenceObject {
    /// REQUIRED.
    /// The reference identifier. This MUST be in the form of a URI.
    #[serde(rename = "$ref")]
    pub r#ref: String,

    /// A short summary which by default SHOULD override that of the referenced component.
    /// If the referenced object-type does not allow a summary field, then this field has no effect.
    #[serde(rename = "summary", default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// A description which by default SHOULD override that of the referenced component.
    /// `CommonMark` syntax MAY be used for rich text representation.
    /// If the referenced object-type does not allow a description field, then this field has no effect.
    #[serde(
        rename = "description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
}
