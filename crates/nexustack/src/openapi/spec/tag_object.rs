/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::ExternalDocumentationObject;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Adds metadata to a single tag that is used by the Operation Object.
///
/// It is not mandatory to have a Tag Object per tag defined in the Operation Object instances.
/// See <https://swagger.io/specification/#tag-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TagObject {
    /// REQUIRED. The name of the tag.
    #[serde(rename = "name")]
    pub name: Cow<'static, str>,

    /// A description for the tag. `CommonMark` syntax MAY be used for rich text representation.
    #[serde(
        rename = "description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<Cow<'static, str>>,

    /// Additional external documentation for this tag.
    #[serde(
        rename = "externalDocs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub external_docs: Option<ExternalDocumentationObject>,
}
