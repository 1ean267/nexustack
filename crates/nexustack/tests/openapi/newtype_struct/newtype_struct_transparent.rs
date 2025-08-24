/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use nexustack::openapi::api_schema;

/// A struct with optional value
#[api_schema(transparent)]
pub struct P(
    /// The optional value
    Option<i32>,
);

#[test]
fn test_openapi_3_0() {
    use nexustack::openapi::json::{Specification, build_schema};

    let schema = build_schema::<P>(Specification::OpenAPI3_0).unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "example": -2_147_483_648,
            "maximum": 2_147_483_647,
            "minimum": -2_147_483_648,
            "nullable": true,
            "type": "integer"
        })
    );
}

#[test]
fn test_openapi_3_0_with_collection() {
    use nexustack::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
    use std::{cell::RefCell, rc::Rc};

    let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));
    let schema =
        build_schema_with_collection::<P>(Specification::OpenAPI3_0, schema_collection.clone())
            .unwrap();

    let schemas_object = Rc::try_unwrap(schema_collection)
        .map_err(|_| "Should be the only Rc strong reference")
        .unwrap()
        .into_inner()
        .to_schemas_object();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "example": -2_147_483_648,
            "maximum": 2_147_483_647,
            "minimum": -2_147_483_648,
            "nullable": true,
            "type": "integer"
        })
    );

    pretty_assertions::assert_eq!(
        serde_json::to_value(schemas_object).unwrap(),
        serde_json::json!({})
    );
}

#[test]
fn test_openapi_3_1() {
    use nexustack::openapi::json::{Specification, build_schema};

    let schema = build_schema::<P>(Specification::OpenAPI3_1).unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647, null],
            "maximum": 2_147_483_647,
            "minimum": -2_147_483_648,
            "type": ["integer", "null"]
        })
    );
}

#[test]
fn test_openapi_3_1_with_collection() {
    use nexustack::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
    use std::{cell::RefCell, rc::Rc};

    let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));
    let schema =
        build_schema_with_collection::<P>(Specification::OpenAPI3_1, schema_collection.clone())
            .unwrap();

    let schemas_object = Rc::try_unwrap(schema_collection)
        .map_err(|_| "Should be the only Rc strong reference")
        .unwrap()
        .into_inner()
        .to_schemas_object();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647, null],
            "maximum": 2_147_483_647,
            "minimum": -2_147_483_648,
            "type": ["integer", "null"]
        })
    );

    pretty_assertions::assert_eq!(
        serde_json::to_value(schemas_object).unwrap(),
        serde_json::json!({})
    );
}
