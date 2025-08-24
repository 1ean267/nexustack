/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::{HeaderOrReferenceObject, LinkOrReferenceObject, MediaTypeObject, ReferenceObject};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Describes a single response from an API Operation, including design-time, static links to operations based on the response.
/// See <https://swagger.io/specification/#response-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseObject {
    /// REQUIRED. A description of the response.
    /// `CommonMark` syntax MAY be used for rich text representation.
    #[serde(rename = "description")]
    pub description: String,

    /// Maps a header name to its definition. RFC7230 states header names are case insensitive.
    /// If a response header is defined with the name "Content-Type", it SHALL be ignored.
    #[serde(rename = "headers", default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, HeaderOrReferenceObject>>,

    /// A map containing descriptions of potential response payloads.
    /// The key is a media type or media type range and the value describes it.
    /// For responses that match multiple keys, only the most specific key is applicable.
    /// e.g. text/plain overrides text/*
    #[serde(rename = "content", default, skip_serializing_if = "Option::is_none")]
    pub content: Option<HashMap<String, MediaTypeObject>>,

    /// A map of operations links that can be followed from the response.
    /// The key of the map is a short name for the link, following the naming constraints
    /// of the names for Component Objects.
    #[serde(rename = "links", default, skip_serializing_if = "Option::is_none")]
    pub links: Option<HashMap<String, LinkOrReferenceObject>>,
}

/// Represents either an [`ResponseObject`] or [`ReferenceObject`] object.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ResponseOrReferenceObject {
    /// An inline response object.
    Response(ResponseObject),
    /// A reference to a response object.
    Reference(ReferenceObject),
}
