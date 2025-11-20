/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use nexustack::openapi::api_schema;

/// A test enum
#[api_schema(rename = "Other")]
pub enum Message {
    /// Variant A
    A,
    /// Variant B
    B(
        /// Content of variant B
        Option<i32>,
    ),
    /// Variant C
    C(
        /// First entry of variant C
        Option<i32>,
        /// Second entry of variant C
        i32,
    ),

    /// Variant D
    D {
        /// Field x of variant D
        x: i32,
        /// Field y of variant D
        y: Option<i32>,
    },
}

#[test]
fn test_openapi_3_0() {
    use nexustack::openapi::{SpecificationVersion, generator::build_schema};
    let schema = build_schema::<Message>(SpecificationVersion::OpenAPI3_0).unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "description": "A test enum",
            "example": "A",
            "anyOf": [
                {
                    "type": "string",
                    "description": "Variant A",
                    "enum": ["A"]
                },
                {
                    "description": "Variant B",
                    "type": "object",
                    "required": ["B"],
                    "properties": {
                        "B": {
                            "example": -2_147_483_648,
                            "maximum": 2_147_483_647,
                            "minimum": -2_147_483_648,
                            "nullable": true,
                            "type": "integer"
                        }
                    },
                },
                {
                    "description": "Variant C",
                    "type": "object",
                    "required": ["C"],
                    "properties": {
                        "C": {
                            "type": "array",
                            "minItems": 2,
                            "maxItems": 2,
                            "items": {
                                "oneOf": [
                                    {
                                        "description": "First entry of variant C",
                                        "example": -2_147_483_648,
                                        "maximum": 2_147_483_647,
                                        "minimum": -2_147_483_648,
                                        "nullable": true,
                                        "type": "integer"
                                    },
                                    {
                                        "description": "Second entry of variant C",
                                        "example": -2_147_483_648,
                                        "maximum": 2_147_483_647,
                                        "minimum": -2_147_483_648,
                                        "type": "integer"
                                    }
                                ]
                            }
                        }
                    }
                },
                {
                    "description": "Variant D",
                    "type": "object",
                    "required": ["D"],
                    "properties": {
                        "D": {

                            "type": "object",
                            "properties": {
                                "x": {
                                    "description": "Field x of variant D",
                                    "example": -2_147_483_648,
                                    "maximum": 2_147_483_647,
                                    "minimum": -2_147_483_648,
                                    "type": "integer"
                                },
                                "y": {
                                    "description": "Field y of variant D",
                                    "example": -2_147_483_648,
                                    "maximum": 2_147_483_647,
                                    "minimum": -2_147_483_648,
                                    "nullable": true,
                                    "type": "integer"
                                }
                            },
                            "required": [
                                "x",
                                "y"
                            ]
                        }
                    }
                }
            ]
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
    let schema = build_schema_with_collection::<Message>(
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
            "$ref": "#/components/schemas/Other"
        })
    );

    pretty_assertions::assert_eq!(
        serde_json::to_value(schemas_object).unwrap(),
        serde_json::json!({
            "Other": {
                "description": "A test enum",
                "example": "A",
                "anyOf": [
                    {
                        "type": "string",
                        "description": "Variant A",
                        "enum": ["A"]
                    },
                    {
                        "description": "Variant B",
                        "type": "object",
                        "required": ["B"],
                        "properties": {
                            "B": {
                                "example": -2_147_483_648,
                                "maximum": 2_147_483_647,
                                "minimum": -2_147_483_648,
                                "nullable": true,
                                "type": "integer",
                            }
                        },
                    },
                    {
                        "description": "Variant C",
                        "type": "object",
                        "required": ["C"],
                        "properties": {
                            "C": {
                                "type": "array",
                                "minItems": 2,
                                "maxItems": 2,
                                "items": {
                                    "oneOf": [
                                        {
                                            "description": "First entry of variant C",
                                            "example": -2_147_483_648,
                                            "maximum": 2_147_483_647,
                                            "minimum": -2_147_483_648,
                                            "nullable": true,
                                            "type": "integer",
                                        },
                                        {
                                            "description": "Second entry of variant C",
                                            "example": -2_147_483_648,
                                            "maximum": 2_147_483_647,
                                            "minimum": -2_147_483_648,
                                            "type": "integer"
                                        }
                                    ]
                                }
                            }
                        }
                    },
                    {
                        "description": "Variant D",
                        "type": "object",
                        "required": ["D"],
                        "properties": {
                            "D": {
                                "type": "object",
                                "properties": {
                                    "x": {
                                        "description": "Field x of variant D",
                                        "example": -2_147_483_648,
                                        "maximum": 2_147_483_647,
                                        "minimum": -2_147_483_648,
                                        "type": "integer"
                                    },
                                    "y": {
                                        "description": "Field y of variant D",
                                        "example": -2_147_483_648,
                                        "maximum": 2_147_483_647,
                                        "minimum": -2_147_483_648,
                                        "nullable": true,
                                        "type": "integer"
                                    }
                                },
                                "required": [
                                    "x",
                                    "y"
                                ]
                            }
                        }
                    }
                ]
            }
        })
    );
}

