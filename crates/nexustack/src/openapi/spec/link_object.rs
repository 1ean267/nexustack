/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::{ReferenceObject, ServerObject};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{borrow::Cow, collections::HashMap};

/// The Link object represents a possible design-time link for a response.
///
/// The presence of a link does not guarantee
/// the caller's ability to successfully invoke it, rather it provides a known relationship and traversal mechanism
/// between responses and other operations.
/// Unlike dynamic links (i.e. links provided in the response payload), the OAS linking mechanism does not require link
/// information in the runtime response.
/// For computing links, and providing instructions to execute them, a runtime expression is used for accessing values
/// in an operation and using them as parameters while invoking the linked operation.
/// See <https://swagger.io/specification/#link-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkObject {
    /// A relative or absolute URI reference to an OAS operation.
    /// This field is mutually exclusive of the operationId field, and MUST point to an Operation Object.
    /// Relative operationRef values MAY be used to locate an existing Operation Object in the `OpenAPI` definition.
    /// See the rules for resolving Relative References.
    #[serde(
        rename = "operationRef",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub operation_ref: Option<Cow<'static, str>>,

    /// The name of an existing, resolvable OAS operation, as defined with a unique operationId.
    /// This field is mutually exclusive of the operationRef field.
    #[serde(
        rename = "operationId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub operation_id: Option<Cow<'static, str>>,

    /// A map representing parameters to pass to an operation as specified with operationId or identified via operationRef.
    /// The key is the parameter name to be used, whereas the value can be a constant or an expression to be evaluated and
    /// passed to the linked operation. The parameter name can be qualified using the parameter location [{in}.]{name} for
    /// operations that use the same parameter name in different locations (e.g. path.id).
    #[serde(
        rename = "parameters",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub parameters: Option<HashMap<Cow<'static, str>, JsonValue>>,
    /// A literal value or {expression} to use as a request body when calling the target operation.
    #[serde(
        rename = "requestBody",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub request_body: Option<JsonValue>,
    /// A description of the link. `CommonMark` syntax MAY be used for rich text representation.
    #[serde(
        rename = "description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<Cow<'static, str>>,
    /// A server object to be used by the target operation.
    #[serde(rename = "server", default, skip_serializing_if = "Option::is_none")]
    pub server: Option<ServerObject>,
}

/// Represents either a Link object or a Reference object in the `OpenAPI` specification.
///
/// This enum is used to allow for fields that can be either a direct Link definition or a reference to one.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum LinkOrReferenceObject {
    /// A direct Link object.
    Link(LinkObject),
    /// A Reference object pointing to a Link.
    Reference(ReferenceObject),
}
