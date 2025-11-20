/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Contact information for the exposed API.
/// See <https://swagger.io/specification/#contact-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContactObject {
    /// The identifying name of the contact person/organization.
    #[serde(rename = "name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Cow<'static, str>>,

    /// The URL pointing to the contact information. This MUST be in the form of a URL.
    #[serde(rename = "url", default, skip_serializing_if = "Option::is_none")]
    pub url: Option<Cow<'static, str>>,

    /// The email address of the contact person/organization. This MUST be in the form of an email address.
    #[serde(rename = "email", default, skip_serializing_if = "Option::is_none")]
    pub email: Option<Cow<'static, str>>,
}
