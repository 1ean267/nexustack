/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use nexustack::openapi::api_schema;

/// A point with optional x coordinate
#[api_schema]
pub struct Point {
    /// The optional x coordinate
    x: Option<i32>,

    /// The y coordinate
    y: i32,
}

/// A flattened point with additional a coordinate
#[api_schema]
pub struct Flatten {
    /// The additional a coordinate
    a: i32,

    /// The flattened point
    #[api_property(flatten)]
    point: Point,
}

#[test]
fn test_openapi_3_0() {
    use nexustack::openapi::json::{Specification, build_schema};

    let schema = build_schema::<Flatten>(Specification::OpenAPI3_0).unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "description": "A flattened point with additional a coordinate",
            "example": {
                "a": -2_147_483_648,
                "x": -2_147_483_648,
                "y": -2_147_483_648
            },
            "properties": {
                "a": {
                    "description": "The additional a coordinate",
                    "example": -2_147_483_648,
                    "maximum": 2_147_483_647,
                    "minimum": -2_147_483_648,
                    "type": "integer"
                },
                "x": {
                    "description": "The optional x coordinate",
                    "example": -2_147_483_648,
                    "maximum": 2_147_483_647,
                    "minimum": -2_147_483_648,
                    "nullable": true,
                    "type": "integer"
                },
                "y": {
                    "description": "The y coordinate",
                    "example":  -2_147_483_648,
                    "maximum": 2_147_483_647,
                    "minimum": -2_147_483_648,
                    "type": "integer"
                }
            },
            "required": [
                "a",
                "x",
                "y"
            ],
            "type": "object"
        })
    );
}

#[test]
fn test_openapi_3_0_with_collection() {
    use nexustack::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
    use std::{cell::RefCell, rc::Rc};

    let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));
    let schema = build_schema_with_collection::<Flatten>(
        Specification::OpenAPI3_0,
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
            "$ref": "#/components/schemas/Flatten"
        })
    );

    pretty_assertions::assert_eq!(
        serde_json::to_value(schemas_object).unwrap(),
        serde_json::json!({
            "Flatten": {
                "description": "A flattened point with additional a coordinate",
                "example": {
                    "a": -2_147_483_648,
                    "x": -2_147_483_648,
                    "y": -2_147_483_648
                },
                "properties": {
                    "a": {
                        "description": "The additional a coordinate",
                        "example": -2_147_483_648,
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer"
                    },
                    "x": {
                        "description": "The optional x coordinate",
                        "example": -2_147_483_648,
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "nullable": true,
                        "type": "integer"
                    },
                    "y": {
                        "description": "The y coordinate",
                        "example": -2_147_483_648,
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer"
                    }
                },
                "required": [
                    "a",
                    "x",
                    "y"
                ],
                "type": "object"
            }
        })
    );
}

#[test]
fn test_openapi_3_1() {
    use nexustack::openapi::json::{Specification, build_schema};

    let schema = build_schema::<Flatten>(Specification::OpenAPI3_1).unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "description": "A flattened point with additional a coordinate",
            "examples": [
                {
                    "a": -2_147_483_648,
                    "x": -2_147_483_648,
                    "y": -2_147_483_648
                },
                {
                    "a": -1,
                    "x": -1,
                    "y": -1
                },
                {
                    "a": 0,
                    "x": 0,
                    "y": 0
                },
                {
                    "a": 1,
                    "x": 1,
                    "y": 1
                },
                {
                    "a": 2_147_483_647,
                    "x": 2_147_483_647,
                    "y": 2_147_483_647
                }
            ],
            "properties": {
                "a": {
                    "description": "The additional a coordinate",
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
                },
                "x": {
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
                    "type": ["integer", "null"]
                },
                "y": {
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
            },
            "required": [
                "a",
                "x",
                "y"
            ],
            "type": "object"
        })
    );
}

#[test]
fn test_openapi_3_1_with_collection() {
    use nexustack::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
    use std::{cell::RefCell, rc::Rc};

    let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));
    let schema = build_schema_with_collection::<Flatten>(
        Specification::OpenAPI3_1,
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
            "$ref": "#/components/schemas/Flatten"
        })
    );

    pretty_assertions::assert_eq!(
        serde_json::to_value(schemas_object).unwrap(),
        serde_json::json!({
            "Flatten": {
                "description": "A flattened point with additional a coordinate",
                "examples": [
                    {
                        "a": -2_147_483_648,
                        "x": -2_147_483_648,
                        "y": -2_147_483_648
                    },
                    {
                        "a": -1,
                        "x": -1,
                        "y": -1
                    },
                    {
                        "a": 0,
                        "x": 0,
                        "y": 0
                    },
                    {
                        "a": 1,
                        "x": 1,
                        "y": 1
                    },
                    {
                        "a": 2_147_483_647,
                        "x": 2_147_483_647,
                        "y": 2_147_483_647
                    }
                ],
                "properties": {
                    "a": {
                        "description": "The additional a coordinate",
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
                    },
                    "x": {
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
                        "type": ["integer", "null"]
                    },
                    "y": {
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
                },
                "required": [
                    "a",
                    "x",
                    "y"
                ],
                "type": "object"
            }
        })
    );
}
