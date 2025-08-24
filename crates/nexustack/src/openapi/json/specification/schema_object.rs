/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::{DiscriminatorObject, ExternalDocumentationObject, ReferenceObject, XmlObject};
use serde::{Deserialize, Serialize};
use serde_json::{Number as JsonNumber, Value as JsonValue};
use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap},
};

/// The Schema Object allows the definition of input and output data types.
//
// This struct is a superset that combines properties from both `OpenAPI` 3.0 and `OpenAPI` 3.1 specifications.
// Some fields are only meaningful in one version or the other, and are marked accordingly.
//
// - OpenAPI 3.0: Based on JSON Schema Draft 4, with some extensions.
// - OpenAPI 3.1: Based on JSON Schema Draft 2020-12, with full compatibility.
//
// See <https://swagger.io/specification/#schema-object> and <https://spec.`OpenAPI`s.org/oas/v3.1.0#schema-object>.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SchemaObject {
    /// Specifies if the value can be null.
    ///
    /// **`OpenAPI` 3.0 only**. In `OpenAPI` 3.1, use `type: ["null", ...]`.
    #[serde(rename = "nullable", default, skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,

    /// Adds support for polymorphism.
    /// The discriminator is an object name that is used to differentiate between other schemas which may satisfy
    /// the payload description. See Composition and Inheritance for more details.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(
        rename = "discriminator",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub discriminator: Option<DiscriminatorObject>,

    /// Indicates that the property is read-only.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "readOnly", default, skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,

    /// Indicates that the property is write-only.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "writeOnly", default, skip_serializing_if = "Option::is_none")]
    pub write_only: Option<bool>,

    /// Adds additional metadata to describe the XML representation of this property.
    /// Only used on property schemas.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "xml", default, skip_serializing_if = "Option::is_none")]
    pub xml: Option<XmlObject>,

    /// Additional external documentation for this schema.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(
        rename = "externalDocs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub external_docs: Option<ExternalDocumentationObject>,

    /// A free-form property to include an example of an instance for this schema.
    /// To represent examples that cannot be naturally represented in JSON or YAML,
    /// a string value can be used to contain the example with escaping where necessary.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "example", default, skip_serializing_if = "Option::is_none")]
    pub example: Option<JsonValue>,

    /// Multiple examples for the schema.
    ///
    /// **`OpenAPI` 3.1 only**. In `OpenAPI` 3.0, only the `example` property is supported.
    #[serde(rename = "examples", default, skip_serializing_if = "Option::is_none")]
    pub examples: Option<Examples>,

    /// Marks the schema as deprecated.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(
        rename = "deprecated",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub deprecated: Option<bool>,

    /// The type(s) of the schema (e.g., "string", "object", "array").
    ///
    /// **`OpenAPI` 3.0:** Single string value.
    /// **`OpenAPI` 3.1:** Can be a string or array of strings (JSON Schema 2020-12).
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<OneOrMany<Cow<'static, str>>>,

    /// Combines schemas using allOf.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "allOf", default, skip_serializing_if = "Option::is_none")]
    pub all_of: Option<Vec<BoxSchemaOrReferenceObject>>,

    /// Combines schemas using oneOf.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "oneOf", default, skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<BoxSchemaOrReferenceObject>>,

    /// Combines schemas using anyOf.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "anyOf", default, skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<BoxSchemaOrReferenceObject>>,

    /// Negates a schema.
    ///
    /// **`OpenAPI` 3.1 only**. Not supported in `OpenAPI` 3.0.
    #[serde(rename = "not", default, skip_serializing_if = "Option::is_none")]
    pub not: Option<BoxSchemaOrReferenceObject>,

    /// Describes the type of items in an array.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "items", default, skip_serializing_if = "Option::is_none")]
    pub items: Option<BoxSchemaOrReferenceObject>,

    /// Describes the types of items at specific positions in an array.
    ///
    /// **`OpenAPI` 3.1 only**. Not supported in `OpenAPI` 3.0.
    #[serde(
        rename = "prefixItems",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prefix_items: Option<Vec<BoxSchemaOrReferenceObject>>,

    /// Properties defined for an object type.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(
        rename = "properties",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub properties: Option<HashMap<Cow<'static, str>, BoxSchemaOrReferenceObject>>,

    /// Defines additional properties allowed in an object.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(
        rename = "additionalProperties",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_properties: Option<AdditionalProperties>,

    /// Defines pattern-based properties for an object.
    ///
    /// **`OpenAPI` 3.1 only**. Not supported in `OpenAPI` 3.0.
    #[serde(
        rename = "patternProperties",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub pattern_properties: Option<HashMap<Cow<'static, str>, BoxSchemaOrReferenceObject>>,

    /// A description of the schema.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(
        rename = "description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<Cow<'static, str>>,

    /// The format of the schema value (e.g., "date-time", "email").
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "format", default, skip_serializing_if = "Option::is_none")]
    pub format: Option<Cow<'static, str>>,

    /// The default value for the schema.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "default", default, skip_serializing_if = "Option::is_none")]
    pub default: Option<JsonValue>,

    /// The title of the schema.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "title", default, skip_serializing_if = "Option::is_none")]
    pub title: Option<Cow<'static, str>>,

    /// The value must be a multiple of this number.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(
        rename = "multipleOf",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub multiple_of: Option<JsonNumber>,

    /// The maximum value allowed.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "maximum", default, skip_serializing_if = "Option::is_none")]
    pub maximum: Option<JsonNumber>,

    /// Whether the maximum is exclusive, or the exclusive maximum value.
    ///
    /// **`OpenAPI` 3.0:** Boolean.
    /// **`OpenAPI` 3.1:** Numeric value.
    #[serde(
        rename = "exclusiveMaximum",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub exclusive_maximum: Option<EitherUntagged<bool, JsonNumber>>,

    /// The minimum value allowed.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "minimum", default, skip_serializing_if = "Option::is_none")]
    pub minimum: Option<JsonNumber>,

    /// Whether the minimum is exclusive, or the exclusive minimum value.
    ///
    /// **`OpenAPI` 3.0:** Boolean.
    /// **`OpenAPI` 3.1:** Numeric value.
    #[serde(
        rename = "exclusiveMinimum",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub exclusive_minimum: Option<EitherUntagged<bool, JsonNumber>>,

    /// The maximum length of a string.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "maxLength", default, skip_serializing_if = "Option::is_none")]
    pub max_length: Option<JsonNumber>,

    /// The minimum length of a string.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "minLength", default, skip_serializing_if = "Option::is_none")]
    pub min_length: Option<JsonNumber>,

    /// A regular expression pattern for a string value.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "pattern", default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<Cow<'static, str>>,

    /// The maximum number of items in an array.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "maxItems", default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<JsonNumber>,

    /// The minimum number of items in an array.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "minItems", default, skip_serializing_if = "Option::is_none")]
    pub min_items: Option<JsonNumber>,

    /// Whether array items must be unique.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(
        rename = "uniqueItems",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub unique_items: Option<bool>,

    /// The maximum number of properties in an object.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(
        rename = "maxProperties",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub max_properties: Option<JsonNumber>,

    /// The minimum number of properties in an object.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(
        rename = "minProperties",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub min_properties: Option<JsonNumber>,

    /// The required properties for an object.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "required", default, skip_serializing_if = "Option::is_none")]
    pub required: Option<BTreeSet<Cow<'static, str>>>,

    /// The allowed values for the schema.
    ///
    /// **`OpenAPI` 3.0 and 3.1**
    #[serde(rename = "enum", default, skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<JsonValue>>,
}

