/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

/// Configuration details for a supported OAuth Flow
/// See <https://swagger.io/specification/#oauth-flow-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OAuthFlowObject {
    /// REQUIRED.
    /// The authorization URL to be used for this flow.
    /// This MUST be in the form of a URL. The `OAuth2` standard requires the use of TLS.
    #[serde(
        rename = "authorizationUrl",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub authorization_url: Option<Cow<'static, str>>,

    /// REQUIRED.
    /// The token URL to be used for this flow. This MUST be in the form of a URL.
    /// The `OAuth2` standard requires the use of TLS.
    #[serde(rename = "tokenUrl", default, skip_serializing_if = "Option::is_none")]
    pub token_url: Option<Cow<'static, str>>,

    /// The URL to be used for obtaining refresh tokens.
    /// This MUST be in the form of a URL. The `OAuth2` standard requires the use of TLS.
    #[serde(
        rename = "refreshUrl",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub refresh_url: Option<Cow<'static, str>>,

    /// REQUIRED. The available scopes for the `OAuth2` security scheme.A map between the scope name and a short description for it. The map MAY be empty.
    #[serde(rename = "scopes")]
    pub scopes: HashMap<Cow<'static, str>, Cow<'static, str>>,
}
