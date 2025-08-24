/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use std::collections::HashMap;

use super::{HeaderOrReferenceObject, ParameterStyle};
use serde::{Deserialize, Serialize};

/// A single encoding definition applied to a single schema property.
/// See <https://swagger.io/specification/#encoding-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncodingObject {
    /// The Content-Type for encoding a specific property.
    /// Default value depends on the property type:
    /// * for object: application/json
    /// * for array: the default is defined based on the inner type
    /// * for all other cases the default is application/octet-stream
    ///   
    /// The value can be a specific media type (e.g. application/json),
    /// a wildcard media type (e.g. image/*), or a comma-separated list of the two types.
    #[serde(
        rename = "contentType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub content_type: Option<String>,

    /// A map allowing additional information to be provided as headers, for example Content-Disposition.
    /// Content-Type is described separately and SHALL be ignored in this section.
    /// This property SHALL be ignored if the request body media type is not a multipart.
    #[serde(rename = "headers", default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, HeaderOrReferenceObject>>,

    /// Describes how a specific property value will be serialized depending on its type.
    /// See Parameter Object for details on the style property.
    /// The behavior follows the same values as query parameters, including default values.
    /// This property SHALL be ignored if the request body media type is not
    /// application/x-www-form-urlencoded or multipart/form-data.
    /// If a value is explicitly defined, then the value of contentType (implicit or explicit) SHALL be ignored.
    #[serde(rename = "style", default, skip_serializing_if = "Option::is_none")]
    pub style: Option<ParameterStyle>,

    /// When this is true, property values of type array or object generate separate parameters for each value
    /// of the array, or key-value-pair of the map.
    /// For other types of properties this property has no effect.
    /// When style is form, the default value is true.
    /// For all other styles, the default value is false.
    /// This property SHALL be ignored if the request body media type is not application/x-www-form-urlencoded
    /// or multipart/form-data. If a value is explicitly defined, then the value of contentType (implicit or explicit)
    ///  SHALL be ignored.
    #[serde(rename = "explode", default, skip_serializing_if = "Option::is_none")]
    pub explode: Option<bool>,

    /// Determines whether the parameter value SHOULD allow reserved characters, as defined by
    /// RFC3986 :/?#[]@!$&'()*+,;= to be included without percent-encoding.
    /// The default value is false.
    /// This property SHALL be ignored if the request body media type is not application/x-www-form-urlencoded
    /// or multipart/form-data. If a value is explicitly defined, then the value of contentType (implicit or explicit)
    /// SHALL be ignored.
    #[serde(
        rename = "allowReserved",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_reserved: Option<bool>,
}