/// Represents multiple examples for a schema.
///
/// **`OpenAPI` 3.1 only**. Not supported in `OpenAPI` 3.0.
///
/// The `examples` property in `OpenAPI` 3.1 allows you to specify multiple examples for a schema.
/// It can be either:
/// - a vector of example values (`Vec` variant), or
/// - a map of named examples (`Map` variant).
///
/// # `Map` variant
/// The `Map` variant corresponds to the `OpenAPI` 3.1 feature where `examples` is an object whose keys are example names
/// and whose values are the example data. This allows you to provide multiple named examples for a schema property,
/// each with its own identifier. This is useful for documenting different valid values or scenarios for a schema property.
///
/// See [`OpenAPI` 3.1 Specification: Schema Object - examples](https://spec.`OpenAPI`s.org/oas/v3.1.0#schema-object)
/// > The `examples` property can be an object where each property name is the example name and the value is the example.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Examples {
    /// Examples as a vector of values.
    Vec(Vec<JsonValue>),
    /// Examples as a map from names to values (`OpenAPI` 3.1 object form).
    Map(HashMap<Cow<'static, str>, JsonValue>),
}

/// Represents the value of `additionalProperties` in a schema.
///
/// **`OpenAPI` 3.0 and 3.1**
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum AdditionalProperties {
    /// Additional properties are described by a schema.
    Schema(Box<SchemaObject>),
    /// Additional properties are described by a reference.
    Reference(ReferenceObject),
    /// Additional properties are allowed or disallowed by a boolean.
    Boolean(bool),
}

