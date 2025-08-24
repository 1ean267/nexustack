/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::{EncodingObject, ExampleOrReferenceObject, SchemaOrReferenceObject};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Each Media Type Object provides schema and examples for the media type identified by its key.
/// See <https://swagger.io/specification/#media-type-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MediaTypeObject {
    /// The schema defining the content of the request, response, or parameter.
    #[serde(rename = "schema", default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<SchemaOrReferenceObject>,

    /// Example of the media type.
    /// The example object SHOULD be in the correct format as specified by the media type.
    /// The example field is mutually exclusive of the examples field.
    /// Furthermore, if referencing a schema which contains an example,
    /// the example value SHALL override the example provided by the schema.
    #[serde(rename = "example", default, skip_serializing_if = "Option::is_none")]
    pub example: Option<JsonValue>,

    /// Examples of the media type.
    /// Each example object SHOULD match the media type and specified schema if present.
    /// The examples field is mutually exclusive of the example field.
    /// Furthermore, if referencing a schema which contains an example,
    /// the examples value SHALL override the example provided by the schema.
    #[serde(rename = "examples", default, skip_serializing_if = "Option::is_none")]
    pub examples: Option<HashMap<String, ExampleOrReferenceObject>>,

    /// A map between a property name and its encoding information.
    /// The key, being the property name, MUST exist in the schema as a property.
    /// The encoding object SHALL only apply to requestBody objects when the media type is
    /// multipart or application/x-www-form-urlencoded.
    #[serde(rename = "encoding", default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<HashMap<String, EncodingObject>>,
}
