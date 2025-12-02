/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use nexustack::openapi::api_schema;

/// A pagination query result
#[api_schema]
pub struct Pagination<T>(
    /// The zero-based page index of the current page
    i32,
    /// The size of one page
    i32,
    /// The total number of pages
    i32,
    /// The entries of the current page
    Vec<T>,
);

#[test]
fn test_openapi_3_0() {
    use nexustack::openapi::{SpecificationVersion, generator::build_schema};

    let schema = build_schema::<Pagination<f64>>(SpecificationVersion::OpenAPI3_0).unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "description": "A pagination query result",
            "example": [
                -2_147_483_648,
                -2_147_483_648,
                -2_147_483_648,
                [],
            ],
            "items": {
                "oneOf": [
                    {
                        "description": "The zero-based page index of the current page",
                        "example": -2_147_483_648,
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer"
                    },
                    {
                        "description": "The size of one page",
                        "example": -2_147_483_648,
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer"
                    },
                    {
                        "description": "The total number of pages",
                        "example": -2_147_483_648,
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer"
                    },
                    {
                        "description": "The entries of the current page",
                        "example": [],
                        "items": {
                            "example": 3.5,
                            "type": "number"
                        },
                        "type": "array"
                    }
                ]
            },
            "maxItems": 4,
            "minItems": 4,
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
    let schema = build_schema_with_collection::<Pagination<f64>>(
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
            "description": "A pagination query result",
            "example": [
                -2_147_483_648,
                -2_147_483_648,
                -2_147_483_648,
                [],
            ],
            "items": {
                "oneOf": [
                    {
                        "description": "The zero-based page index of the current page",
                        "example": -2_147_483_648,
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer"
                    },
                    {
                        "description": "The size of one page",
                        "example": -2_147_483_648,
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer"
                    },
                    {
                        "description": "The total number of pages",
                        "example": -2_147_483_648,
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer"
                    },
                    {
                        "description": "The entries of the current page",
                        "example": [],
                        "items": {
                            "example": 3.5,
                            "type": "number"
                        },
                        "type": "array"
                    }
                ]
            },
            "maxItems": 4,
            "minItems": 4,
            "type": "array"
        })
    );

    pretty_assertions::assert_eq!(
        serde_json::to_value(schemas_object).unwrap(),
        serde_json::json!({})
    );
}

#[test]
fn test_openapi_3_1() {
    use nexustack::openapi::{SpecificationVersion, generator::build_schema};

    let schema = build_schema::<Pagination<f64>>(SpecificationVersion::OpenAPI3_1).unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "description": "A pagination query result",
            "examples": [
                [
                    -2_147_483_648,
                    -2_147_483_648,
                    -2_147_483_648,
                    [],
                ], [
                    -1,
                    -1,
                    -1,
                    [
                        3.5
                    ],
                ], [
                    0,
                    0,
                    0,
                    [
                        3.5,
                        27.0
                    ],
                ], [
                    1,
                    1,
                    1,
                    [
                        3.5,
                        27.0,
                        -113.75,
                        0.007_812_5,
                        34_359_738_368.0,
                        0.0,
                        -1.0,
                        3.5,
                        27.0,
                        -113.75
                    ],
                ]
            ],
            "prefixItems": [
                {
                    "description": "The zero-based page index of the current page",
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
                {
                    "description": "The size of one page",
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
                {
                    "description": "The total number of pages",
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
                {
                    "description": "The entries of the current page",
                    "examples": [
                        [],
                        [
                            3.5
                        ],
                        [
                            3.5,
                            27.0
                        ],
                        [
                            3.5,
                            27.0,
                            -113.75,
                            0.007_812_5,
                            34_359_738_368.0,
                            0.0,
                            -1.0,
                            3.5,
                            27.0,
                            -113.75
                        ]
                    ],
                    "items": {
                        "examples": [
                            3.5,
                            27.0,
                            -113.75,
                            0.007_812_5,
                            34_359_738_368.0,
                            0.0,
                            -1.0
                        ],
                        "type": "number"
                    },
                    "type": "array"
                }
            ],
            "maxItems": 4,
            "minItems": 4,
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
    let schema = build_schema_with_collection::<Pagination<f64>>(
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
            "description": "A pagination query result",
            "examples": [
                [
                    -2_147_483_648,
                    -2_147_483_648,
                    -2_147_483_648,
                    [],
                ], [
                    -1,
                    -1,
                    -1,
                    [
                        3.5
                    ],
                ], [
                    0,
                    0,
                    0,
                    [
                        3.5,
                        27.0
                    ],
                ], [
                    1,
                    1,
                    1,
                    [
                        3.5,
                        27.0,
                        -113.75,
                        0.007_812_5,
                        34_359_738_368.0,
                        0.0,
                        -1.0,
                        3.5,
                        27.0,
                        -113.75
                    ],
                ]
            ],
            "prefixItems": [
                {
                    "description": "The zero-based page index of the current page",
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
                {
                    "description": "The size of one page",
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
                {
                    "description": "The total number of pages",
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
                {
                    "description": "The entries of the current page",
                    "examples": [
                        [],
                        [
                            3.5
                        ],
                        [
                            3.5,
                            27.0
                        ],
                        [
                            3.5,
                            27.0,
                            -113.75,
                            0.007_812_5,
                            34_359_738_368.0,
                            0.0,
                            -1.0,
                            3.5,
                            27.0,
                            -113.75
                        ]
                    ],
                    "items": {
                        "examples": [
                            3.5,
                            27.0,
                            -113.75,
                            0.007_812_5,
                            34_359_738_368.0,
                            0.0,
                            -1.0
                        ],
                        "type": "number"
                    },
                    "type": "array"
                }
            ],
            "maxItems": 4,
            "minItems": 4,
            "type": "array"
        })
    );

    pretty_assertions::assert_eq!(
        serde_json::to_value(schemas_object).unwrap(),
        serde_json::json!({})
    );
}
