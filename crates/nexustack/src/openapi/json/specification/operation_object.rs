/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::{
    CallbackOrReferenceObject, ExternalDocumentationObject, ParameterOrReferenceObject,
    RequestBodyOrReferenceObject, ResponseObject, ServerObject,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::Not};

/// Describes a single API operation on a path.
/// See <https://swagger.io/specification/#operation-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OperationObject {
    /// A list of tags for API documentation control.
    /// Tags can be used for logical grouping of operations by resources or any other qualifier.
    #[serde(rename = "tags", default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// A short summary of what the operation does.
    #[serde(rename = "summary", default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// A verbose explanation of the operation behavior.
    /// `CommonMark` syntax MAY be used for rich text representation.
    #[serde(
        rename = "description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,

    /// Additional external documentation for this operation.
    #[serde(
        rename = "externalDocs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub external_docs: Option<ExternalDocumentationObject>,

    /// Unique string used to identify the operation.
    /// The id MUST be unique among all operations described in the API.
    /// The operationId value is case-sensitive.
    /// Tools and libraries MAY use the operationId to uniquely identify an operation, therefore,
    /// it is RECOMMENDED to follow common programming naming conventions.
    #[serde(
        rename = "operationId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub operation_id: Option<String>,

    /// A list of parameters that are applicable for this operation.
    /// If a parameter is already defined at the Path Item, the new definition will override it
    /// but can never remove it. The list MUST NOT include duplicated parameters.
    /// A unique parameter is defined by a combination of a name and location.
    /// The list can use the Reference Object to link to parameters that are defined at the
    /// `OpenAPI` Object's components/parameters.
    #[serde(
        rename = "parameters",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub parameters: Option<Vec<ParameterOrReferenceObject>>,

    /// The request body applicable for this operation.
    /// The requestBody is fully supported in HTTP methods where the HTTP 1.1 specification RFC7231
    /// has explicitly defined semantics for request bodies.
    /// In other cases where the HTTP spec is vague (such as GET, HEAD and DELETE),
    /// requestBody is permitted but does not have well-defined semantics and SHOULD be avoided if possible.
    #[serde(
        rename = "requestBody",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub request_body: Option<RequestBodyOrReferenceObject>,

    /// The list of possible responses as they are returned from executing this operation.
    #[serde(rename = "responses")]
    pub responses: HashMap<String, ResponseObject>,

    /// A map of possible out-of band callbacks related to the parent operation.
    /// The key is a unique identifier for the Callback Object.
    /// Each value in the map is a Callback Object that describes a request that may be initiated
    /// by the API provider and the expected responses.
    #[serde(rename = "callbacks", default, skip_serializing_if = "Option::is_none")]
    pub callbacks: Option<HashMap<String, CallbackOrReferenceObject>>,

    /// Declares this operation to be deprecated.
    /// Consumers SHOULD refrain from usage of the declared operation.
    /// Default value is false.
    #[serde(rename = "deprecated", default, skip_serializing_if = "<&bool>::not")]
    pub deprecated: bool,

    /// A declaration of which security mechanisms can be used for this operation.
    /// The list of values includes alternative security requirement objects that can be used.
    /// Only one of the security requirement objects need to be satisfied to authorize a request.
    /// To make security optional, an empty security requirement ({}) can be included in the array.
    /// This definition overrides any declared top-level security.
    /// To remove a top-level security declaration, an empty array can be used.
    #[serde(rename = "security", default, skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<HashMap<String, Vec<String>>>>,

    /// An alternative server array to service this operation.
    /// If an alternative server object is specified at the Path Item Object or Root level,
    /// it will be overridden by this value.
    #[serde(rename = "servers", default, skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<ServerObject>>,
}
