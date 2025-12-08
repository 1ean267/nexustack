/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/// Represents the supported `OpenAPI` specification versions.
///
/// This enum is used to distinguish between different versions of the `OpenAPI` specification.
/// Currently, only `OpenAPI` 3.0 an`OpenAPI` 3.1 are supported.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum SpecificationVersion {
    /// `OpenAPI` Specification version 3.0
    OpenAPI3_0,
    /// `OpenAPI` Specification version 3.1
    OpenAPI3_1,
}

impl std::fmt::Display for SpecificationVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAPI3_0 => f.write_str("Open API 3.0"),
            Self::OpenAPI3_1 => f.write_str("Open API 3.1"),
        }
    }
}
