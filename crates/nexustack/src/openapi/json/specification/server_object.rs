/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::ServerVariableObject;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

/// An object representing a Server.
/// See <https://swagger.io/specification/#server-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerObject {
    /// REQUIRED.
    /// A URL to the target host.
    /// This URL supports Server Variables and MAY be relative, to indicate that the host location is
    /// relative to the location where the `OpenAPI` document is being served.
    /// Variable substitutions will be made when a variable is named in {brackets}.
    #[serde(rename = "url")]
    pub url: Cow<'static, str>,

    /// An optional Cow<'static, str> describing the host designated by the URL.
    /// `CommonMark` syntax MAY be used for rich text representation.
    #[serde(
        rename = "description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<Cow<'static, str>>,

    /// A map between a variable name and its value. The value is used for substitution in the server's URL template.
    #[serde(rename = "variables", default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<Cow<'static, str>, ServerVariableObject>>, // TODO: Serialize to JSON object
}
