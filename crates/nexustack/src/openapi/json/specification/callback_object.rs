/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::{PathItemObject, ReferenceObject};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A map of possible out-of band callbacks related to the parent operation.
///
/// Each value in the map is a Path Item Object that describes a set of requests that may be initiated
/// by the API provider and the expected responses. The key value used to identify the path item object
/// is an expression, evaluated at runtime, that identifies a URL to use for the callback operation.
/// See <https://swagger.io/specification/#callback-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CallbackObject(HashMap<String, PathItemObject>);

/// Represents either a [`CallbackObject`] or [`ReferenceObject`].
/// Used to allow references to callback objects in `OpenAPI` specifications.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum CallbackOrReferenceObject {
    /// A direct callback object.
    Callback(CallbackObject),
    /// A reference to a callback object.
    Reference(ReferenceObject),
}
