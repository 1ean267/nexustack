/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use nexustack::openapi::api_schema;

/// A pagination query result
#[api_schema]
pub struct Pagination<T> {
    /// The zero-based page index of the current page
    page: i32,

    /// The size of one page
    size: i32,

    /// The total number of pages
    total_pages: i32,

    /// The entries of the current page
    data: Vec<T>,
}

#[test]
fn test_openapi_3_0() {
    use nexustack::openapi::json::{Specification, build_schema};

    let schema = build_schema::<Pagination<f64>>(Specification::OpenAPI3_0).unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "allOf": [
                {
                    "description": "A pagination query result",
                    "example":  {
                        "data": []
                    },
                    "properties": {
                        "data": {
                            "description": "The entries of the current page",
                            "example": [],
                            "items": {
                                "example": 3.5,
                                "type": "number"
                            },
                            "type": "array"
                        }
                    },
                    "required": [
                        "data"
                    ],
                    "type": "object"
                },
                {
                    "description": "A pagination query result",
                    "example": {
                        "page": -2_147_483_648,
                        "size": -2_147_483_648,
                        "total_pages": -2_147_483_648
                    },
                    "properties": {
                        "page": {
                            "description": "The zero-based page index of the current page",
                            "example":  -2_147_483_648,
                            "maximum": 2_147_483_647,
                            "minimum": -2_147_483_648,
                            "type": "integer"
                        },
                        "size": {
                            "description": "The size of one page",
                            "example": -2_147_483_648,
                            "maximum": 2_147_483_647,
                            "minimum": -2_147_483_648,
                            "type": "integer"
                        },
                        "total_pages": {
                            "description": "The total number of pages",
                            "example": -2_147_483_648,
                            "maximum": 2_147_483_647,
                            "minimum": -2_147_483_648,
                            "type": "integer"
                        }
                    },
                    "required": [
                        "page",
                        "size",
                        "total_pages"
                    ],
                    "type": "object"
                }
            ],
            "description": "A pagination query result",
            "example": {
                "data": [],
                "page": -2_147_483_648,
                "size": -2_147_483_648,
                "total_pages": -2_147_483_648
            },
        })
    );
}

#[test]
fn test_openapi_3_0_with_collection() {
    use nexustack::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
    use std::{cell::RefCell, rc::Rc};

    let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));
    let schema = build_schema_with_collection::<Pagination<f64>>(
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
            "allOf": [
                {
                    "description": "A pagination query result",
                    "example": {
                        "data": []
                    },
                    "properties": {
                        "data": {
                            "description": "The entries of the current page",
                            "example": [],
                            "items": {
                                "example": 3.5,
                                "type": "number"
                            },
                            "type": "array"
                        }
                    },
                    "required": [
                        "data"
                    ],
                    "type": "object"
                },
                {
                    "$ref": "#/components/schemas/Pagination",
                    "description": "A pagination query result"
                }
            ],
            "description": "A pagination query result",
            "example": {
                "data": [],
                "page": -2_147_483_648,
                "size": -2_147_483_648,
                "total_pages": -2_147_483_648
            }
        })
    );

    pretty_assertions::assert_eq!(
        serde_json::to_value(schemas_object).unwrap(),
        serde_json::json!({
            "Pagination": {
                "description": "A pagination query result",
                "example": {
                    "page": -2_147_483_648,
                    "size": -2_147_483_648,
                    "total_pages": -2_147_483_648
                },
                "properties": {
                    "page": {
                        "description": "The zero-based page index of the current page",
                        "example":  -2_147_483_648,
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer"
                    },
                    "size": {
                        "description": "The size of one page",
                        "example":  -2_147_483_648,
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer"
                    },
                    "total_pages": {
                        "description": "The total number of pages",
                        "example": -2_147_483_648,
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer"
                    }
                },
                "required": [
                    "page",
                    "size",
                    "total_pages"
                ],
                "type": "object"
            }
        })
    );
}

