/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use nexustack::openapi::api_schema;

/// A unit struct
#[api_schema(rename = "MyUnit")]
pub struct Unit;

#[test]
fn test_openapi_3_0() {
    use nexustack::openapi::json::{Specification, build_schema};
    let schema = build_schema::<Unit>(Specification::OpenAPI3_0).unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "description": "A unit struct",
            "example": null,
            "enum": [null],
            "nullable": true,
            "type": "string"
        })
    );
}

#[test]
fn test_openapi_3_0_with_collection() {
    use nexustack::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
    use std::{cell::RefCell, rc::Rc};

    let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));
    let schema =
        build_schema_with_collection::<Unit>(Specification::OpenAPI3_0, schema_collection.clone())
            .unwrap();

    let schemas_object = Rc::try_unwrap(schema_collection)
        .map_err(|_| "Should be the only Rc strong reference")
        .unwrap()
        .into_inner()
        .to_schemas_object();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "$ref": "#/components/schemas/MyUnit"
        })
    );

    pretty_assertions::assert_eq!(
        serde_json::to_value(schemas_object).unwrap(),
        serde_json::json!({
            "MyUnit": {
                "description": "A unit struct",
                "example": null,
                "enum": [null],
                "nullable": true,
                "type": "string"
            }
        })
    );
}

#[test]
fn test_openapi_3_1() {
    use nexustack::openapi::json::{Specification, build_schema};
    let schema = build_schema::<Unit>(Specification::OpenAPI3_1).unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "description": "A unit struct",
            "examples": [null],
            "type": "null"
        })
    );
}

#[test]
fn test_openapi_3_1_with_collection() {
    use nexustack::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
    use std::{cell::RefCell, rc::Rc};

    let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));
    let schema =
        build_schema_with_collection::<Unit>(Specification::OpenAPI3_1, schema_collection.clone())
            .unwrap();

    let schemas_object = Rc::try_unwrap(schema_collection)
        .map_err(|_| "Should be the only Rc strong reference")
        .unwrap()
        .into_inner()
        .to_schemas_object();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "$ref": "#/components/schemas/MyUnit"
        })
    );

    pretty_assertions::assert_eq!(
        serde_json::to_value(schemas_object).unwrap(),
        serde_json::json!({
            "MyUnit": {
                "description": "A unit struct",
                "examples": [null],
                "type": "null"
            }
        })
    );
}
