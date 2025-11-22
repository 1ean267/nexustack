/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

/// Represents discriminator information.
///
/// When request bodies or response payloads may be one of a number of different schemas, a discriminator object
/// can be used to aid in serialization, deserialization, and validation. The discriminator is a specific object
/// in a schema which is used to inform the consumer of the document of an alternative schema based on the value
/// associated with it.
/// When using the discriminator, inline schemas will not be considered.
/// See <https://swagger.io/specification/#discriminator-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiscriminatorObject {
    /// REQUIRED. The name of the property in the payload that will hold the discriminator value.
    #[serde(rename = "propertyName")]
    pub property_name: Cow<'static, str>,

    /// An object to hold mappings between payload values and schema names or references.
    #[serde(rename = "mapping", default, skip_serializing_if = "Option::is_none")]
    pub mapping: Option<HashMap<Cow<'static, str>, Cow<'static, str>>>,
}
