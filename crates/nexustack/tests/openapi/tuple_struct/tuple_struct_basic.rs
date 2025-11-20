/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use nexustack::openapi::api_schema;

/// A point with optional x coordinate
#[api_schema]
pub struct Point(
    /// The optional x coordinate
    Option<i32>,
    /// The y coordinate
    i32,
);

#[test]
fn test_openapi_3_0() {
    use nexustack::openapi::{SpecificationVersion, generator::build_schema};

    let schema = build_schema::<Point>(SpecificationVersion::OpenAPI3_0).unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "description": "A point with optional x coordinate",
            "example": [-2_147_483_648, -2_147_483_648],
            "items": {
                "oneOf": [
                    {
                        "description": "The optional x coordinate",
                        "example": -2_147_483_648,
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "nullable": true,
                        "type": "integer"
                    },
                    {
                        "description": "The y coordinate",
                        "example": -2_147_483_648,
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer"
                    }
                ]
            },
            "maxItems": 2,
            "minItems": 2,
            "type": "array"
        })
    );
}

#[test]
fn test_openapi_3_0_with_collection() {
    use nexustack::openapi::{
        SpecificationVersion,
        generator::{SchemaCollection, build_schema_with_collection},
    };
    use std::{cell::RefCell, rc::Rc};

    let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));
    let schema = build_schema_with_collection::<Point>(
        SpecificationVersion::OpenAPI3_0,
        schema_collection.clone(),
    )
    .unwrap();

    let schemas_object = Rc::try_unwrap(schema_collection)
        .map_err(|_| "Should be the only Rc strong reference")
        .unwrap()
        .into_inner()
        .to_schemas_object();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "$ref": "#/components/schemas/Point"
        })
    );

    pretty_assertions::assert_eq!(
        serde_json::to_value(schemas_object).unwrap(),
        serde_json::json!({
            "Point": {
                "description": "A point with optional x coordinate",
                "example": [-2_147_483_648, -2_147_483_648],
                "items": {
                    "oneOf": [
                        {
                            "description": "The optional x coordinate",
                            "example": -2_147_483_648,
                            "maximum": 2_147_483_647,
                            "minimum": -2_147_483_648,
                            "nullable": true,
                            "type": "integer"
                        },
                        {
                            "description": "The y coordinate",
                            "example": -2_147_483_648,
                            "maximum": 2_147_483_647,
                            "minimum": -2_147_483_648,
                            "type": "integer"
                        }
                    ]
                },
                "maxItems": 2,
                "minItems": 2,
                "type": "array"
            }
        })
    );
}

#[test]
fn test_openapi_3_1() {
    use nexustack::openapi::{SpecificationVersion, generator::build_schema};

    let schema = build_schema::<Point>(SpecificationVersion::OpenAPI3_1).unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "description": "A point with optional x coordinate",
            "examples": [
                [-2_147_483_648, -2_147_483_648],
                [-1, -1],
                [0, 0],
                [1, 1],
                [2_147_483_647, 2_147_483_647]
            ],
            "maxItems": 2,
            "minItems": 2,
            "prefixItems": [
                {
                    "description": "The optional x coordinate",
                    "examples": [
                        -2_147_483_648,
                        -1,
                        0,
                        1,
                        2_147_483_647,
                        null
                    ],
                    "maximum": 2_147_483_647,
                    "minimum": -2_147_483_648,
                    "type": [
                        "integer",
                        "null"
                    ]
                },
                {
                    "description": "The y coordinate",
                    "examples": [
                        -2_147_483_648,
                        -1,
                        0,
                        1,
                        2_147_483_647
                    ],
                    "maximum": 2_147_483_647,
                    "minimum": -2_147_483_648,
                    "type": "integer"
                }
            ],
            "type": "array"
        })
    );
}

#[test]
fn test_openapi_3_1_with_collection() {
    use nexustack::openapi::{
        SpecificationVersion,
        generator::{SchemaCollection, build_schema_with_collection},
    };
    use std::{cell::RefCell, rc::Rc};

    let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));
    let schema = build_schema_with_collection::<Point>(
        SpecificationVersion::OpenAPI3_1,
        schema_collection.clone(),
    )
    .unwrap();

    let schemas_object = Rc::try_unwrap(schema_collection)
        .map_err(|_| "Should be the only Rc strong reference")
        .unwrap()
        .into_inner()
        .to_schemas_object();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "$ref": "#/components/schemas/Point"
        })
    );

    pretty_assertions::assert_eq!(
        serde_json::to_value(schemas_object).unwrap(),
        serde_json::json!({
            "Point": {
                "description": "A point with optional x coordinate",
                "examples": [
                    [-2_147_483_648, -2_147_483_648],
                    [-1, -1],
                    [0, 0],
                    [1, 1],
                    [2_147_483_647, 2_147_483_647]
                ],
                "maxItems": 2,
                "minItems": 2,
                "prefixItems": [
                    {
                        "description": "The optional x coordinate",
                        "examples": [
                            -2_147_483_648,
                            -1,
                            0,
                            1,
                            2_147_483_647,
                            null
                        ],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": [
                            "integer",
                            "null"
                        ]
                    },
                    {
                        "description": "The y coordinate",
                        "examples": [
                            -2_147_483_648,
                            -1,
                            0,
                            1,
                            2_147_483_647
                        ],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer"
                    }
                ],
                "type": "array"
            }
        })
    );
}
