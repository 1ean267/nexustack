/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::{
    BoxSchemaOrReferenceObject, ExampleOrReferenceObject, MediaTypeObject, ReferenceObject,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{borrow::Cow, collections::HashMap};

/// Represents the style of an `OpenAPI` header.
///
/// The style determines how a header value is serialized in the request.
/// See [OpenAPI Specification](https://spec.openapis.org/oas/v3.1.0#style) for details.
#[derive(Serialize, Debug, Clone)]
pub enum HeaderStyle {
    /// Simple style, e.g. `blue,black,red`.
    #[serde(rename = "simple")]
    Simple = 0,
}

impl<'de> Deserialize<'de> for HeaderStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;

        if s.eq_ignore_ascii_case("simple") {
            Ok(Self::Simple)
        } else {
            Err(serde::de::Error::custom("Unknown header style."))
        }
    }
}

/// The Header Object follows the structure of the Parameter Object with the following changes:
/// * name MUST NOT be specified, it is given in the corresponding headers map.
/// * in MUST NOT be specified, it is implicitly in header.
/// * All traits that are affected by the location MUST be applicable to a location of header (for example, style).
///
/// See <https://swagger.io/specification/#header-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum HeaderObject {
    /// The rules for serialization of the header are specified in one of two ways.
    /// For simpler scenarios, a schema and style can describe the structure and syntax of the header.
    Schema {
        /// A brief description of the header. This could contain examples of use.
        /// `CommonMark` syntax MAY be used for rich text representation.
        #[serde(
            rename = "description",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        description: Option<Cow<'static, str>>,

        /// Determines whether this header is mandatory.
        /// If the header location is "path", this property is REQUIRED and its value MUST be true.
        /// Otherwise, the property MAY be included and its default value is false.
        #[serde(rename = "required", default)]
        required: bool,

        /// Specifies that a header is deprecated and SHOULD be transitioned out of usage.
        /// Default value is false.
        #[serde(rename = "deprecated", default)]
        deprecated: bool,

        /// Describes how the header value will be serialized depending on the type of the header value.
        /// Default values (based on value of in):
        /// * for query: form
        /// * for path: simple
        /// * for header: simple
        /// * for cookie: form
        #[serde(rename = "style", default, skip_serializing_if = "Option::is_none")]
        style: Option<HeaderStyle>,

        /// When this is true, header values of type array or object generate separate headers for each value of the array
        /// or key-value pair of the map.
        /// For other types of headers this property has no effect.
        /// When style is form, the default value is true.
        /// For all other styles, the default value is false.
        #[serde(rename = "explode", default, skip_serializing_if = "Option::is_none")]
        explode: Option<bool>,

        /// The schema defining the type used for the header.
        #[serde(rename = "schema")]
        schema: BoxSchemaOrReferenceObject,

        /// Example of the header's potential value.
        /// The example SHOULD match the specified schema and encoding properties if present.
        /// The example field is mutually exclusive of the examples field.
        /// Furthermore, if referencing a schema that contains an example, the example value SHALL
        /// override the example provided by the schema.
        /// To represent examples of media types that cannot naturally be represented in JSON or YAML,
        /// a Cow<'static, str> value can contain the example with escaping where necessary.
        #[serde(rename = "example", default, skip_serializing_if = "Option::is_none")]
        example: Option<JsonValue>,

        /// Examples of the header's potential value.
        /// Each example SHOULD contain a value in the correct format as specified in the header encoding.
        /// The examples field is mutually exclusive of the example field.
        /// Furthermore, if referencing a schema that contains an example, the examples value SHALL override
        /// the example provided by the schema.
        #[serde(
            rename = "examples",
            default,
            skip_serializing_if = "HashMap::is_empty"
        )]
        examples: HashMap<Cow<'static, str>, ExampleOrReferenceObject>,
    },
    /// For more complex scenarios, the content property can define the media type and schema of the header.
    /// A header MUST contain either a schema property, or a content property, but not both.
    /// When example or examples are provided in conjunction with the schema object, the example MUST follow the
    /// prescribed serialization strategy for the header.
    Content {
        /// A brief description of the header. This could contain examples of use.
        /// `CommonMark` syntax MAY be used for rich text representation.
        #[serde(
            rename = "description",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        description: Option<Cow<'static, str>>,

        /// Determines whether this header is mandatory.
        /// If the header location is "path", this property is REQUIRED and its value MUST be true.
        /// Otherwise, the property MAY be included and its default value is false.
        #[serde(rename = "required", default)]
        required: bool,

        /// Specifies that a header is deprecated and SHOULD be transitioned out of usage.
        /// Default value is false.
        #[serde(rename = "deprecated", default)]
        deprecated: bool,

        /// A map containing the representations for the header.
        /// The key is the media type and the value describes it.
        /// The map MUST only contain one entry.
        #[serde(rename = "content")]
        content: HashMap<Cow<'static, str>, MediaTypeObject>,
    },
}

/// Represents either a [`HeaderObject`] or [`ReferenceObject`].
/// Used to allow referencing headers via `$ref` in `OpenAPI`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum HeaderOrReferenceObject {
    /// A Header definition.
    Header(HeaderObject),
    /// A Reference to a header definition.
    Reference(ReferenceObject),
}

impl From<HeaderObject> for HeaderOrReferenceObject {
    fn from(value: HeaderObject) -> Self {
        Self::Header(value)
    }
}

impl From<ReferenceObject> for HeaderOrReferenceObject {
    fn from(value: ReferenceObject) -> Self {
        Self::Reference(value)
    }
}
