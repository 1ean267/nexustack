/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::{OperationObject, ParameterOrReferenceObject, ReferenceObject, ServerObject};
use serde::{Deserialize, Serialize};

/// Describes the operations available on a single path.
///
/// A Path Item MAY be empty, due to ACL constraints.
/// The path itself is still exposed to the documentation viewer but they will not know which operations and parameters are available.
/// See <https://swagger.io/specification/#path-item-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PathItemObject {
    /// Allows for a referenced definition of this path item.
    /// The referenced structure MUST be in the form of a Path Item Object.
    /// In case a Path Item Object field appears both in the defined object and the referenced object,
    /// the behavior is undefined. See the rules for resolving Relative References.
    #[serde(rename = "$ref", default, skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,

    /// An optional, string summary, intended to apply to all operations in this path.
    #[serde(rename = "summary", default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// An optional, string description, intended to apply to all operations in this path.
    /// `CommonMark` syntax MAY be used for rich text representation.
    #[serde(
        rename = "description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,

    /// A definition of a GET operation on this path.
    #[serde(rename = "get", default, skip_serializing_if = "Option::is_none")]
    pub get: Option<Box<OperationObject>>,

    /// A definition of a PUT operation on this path.
    #[serde(rename = "put", default, skip_serializing_if = "Option::is_none")]
    pub put: Option<Box<OperationObject>>,

    /// A definition of a POST operation on this path.
    #[serde(rename = "post", default, skip_serializing_if = "Option::is_none")]
    pub post: Option<Box<OperationObject>>,

    /// A definition of a DELETE operation on this path.
    #[serde(rename = "delete", default, skip_serializing_if = "Option::is_none")]
    pub delete: Option<Box<OperationObject>>,

    /// A definition of a OPTIONS operation on this path.
    #[serde(rename = "options", default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Box<OperationObject>>,

    /// A definition of a HEAD operation on this path.
    #[serde(rename = "head", default, skip_serializing_if = "Option::is_none")]
    pub head: Option<Box<OperationObject>>,

    /// A definition of a PATCH operation on this path.
    #[serde(rename = "patch", default, skip_serializing_if = "Option::is_none")]
    pub patch: Option<Box<OperationObject>>,

    /// A definition of a TRACE operation on this path.
    #[serde(rename = "trace", default, skip_serializing_if = "Option::is_none")]
    pub trace: Option<Box<OperationObject>>,

    /// An alternative server array to service all operations in this path.
    #[serde(rename = "servers", default, skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<ServerObject>>,

    /// A list of parameters that are applicable for all the operations described under this path.
    /// These parameters can be overridden at the operation level, but cannot be removed there.
    /// The list MUST NOT include duplicated parameters.
    /// A unique parameter is defined by a combination of a name and location.
    /// The list can use the Reference Object to link to parameters that are defined at the
    /// `OpenAPI` Object's components/parameters.
    #[serde(
        rename = "parameters",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub parameters: Option<Vec<ParameterOrReferenceObject>>,
}

/// Represents either an [`PathItemObject`] or [`ReferenceObject`] object.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PathItemOrReferenceObject {
    /// An inline path-item object.
    PathItem(PathItemObject),
    /// A reference to a path-item object.
    Reference(ReferenceObject),
}
