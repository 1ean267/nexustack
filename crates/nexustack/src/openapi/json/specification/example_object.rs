/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::ReferenceObject;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Represents an `OpenAPI` Example Object.
///
/// See <https://swagger.io/specification/#example-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ExampleObject {
    /// Example object with an embedded value.
    Value {
        /// Short description for the example.
        #[serde(rename = "summary", default, skip_serializing_if = "Option::is_none")]
        summary: Option<String>,

        /// Long description for the example.
        /// `CommonMark` syntax MAY be used for rich text representation.
        #[serde(
            rename = "description",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        description: Option<String>,

        /// Embedded literal example.
        /// The value field and externalValue field are mutually exclusive.
        /// To represent examples of media types that cannot naturally represented in JSON or YAML,
        /// use a string value to contain the example, escaping where necessary.
        #[serde(rename = "value", default, skip_serializing_if = "Option::is_none")]
        value: Option<JsonValue>,
    },
    /// Example object referencing an external value.
    ExternalValue {
        /// Short description for the example.
        #[serde(rename = "summary", default, skip_serializing_if = "Option::is_none")]
        summary: Option<String>,

        /// Long description for the example.
        /// `CommonMark` syntax MAY be used for rich text representation.
        #[serde(
            rename = "description",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        description: Option<String>,

        /// A URI that points to the literal example.
        /// This provides the capability to reference examples that cannot easily be included in JSON or YAML documents.
        /// The value field and externalValue field are mutually exclusive.
        /// See the rules for resolving Relative References.
        #[serde(
            rename = "externalValue",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        external_value: Option<String>,
    },
}

/// Represents either an [`ExampleObject`] or [`ReferenceObject`] object.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ExampleOrReferenceObject {
    /// An inline example object.
    Example(ExampleObject),
    /// A reference to an example object.
    Reference(ReferenceObject),
}
