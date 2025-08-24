/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::{
    CallbackOrReferenceObject, ExampleOrReferenceObject, HeaderOrReferenceObject,
    LinkOrReferenceObject, ParameterOrReferenceObject, PathItemOrReferenceObject,
    RequestBodyOrReferenceObject, ResponseOrReferenceObject, SchemaOrReferenceObject,
    SecuritySchemeOrReferenceObject,
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

/// Holds a set of reusable objects for different aspects of the OAS.
///
/// All objects defined within the components object will have no effect on the API
/// unless they are explicitly referenced from properties outside the components object.
/// See <https://swagger.io/specification/#components-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComponentsObject {
    /// An object to hold reusable Schema Objects.
    #[serde(rename = "schemas", default, skip_serializing_if = "Option::is_none")]
    pub schemas: Option<HashMap<Cow<'static, str>, SchemaOrReferenceObject>>, // TODO: Serialize to JSON object

    /// An object to hold reusable Response Objects.
    #[serde(rename = "responses", default, skip_serializing_if = "Option::is_none")]
    pub responses: Option<HashMap<Cow<'static, str>, ResponseOrReferenceObject>>, // TODO: Serialize to JSON object

    /// An object to hold reusable Parameter Objects.
    #[serde(
        rename = "parameters",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub parameters: Option<HashMap<Cow<'static, str>, ParameterOrReferenceObject>>, // TODO: Serialize to JSON object

    /// An object to hold reusable Example Objects.
    #[serde(rename = "examples", default, skip_serializing_if = "Option::is_none")]
    pub examples: Option<HashMap<Cow<'static, str>, ExampleOrReferenceObject>>, // TODO: Serialize to JSON object

    /// An object to hold reusable Request Body Objects.
    #[serde(
        rename = "requestBodies",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub request_bodies: Option<HashMap<Cow<'static, str>, RequestBodyOrReferenceObject>>, // TODO: Serialize to JSON object

    /// An object to hold reusable Header Objects.
    #[serde(rename = "headers", default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<Cow<'static, str>, HeaderOrReferenceObject>>, // TODO: Serialize to JSON object

    /// An object to hold reusable Security Scheme Objects.
    #[serde(
        rename = "securitySchemes",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub security_schemes: Option<HashMap<Cow<'static, str>, SecuritySchemeOrReferenceObject>>, // TODO: Serialize to JSON object

    /// An object to hold reusable Link Objects.
    #[serde(rename = "links", default, skip_serializing_if = "Option::is_none")]
    pub links: Option<HashMap<Cow<'static, str>, LinkOrReferenceObject>>, // TODO: Serialize to JSON object

    /// An object to hold reusable Callback Objects.
    #[serde(rename = "callbacks", default, skip_serializing_if = "Option::is_none")]
    pub callbacks: Option<HashMap<Cow<'static, str>, CallbackOrReferenceObject>>, // TODO: Serialize to JSON object

    /// An object to hold reusable Path Item Object.
    #[serde(rename = "pathItems", default, skip_serializing_if = "Option::is_none")]
    pub path_items: Option<HashMap<Cow<'static, str>, PathItemOrReferenceObject>>, // TODO: Serialize to JSON object
}

// TODO: Model explicitly:
//       All the fixed fields declared above are objects that MUST use keys that match the regular expression: ^[a-zA-Z0-9\.\-_]+$.
