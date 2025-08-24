/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::{MediaTypeObject, ReferenceObject};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::Not};

/// Describes a single request body.
/// See <https://swagger.io/specification/#request-body-object>

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestBodyObject {
    /// A brief description of the request body. This could contain examples of use.
    /// `CommonMark` syntax MAY be used for rich text representation.
    #[serde(
        rename = "description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,

    /// REQUIRED.
    /// The content of the request body.
    /// The key is a media type or media type range and the value describes it.
    /// For requests that match multiple keys, only the most specific key is applicable.
    /// e.g. text/plain overrides text/*
    #[serde(rename = "content")]
    pub content: HashMap<String, MediaTypeObject>,

    /// Determines if the request body is required in the request. Defaults to false.
    #[serde(rename = "required", default, skip_serializing_if = "<&bool>::not")]
    pub required: bool,
}

/// Represents either an [`RequestBodyObject`] or [`ReferenceObject`] object.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RequestBodyOrReferenceObject {
    /// An inline request body object.
    RequestBody(RequestBodyObject),
    /// A reference to a request body object.
    Reference(ReferenceObject),
}
