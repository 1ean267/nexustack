/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use serde::{Deserialize, Serialize};

/// An object representing a Server Variable for server URL template substitution.
/// See <https://swagger.io/specification/#server-variable-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerVariableObject {
    /// An enumeration of string values to be used if the substitution options are from a limited set.
    /// The array MUST NOT be empty.
    #[serde(rename = "enum", default, skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<String>>, // TODO: Guarantee: The array MUST NOT be empty.

    /// REQUIRED.
    /// The default value to use for substitution, which SHALL be sent if an alternate value is not supplied.
    /// Note this behavior is different than the Schema Object's treatment of default values,
    /// because in those cases parameter values are optional.
    /// If the enum is defined, the value MUST exist in the enum's values.
    #[serde(rename = "default")]
    pub default: String, // TODO: Guarantee: If the enum is defined, the value MUST exist in the enum's values.

    /// An optional description for the server variable. `CommonMark` syntax MAY be used for rich text representation.
    #[serde(
        rename = "description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
}
