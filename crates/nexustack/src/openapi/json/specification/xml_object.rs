/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use serde::{Deserialize, Serialize};
use std::{borrow::Cow, ops::Not};

/// A metadata object that allows for more fine-tuned XML model definitions.
///
/// When using arrays, XML element names are not inferred (for singular/plural forms) and the name property SHOULD
/// be used to add that information. See examples for expected behavior.
/// See <https://swagger.io/specification/#xml-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct XmlObject {
    /// Replaces the name of the element/attribute used for the described schema property.
    /// When defined within items, it will affect the name of the individual XML elements within the list.
    /// When defined alongside type being array (outside the items), it will affect the wrapping element and
    /// only if wrapped is true. If wrapped is false, it will be ignored.
    #[serde(rename = "name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Cow<'static, str>>,

    /// The URI of the namespace definition. This MUST be in the form of an absolute URI.
    #[serde(rename = "namespace", default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<Cow<'static, str>>,

    /// The prefix to be used for the name.
    #[serde(rename = "prefix", default, skip_serializing_if = "Option::is_none")]
    pub prefix: Option<Cow<'static, str>>,

    /// Declares whether the property definition translates to an attribute instead of an element.
    /// Default value is false.
    #[serde(rename = "attribute", default, skip_serializing_if = "<&bool>::not")]
    pub attribute: bool,

    /// MAY be used only for an array definition.
    /// Signifies whether the array is wrapped (for example, <books><book/><book/></books>) or unwrapped (<book/><book/>).
    /// Default value is false.
    /// The definition takes effect only when defined alongside type being array (outside the items).
    #[serde(rename = "wrapped", default, skip_serializing_if = "<&bool>::not")]
    pub wrapped: bool,
}
