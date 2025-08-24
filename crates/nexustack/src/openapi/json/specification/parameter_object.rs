/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::{
    BoxSchemaOrReferenceObject, ExampleOrReferenceObject, MediaTypeObject, ParameterLocation,
    ParameterStyle, ReferenceObject,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{collections::HashMap, ops::Not};

/// Describes a single operation parameter.
///
/// A unique parameter is defined by a combination of a name and location.
/// Parameter Locations
///
/// There are four possible parameter locations specified by the in field:
///
/// * path - Used together with Path Templating, where the parameter value is actually part of the operation's URL. This does not include the host or base path of the API. For example, in /items/{itemId}, the path parameter is itemId.
/// * query - Parameters that are appended to the URL. For example, in /items?id=###, the query parameter is id.
/// * header - Custom headers that are expected as part of the request. Note that RFC7230 states header names are case insensitive.
/// * cookie - Used to pass a specific cookie value to the API.
// See https://swagger.io/specification/#parameter-object
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ParameterObject {
    /// The rules for serialization of the parameter are specified in one of two ways.
    /// For simpler scenarios, a schema and style can describe the structure and syntax of the parameter.
    Schema {
        /// REQUIRED. The name of the parameter. Parameter names are case sensitive.
        /// * If in is "path",
        ///   the name field MUST correspond to a template expression occurring within the path field in the Paths Object.
        ///   See Path Templating for further information.
        /// * If in is "header" and the name field is "Accept", "Content-Type" or "Authorization",
        ///   the parameter definition SHALL be ignored.
        /// * For all other cases, the name corresponds to the parameter name used by the in property.
        #[serde(rename = "name")]
        name: String,

        /// REQUIRED.
        /// The location of the parameter.
        /// Possible values are "query", "header", "path" or "cookie".
        #[serde(rename = "in")]
        r#in: ParameterLocation,

        /// A brief description of the parameter. This could contain examples of use.
        /// `CommonMark` syntax MAY be used for rich text representation.
        #[serde(
            rename = "description",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        description: Option<String>,

        /// Determines whether this parameter is mandatory.
        /// If the parameter location is "path", this property is REQUIRED and its value MUST be true.
        /// Otherwise, the property MAY be included and its default value is false.
        #[serde(rename = "required", default, skip_serializing_if = "<&bool>::not")]
        required: bool,

        /// Specifies that a parameter is deprecated and SHOULD be transitioned out of usage.
        /// Default value is false.
        #[serde(rename = "deprecated", default, skip_serializing_if = "<&bool>::not")]
        deprecated: bool,

        /// Sets the ability to pass empty-valued parameters.
        /// This is valid only for query parameters and allows sending a parameter with an empty value.
        /// Default value is false
        /// If style is used, and if behavior is n/a (cannot be serialized), the value of allowEmptyValue SHALL be ignored.
        /// Use of this property is NOT RECOMMENDED, as it is likely to be removed in a later revision.
        #[serde(
            rename = "allowEmptyValue",
            default,
            skip_serializing_if = "<&bool>::not"
        )]
        allow_empty_value: bool,

        /// Describes how the parameter value will be serialized depending on the type of the parameter value.
        /// Default values (based on value of in):  
        /// * for query: form
        /// * for path: simple
        /// * for header: simple
        /// * for cookie: form
        #[serde(rename = "style", default, skip_serializing_if = "Option::is_none")]
        style: Option<ParameterStyle>,

        /// When this is true, parameter values of type array or object generate separate parameters for each value of the array
        /// or key-value pair of the map.
        /// For other types of parameters this property has no effect.
        /// When style is form, the default value is true.
        /// For all other styles, the default value is false.
        #[serde(rename = "explode", default, skip_serializing_if = "Option::is_none")]
        explode: Option<bool>,

        /// Determines whether the parameter value SHOULD allow reserved characters,
        /// as defined by RFC3986 :/?#[]@!$&'()*+,;= to be included without percent-encoding.
        /// This property only applies to parameters with an in value of query.
        /// The default value is false.
        #[serde(
            rename = "allowReserved",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        allow_reserved: Option<bool>,

        /// The schema defining the type used for the parameter.
        #[serde(rename = "schema", default, skip_serializing_if = "Option::is_none")]
        schema: Option<BoxSchemaOrReferenceObject>,

        /// Example of the parameter's potential value.
        /// The example SHOULD match the specified schema and encoding properties if present.
        /// The example field is mutually exclusive of the examples field.
        /// Furthermore, if referencing a schema that contains an example, the example value SHALL
        /// override the example provided by the schema.
        /// To represent examples of media types that cannot naturally be represented in JSON or YAML,
        /// a string value can contain the example with escaping where necessary.
        #[serde(rename = "example", default, skip_serializing_if = "Option::is_none")]
        example: Option<JsonValue>,

        /// Examples of the parameter's potential value.
        /// Each example SHOULD contain a value in the correct format as specified in the parameter encoding.
        /// The examples field is mutually exclusive of the example field.
        /// Furthermore, if referencing a schema that contains an example, the examples value SHALL override
        /// the example provided by the schema.
        #[serde(rename = "examples", default, skip_serializing_if = "Option::is_none")]
        examples: Option<HashMap<String, ExampleOrReferenceObject>>,
    },
    /// For more complex scenarios, the content property can define the media type and schema of the parameter.
    /// A parameter MUST contain either a schema property, or a content property, but not both.
    /// When example or examples are provided in conjunction with the schema object, the example MUST follow the
    /// prescribed serialization strategy for the parameter.
    Content {
        /// REQUIRED. The name of the parameter. Parameter names are case sensitive.
        /// * If in is "path",
        ///   the name field MUST correspond to a template expression occurring within the path field in the Paths Object.
        ///   See Path Templating for further information.
        /// * If in is "header" and the name field is "Accept", "Content-Type" or "Authorization",
        ///   the parameter definition SHALL be ignored.
        /// * For all other cases, the name corresponds to the parameter name used by the in property.
        #[serde(rename = "name")]
        name: String,

        /// REQUIRED.
        /// The location of the parameter.
        /// Possible values are "query", "header", "path" or "cookie".
        #[serde(rename = "in")]
        r#in: ParameterLocation,

        /// A brief description of the parameter. This could contain examples of use.
        /// `CommonMark` syntax MAY be used for rich text representation.
        #[serde(
            rename = "description",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        description: Option<String>,

        /// Determines whether this parameter is mandatory.
        /// If the parameter location is "path", this property is REQUIRED and its value MUST be true.
        /// Otherwise, the property MAY be included and its default value is false.
        #[serde(rename = "required", default, skip_serializing_if = "<&bool>::not")]
        required: bool,

        /// Specifies that a parameter is deprecated and SHOULD be transitioned out of usage.
        /// Default value is false.
        #[serde(rename = "deprecated", default, skip_serializing_if = "<&bool>::not")]
        deprecated: bool,

        /// Sets the ability to pass empty-valued parameters.
        /// This is valid only for query parameters and allows sending a parameter with an empty value.
        /// Default value is false
        ///  If style is used, and if behavior is n/a (cannot be serialized), the value of allowEmptyValue SHALL be ignored.
        /// Use of this property is NOT RECOMMENDED, as it is likely to be removed in a later revision.
        #[serde(
            rename = "allowEmptyValue",
            default,
            skip_serializing_if = "<&bool>::not"
        )]
        allow_empty_value: bool,

        /// A map containing the representations for the parameter.
        /// The key is the media type and the value describes it.
        /// The map MUST only contain one entry.
        #[serde(rename = "content", default, skip_serializing_if = "Option::is_none")]
        content: Option<HashMap<String, MediaTypeObject>>,
    },
}

/// Represents either an [`ParameterObject`] or [`ReferenceObject`] object.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ParameterOrReferenceObject {
    /// An inline parameter object.
    Parameter(ParameterObject),
    /// A reference to a parameter object.
    Reference(ReferenceObject),
}
