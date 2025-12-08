/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::PathItemObject;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

/// Holds the relative paths to the individual endpoints and their operations.
///
/// The path is appended to the URL from the Server Object in order to construct the full URL.
/// The Paths MAY be empty, due to Access Control List (ACL) constraints.
/// See <https://swagger.io/specification/#paths-object>
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PathsObject(pub HashMap<Cow<'static, str>, PathItemObject>);