#[test]
fn test_openapi_3_1() {
    use nexustack::openapi::json::{Specification, build_schema};

    let schema = build_schema::<Pagination<f64>>(Specification::OpenAPI3_1).unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "allOf": [
                {
                    "description": "A pagination query result",
                    "examples": [
                        {
                            "data": []
                        },
                        {
                            "data": [
                                3.5
                            ]
                        },
                        {
                            "data": [
                                3.5,
                                27.0
                            ]
                        },
                        {
                            "data": [
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
                        }
                    ],
                    "properties": {
                        "data": {
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
                    },
                    "required": [
                        "data"
                    ],
                    "type": "object"
                },
                {
                    "description": "A pagination query result",
                    "examples": [
                        {
                            "page": -2_147_483_648,
                            "size": -2_147_483_648,
                            "total_pages": -2_147_483_648
                        },
                        {
                            "page": -1,
                            "size": -1,
                            "total_pages": -1
                        },
                        {
                            "page": 0,
                            "size": 0,
                            "total_pages": 0
                        },
                        {
                            "page": 1,
                            "size": 1,
                            "total_pages": 1
                        },
                        {
                            "page": 2_147_483_647,
                            "size": 2_147_483_647,
                            "total_pages": 2_147_483_647
                        }
                    ],
                    "properties": {
                        "page": {
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
                        "size": {
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
                        "total_pages": {
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
                        }
                    },
                    "required": [
                        "page",
                        "size",
                        "total_pages"
                    ],
                    "type": "object"
                }
            ],
            "description": "A pagination query result",
            "examples": [
                {
                    "data": [],
                    "page": -2_147_483_648,
                    "size": -2_147_483_648,
                    "total_pages": -2_147_483_648
                },
                {
                    "data": [
                        3.5
                    ],
                    "page": -1,
                    "size": -1,
                    "total_pages": -1
                },
                {
                    "data": [
                        3.5,
                        27.0
                    ],
                    "page": 0,
                    "size": 0,
                    "total_pages": 0
                },
                {
                    "data": [
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
                    "page": 1,
                    "size": 1,
                    "total_pages": 1
                }
            ]
        })
    );
}

#[test]
fn test_openapi_3_1_with_collection() {
    use nexustack::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
    use std::{cell::RefCell, rc::Rc};

    let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));
    let schema = build_schema_with_collection::<Pagination<f64>>(
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
            "allOf": [
                {
                    "description": "A pagination query result",
                    "examples": [
                        {
                            "data": []
                        },
                        {
                            "data": [
                                3.5
                            ]
                        },
                        {
                            "data": [
                                3.5,
                                27.0
                            ]
                        },
                        {
                            "data": [
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
                        }
                    ],
                    "properties": {
                        "data": {
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
                    },
                    "required": [
                        "data"
                    ],
                    "type": "object"
                },
                {
                    "$ref": "#/components/schemas/Pagination",
                    "description": "A pagination query result"
                }
            ],
            "description": "A pagination query result",
            "examples": [
                {
                    "data": [],
                    "page": -2_147_483_648,
                    "size": -2_147_483_648,
                    "total_pages": -2_147_483_648
                },
                {
                    "data": [
                        3.5
                    ],
                    "page": -1,
                    "size": -1,
                    "total_pages": -1
                },
                {
                    "data": [
                        3.5,
                        27.0
                    ],
                    "page": 0,
                    "size": 0,
                    "total_pages": 0
                },
                {
                    "data": [
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
                    "page": 1,
                    "size": 1,
                    "total_pages": 1
                }
            ]
        })
    );

    pretty_assertions::assert_eq!(
        serde_json::to_value(schemas_object).unwrap(),
        serde_json::json!({
            "Pagination": {
                "description": "A pagination query result",
                "examples": [
                    {
                        "page": -2_147_483_648,
                        "size": -2_147_483_648,
                        "total_pages": -2_147_483_648
                    },
                    {
                        "page": -1,
                        "size": -1,
                        "total_pages": -1
                    },
                    {
                        "page": 0,
                        "size": 0,
                        "total_pages": 0
                    },
                    {
                        "page": 1,
                        "size": 1,
                        "total_pages": 1
                    },
                    {
                        "page": 2_147_483_647,
                        "size": 2_147_483_647,
                        "total_pages": 2_147_483_647
                    }
                ],
                "properties": {
                    "page": {
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
                    "size": {
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
                    "total_pages": {
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
                    }
                },
                "required": [
                    "page",
                    "size",
                    "total_pages"
                ],
                "type": "object"
            }
        })
    );
}