#[test]
fn test_openapi_3_1() {
    use nexustack::openapi::{SpecificationVersion, generator::build_schema};
    let schema = build_schema::<Message>(SpecificationVersion::OpenAPI3_1).unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "description": "A test enum",
            "examples": [
                "A",
                { "B": -2_147_483_648 },
                { "B": -1 },
                { "B": 0 },
                { "B": 1 },
                { "B": 2_147_483_647 },
                { "B": null },
                {
                    "C": [
                        -2_147_483_648,
                        -2_147_483_648
                    ]
                },
                {
                    "C": [
                        -1,
                        -1
                    ],
                },
                {
                    "C": [
                        0,
                        0
                    ],
                },
                {
                    "C": [
                        1,
                        1
                    ],
                },
                {
                    "C": [
                        2_147_483_647,
                        2_147_483_647
                    ],
                },
                {
                    "D": {
                        "x": -2_147_483_648,
                        "y": -2_147_483_648
                    },
                },
                {
                    "D": {
                        "x": -1,
                        "y": -1
                    },
                },
                {
                    "D": {
                        "x": 0,
                        "y": 0
                    },
                },
                {
                    "D": {
                        "x": 1,
                        "y": 1
                    },
                },
                {
                    "D": {
                        "x": 2_147_483_647,
                        "y": 2_147_483_647
                    }
                }
            ],
            "anyOf": [
                {
                    "type": "string",
                    "description": "Variant A",
                    "enum": ["A"]
                },
                {
                    "description": "Variant B",
                    "type": "object",
                    "required": ["B"],
                    "properties": {
                        "B": {
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
                        }
                    },
                },
                {
                    "description": "Variant C",
                    "type": "object",
                    "required": ["C"],
                    "properties": {
                        "C": {
                            "type": "array",
                            "minItems": 2,
                            "maxItems": 2,
                            "prefixItems": [
                                {
                                    "description": "First entry of variant C",
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
                                    "description": "Second entry of variant C",
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
                            ]
                        }
                    }
                },
                {
                    "description": "Variant D",
                    "type": "object",
                    "required": ["D"],
                    "properties": {
                        "D": {
                            "type": "object",
                            "properties": {
                                "x": {
                                    "description": "Field x of variant D",
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
                                "y": {
                                    "description": "Field y of variant D",
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
                                }
                            },
                            "required": [
                                "x",
                                "y"
                            ]
                        }
                    }
                }
            ]
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
    let schema = build_schema_with_collection::<Message>(
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
            "$ref": "#/components/schemas/Other"
        })
    );

    pretty_assertions::assert_eq!(
        serde_json::to_value(schemas_object).unwrap(),
        serde_json::json!({
            "Other": {
                "description": "A test enum",
                "examples": [
                    "A",
                    { "B": -2_147_483_648 },
                    { "B": -1 },
                    { "B": 0 },
                    { "B": 1 },
                    { "B": 2_147_483_647 },
                    { "B": null },
                    {
                        "C": [
                            -2_147_483_648,
                            -2_147_483_648
                        ]
                    },
                    {
                        "C": [
                            -1,
                            -1
                        ],
                    },
                    {
                        "C": [
                            0,
                            0
                        ],
                    },
                    {
                        "C": [
                            1,
                            1
                        ],
                    },
                    {
                        "C": [
                            2_147_483_647,
                            2_147_483_647
                        ],
                    },
                    {
                        "D": {
                            "x": -2_147_483_648,
                            "y": -2_147_483_648
                        },
                    },
                    {
                        "D": {
                            "x": -1,
                            "y": -1
                        },
                    },
                    {
                        "D": {
                            "x": 0,
                            "y": 0
                        },
                    },
                    {
                        "D": {
                            "x": 1,
                            "y": 1
                        },
                    },
                    {
                        "D": {
                            "x": 2_147_483_647,
                            "y": 2_147_483_647
                        }
                    }
                ],
                "anyOf": [
                    {
                        "type": "string",
                        "description": "Variant A",
                        "enum": ["A"]
                    },
                    {
                        "description": "Variant B",
                        "type": "object",
                        "required": ["B"],
                        "properties": {
                            "B": {
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
                            }
                        },
                    },
                    {
                        "description": "Variant C",
                        "type": "object",
                        "required": ["C"],
                        "properties": {
                            "C": {
                                "type": "array",
                                "minItems": 2,
                                "maxItems": 2,
                                "prefixItems": [
                                    {
                                        "description": "First entry of variant C",
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
                                        "description": "Second entry of variant C",
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
                                ]
                            }
                        }
                    },
                    {
                        "description": "Variant D",
                        "type": "object",
                        "required": ["D"],
                        "properties": {
                            "D": {
                                "type": "object",
                                "properties": {
                                    "x": {
                                        "description": "Field x of variant D",
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
                                    "y": {
                                        "description": "Field y of variant D",
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
                                    }
                                },
                                "required": [
                                    "x",
                                    "y"
                                ]
                            }
                        }
                    }
                ]
            }
        })
    );
}
