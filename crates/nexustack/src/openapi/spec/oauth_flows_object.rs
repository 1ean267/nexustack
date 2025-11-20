/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::OAuthFlowObject;
use serde::{Deserialize, Serialize};

/// Configuration details for a supported OAuth Flow
/// See <https://swagger.io/specification/#oauth-flows-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OAuthFlowsObject {
    /// Configuration for the OAuth Implicit flow
    #[serde(rename = "implicit", default, skip_serializing_if = "Option::is_none")]
    pub implicit: Option<OAuthFlowObject>,

    /// Configuration for the OAuth Resource Owner Password flow
    #[serde(rename = "password", default, skip_serializing_if = "Option::is_none")]
    pub password: Option<OAuthFlowObject>,

    /// Configuration for the OAuth Client Credentials flow. Previously called application in `OpenAPI` 2.0.
    #[serde(
        rename = "clientCredentials",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub client_credentials: Option<OAuthFlowObject>,

    /// Configuration for the OAuth Authorization Code flow. Previously called accessCode in `OpenAPI` 2.0.
    #[serde(
        rename = "authorizationCode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub authorization_code: Option<OAuthFlowObject>,
}
