/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::ReferenceObject;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::borrow::Cow;

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
        summary: Option<Cow<'static, str>>,

        /// Long description for the example.
        /// `CommonMark` syntax MAY be used for rich text representation.
        #[serde(
            rename = "description",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        description: Option<Cow<'static, str>>,

        /// Embedded literal example.
        /// The value field and externalValue field are mutually exclusive.
        /// To represent examples of media types that cannot naturally represented in JSON or YAML,
        /// use a Cow<'static, str> value to contain the example, escaping where necessary.
        #[serde(rename = "value")]
        value: JsonValue,
    },
    /// Example object with an embedded value.
    SerializedValue {
        /// Short description for the example.
        #[serde(rename = "summary", default, skip_serializing_if = "Option::is_none")]
        summary: Option<Cow<'static, str>>,

        /// Long description for the example.
        /// `CommonMark` syntax MAY be used for rich text representation.
        #[serde(
            rename = "description",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        description: Option<Cow<'static, str>>,

        /// An example of the data structure that MUST be valid according to the relevant Schema
        /// Object. If this field is present, value MUST be absent.
        #[serde(rename = "dataValue")]
        data_value: JsonValue,

        /// An example of the serialized form of the value, including encoding and escaping as
        /// described under Validating Examples.
        /// If dataValue is present, then this field SHOULD contain the serialization of the given data.
        /// Otherwise, it SHOULD be the valid serialization of a data value that itself MUST be
        /// valid as described for dataValue.
        /// This field SHOULD NOT be used if the serialization format is JSON, as the data form is easier
        /// to work with.
        /// If this field is present, value, and externalValue MUST be absent.
        #[serde(rename = "serializedValue")]
        serialized_value: Cow<'static, str>,
    },
    /// Example object referencing an external value.
    ExternalValue {
        /// Short description for the example.
        #[serde(rename = "summary", default, skip_serializing_if = "Option::is_none")]
        summary: Option<Cow<'static, str>>,

        /// Long description for the example.
        /// `CommonMark` syntax MAY be used for rich text representation.
        #[serde(
            rename = "description",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        description: Option<Cow<'static, str>>,

        /// A URI that points to the literal example.
        /// This provides the capability to reference examples that cannot easily be included in JSON or YAML documents.
        /// The value field and externalValue field are mutually exclusive.
        /// See the rules for resolving Relative References.
        #[serde(rename = "externalValue")]
        external_value: Cow<'static, str>,
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

impl From<ExampleObject> for ExampleOrReferenceObject {
    fn from(value: ExampleObject) -> Self {
        Self::Example(value)
    }
}

impl From<ReferenceObject> for ExampleOrReferenceObject {
    fn from(value: ReferenceObject) -> Self {
        Self::Reference(value)
    }
}