impl From<SchemaObject> for AdditionalProperties {
    fn from(value: SchemaObject) -> Self {
        Self::Schema(Box::new(value))
    }
}

impl From<Box<SchemaObject>> for AdditionalProperties {
    fn from(value: Box<SchemaObject>) -> Self {
        Self::Schema(value)
    }
}

impl From<ReferenceObject> for AdditionalProperties {
    fn from(value: ReferenceObject) -> Self {
        Self::Reference(value)
    }
}

impl From<SchemaOrReferenceObject> for AdditionalProperties {
    fn from(value: SchemaOrReferenceObject) -> Self {
        match value {
            SchemaOrReferenceObject::Schema(schema_object) => Self::Schema(Box::new(schema_object)),
            SchemaOrReferenceObject::Reference(reference_object) => {
                Self::Reference(reference_object)
            }
        }
    }
}

impl From<BoxSchemaOrReferenceObject> for AdditionalProperties {
    fn from(value: BoxSchemaOrReferenceObject) -> Self {
        match value {
            BoxSchemaOrReferenceObject::Schema(schema_object) => Self::Schema(schema_object),
            BoxSchemaOrReferenceObject::Reference(reference_object) => {
                Self::Reference(reference_object)
            }
        }
    }
}

impl From<bool> for AdditionalProperties {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

/// Represents either a boxed [`SchemaObject`] or a [`ReferenceObject`].
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum BoxSchemaOrReferenceObject {
    /// A boxed inline schema object.
    Schema(Box<SchemaObject>),
    /// A reference to a schema object.
    Reference(ReferenceObject),
}

impl From<SchemaObject> for BoxSchemaOrReferenceObject {
    fn from(value: SchemaObject) -> Self {
        Self::Schema(Box::new(value))
    }
}

impl From<Box<SchemaObject>> for BoxSchemaOrReferenceObject {
    fn from(value: Box<SchemaObject>) -> Self {
        Self::Schema(value)
    }
}

impl From<ReferenceObject> for BoxSchemaOrReferenceObject {
    fn from(value: ReferenceObject) -> Self {
        Self::Reference(value)
    }
}

impl From<SchemaOrReferenceObject> for BoxSchemaOrReferenceObject {
    fn from(value: SchemaOrReferenceObject) -> Self {
        match value {
            SchemaOrReferenceObject::Schema(schema_object) => Self::Schema(Box::new(schema_object)),
            SchemaOrReferenceObject::Reference(reference_object) => {
                Self::Reference(reference_object)
            }
        }
    }
}

/// Represents either a [`SchemaObject`] or a [`ReferenceObject`].
#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SchemaOrReferenceObject {
    /// A schema object.
    Schema(SchemaObject),
    /// A reference object.
    Reference(ReferenceObject),
}

impl From<SchemaObject> for SchemaOrReferenceObject {
    fn from(value: SchemaObject) -> Self {
        Self::Schema(value)
    }
}

impl From<ReferenceObject> for SchemaOrReferenceObject {
    fn from(value: ReferenceObject) -> Self {
        Self::Reference(value)
    }
}

/// Represents either a single value or multiple values.
///
/// **`OpenAPI` 3.1 only**. In `OpenAPI` 3.0, only single string values are allowed for `type`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    /// A single value.
    One(T),
    /// Multiple values.
    Many(Vec<T>),
}

impl From<Cow<'static, str>> for OneOrMany<Cow<'static, str>> {
    fn from(value: Cow<'static, str>) -> Self {
        Self::One(value)
    }
}

impl From<&'static str> for OneOrMany<Cow<'static, str>> {
    fn from(value: &'static str) -> Self {
        Self::One(value.into())
    }
}

impl From<Vec<Cow<'static, str>>> for OneOrMany<Cow<'static, str>> {
    fn from(value: Vec<Cow<'static, str>>) -> Self {
        Self::Many(value)
    }
}

/// Represents either a left or right value, for untagged enums.
///
/// Used for properties like `exclusiveMaximum` and `exclusiveMinimum` that differ between `OpenAPI` 3.0 (bool) and 3.1 (number).
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum EitherUntagged<L, R> {
    /// The left value.
    Left(L),
    /// The right value.
    Right(R),
}
