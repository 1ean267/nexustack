/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

mod error;
mod http;
mod schema;
mod schema_collection;

pub mod specification;

#[cfg(feature = "http")]
pub use http::{
    KeyedOperationObject, add_http_operation_to_paths, build_http_operation,
    build_http_operation_with_collection,
};
pub use schema::{build_schema, build_schema_with_collection};
pub use schema_collection::SchemaCollection;
