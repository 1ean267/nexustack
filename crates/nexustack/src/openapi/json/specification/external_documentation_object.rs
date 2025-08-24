/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use serde::{Deserialize, Serialize};

// Allows referencing an external resource for extended documentation.
/// See <https://swagger.io/specification/#external-documentation-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExternalDocumentationObject {
    /// A description of the target documentation.
    /// `CommonMark` syntax MAY be used for rich text representation.
    #[serde(
        rename = "description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,

    /// REQUIRED.
    /// The URL for the target documentation. This MUST be in the form of a URL.
    #[serde(rename = "url")]
    pub url: String,
}
