/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::{
    ComponentsObject, ExternalDocumentationObject, InfoObject, PathItemOrReferenceObject,
    PathsObject, SecurityRequirements, ServerObject, TagObject,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

/// This is the root object of the `OpenAPI` document.
/// See <https://swagger.io/specification/#openapi-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenAPIObject {
    /// REQUIRED.
    /// This Cow<'static, str> MUST be the version number of the `OpenAPI` Specification that th`OpenAPI`PI document uses.
    /// The openapi field SHOULD be used by tooling to interpret the `OpenAPI` document.
    /// This is not related to the API info.version Cow<'static, str>.
    #[serde(rename = "openapi")]
    pub openapi: Cow<'static, str>,

    /// REQUIRED.
    /// Provides metadata about the API. The metadata MAY be used by tooling as required.
    #[serde(rename = "info")]
    pub info: InfoObject,

    /// The default value for the $schema keyword within Schema Objects contained within this OAS document.
    /// This MUST be in the form of a URI.
    #[serde(
        rename = "jsonSchemaDialect",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub json_schema_dialect: Option<Cow<'static, str>>,

    /// An array of Server Objects, which provide connectivity information to a target server.
    /// If the servers property is not provided, or is an empty array, the default value would be a Server Object with a url value of /.
    #[serde(rename = "servers", default, skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<ServerObject>>,

    /// The available paths and operations for the API.
    #[serde(rename = "paths")]
    pub paths: PathsObject,

    /// The incoming webhooks that MAY be received as part of this API and that the API consumer MAY choose to implement.
    /// Closely related to the callbacks feature, this section describes requests initiated other than by an API call,
    /// for example by an out of band registration. The key name is a unique Cow<'static, str> to refer to each webhook,
    /// while the (optionally referenced) Path Item Object describes a request that may be initiated by the API provider
    /// and the expected responses.
    #[serde(rename = "webhooks", default, skip_serializing_if = "Option::is_none")]
    pub webhooks: Option<HashMap<Cow<'static, str>, PathItemOrReferenceObject>>, // TODO: Serialize to JSON object

    /// An element to hold various schemas for the document.
    #[serde(
        rename = "components",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub components: Option<ComponentsObject>,

    /// A declaration of which security mechanisms can be used across the API.
    /// The list of values includes alternative security requirement objects that can be used.
    /// Only one of the security requirement objects need to be satisfied to authorize a request.
    /// Individual operations can override this definition.
    /// To make security optional, an empty security requirement ({}) can be included in the array.
    #[serde(rename = "security", default, skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityRequirements>,

    /// A list of tags used by the document with additional metadata.
    /// The order of the tags can be used to reflect on their order by the parsing tools.
    /// Not all tags that are used by the Operation Object must be declared.
    /// The tags that are not declared MAY be organized randomly or based on the tools' logic.
    /// Each tag name in the list MUST be unique.
    #[serde(rename = "tags", default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<TagObject>>,

    /// Additional external documentation.
    #[serde(
        rename = "externalDocs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub external_docs: Option<ExternalDocumentationObject>,
}
