/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use nexustack::openapi::api_schema;

/// A description
#[api_schema]
pub struct Wrapped {
    /// Field r
    r: f64,
    /// Field s
    s: Option<f64>,
}

/// A test enum
#[api_schema(tag = "type")]
pub enum Message {
    /// Variant A
    VariantA,
    /// Variant B
    VariantB(
        /// Content of variant B
        Wrapped,
    ),
    /// Variant D
    VariantD {
        /// Field x of variant D
        x: i32,
        /// Field y of variant D
        y: Option<i32>,
    },

    /// Other stuff
    #[api_variant(other)]
    Other,
}

#[test]
fn test_openapi_3_0() {
    use nexustack::openapi::json::{Specification, build_schema};
    let schema = build_schema::<Message>(Specification::OpenAPI3_0).unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "description": "A test enum",
            "example": { "type": "VariantA" },
            "anyOf": [
                {
                    "type": "object",
                    "description": "Variant A",
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": ["VariantA"]
                        }
                    },
                    "required": ["type"]
                },
                {
                    "type": "object",
                    "description": "Variant B",
                    "required": ["r", "s", "type"],
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": ["VariantB"]
                        },
                        "r": {
                            "description": "Field r",
                            "example": 3.5,
                            "type": "number",
                        },
                        "s": {
                            "description": "Field s",
                            "example": 3.5,
                            "nullable": true,
                            "type": "number"
                        }
                    },
                    "example": {
                        "type": "VariantB",
                        "r": 3.5,
                        "s": 3.5,
                    },
                },
                {
                    "description": "Variant D",
                    "type": "object",
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": ["VariantD"]
                        },
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
                        "type",
                        "x",
                        "y",
                    ]
                },
                {
                    "type": "object",
                    "properties": {
                        "type": {
                            "type": "string",
                            "pattern": "(?=^[^V\\n].*$|^V$|^V[^a\\n].*$|^Va$|^Va[^r\\n].*$|^Var$|^Var[^i\\n].*$|^Vari$|^Vari[^a\\n].*$|^Varia$|^Varia[^n\\n].*$|^Varian$|^Varian[^t\\n].*$|^Variant$|^Variant[^A\\n].*$|^VariantA.+$)(?=^[^V\\n].*$|^V$|^V[^a\\n].*$|^Va$|^Va[^r\\n].*$|^Var$|^Var[^i\\n].*$|^Vari$|^Vari[^a\\n].*$|^Varia$|^Varia[^n\\n].*$|^Varian$|^Varian[^t\\n].*$|^Variant$|^Variant[^B\\n].*$|^VariantB.+$)(?=^[^V\\n].*$|^V$|^V[^a\\n].*$|^Va$|^Va[^r\\n].*$|^Var$|^Var[^i\\n].*$|^Vari$|^Vari[^a\\n].*$|^Varia$|^Varia[^n\\n].*$|^Varian$|^Varian[^t\\n].*$|^Variant$|^Variant[^D\\n].*$|^VariantD.+$)^.*$"
                        }
                    },
                    "required": ["type"]
                },
            ]
        })
    );
}

#[test]
fn test_openapi_3_0_with_collection() {
    use nexustack::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
    use std::{cell::RefCell, rc::Rc};

    let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));
    let schema = build_schema_with_collection::<Message>(
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
            "$ref": "#/components/schemas/Message"
        })
    );

    pretty_assertions::assert_eq!(
        serde_json::to_value(schemas_object).unwrap(),
        serde_json::json!({
            "Message": {
                "description": "A test enum",
                "example": { "type": "VariantA" },
                "anyOf": [
                    {
                        "type": "object",
                        "description": "Variant A",
                        "properties": {
                            "type": {
                                "type": "string",
                                "enum": ["VariantA"]
                            }
                        },
                        "required": ["type"]
                    },
                    {
                        "description": "Variant B",
                        "allOf": [
                            {
                                "$ref": "#/components/schemas/Wrapped",
                            },
                            {
                                "type": "object",
                                "required": ["type"],
                                "properties": {
                                    "type": {
                                        "type": "string",
                                        "enum": ["VariantB"]
                                    },
                                },
                            }
                        ]
                    },
                    {
                        "description": "Variant D",
                        "type": "object",
                        "properties": {
                            "type": {
                                "type": "string",
                                "enum": ["VariantD"]
                            },
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
                                "type": "integer",
                            }
                        },
                        "required": [
                            "type",
                            "x",
                            "y",
                        ]
                    },
                    {
                        "type": "object",
                        "properties": {
                            "type": {
                                "type": "string",
                                "pattern": "(?=^[^V\\n].*$|^V$|^V[^a\\n].*$|^Va$|^Va[^r\\n].*$|^Var$|^Var[^i\\n].*$|^Vari$|^Vari[^a\\n].*$|^Varia$|^Varia[^n\\n].*$|^Varian$|^Varian[^t\\n].*$|^Variant$|^Variant[^A\\n].*$|^VariantA.+$)(?=^[^V\\n].*$|^V$|^V[^a\\n].*$|^Va$|^Va[^r\\n].*$|^Var$|^Var[^i\\n].*$|^Vari$|^Vari[^a\\n].*$|^Varia$|^Varia[^n\\n].*$|^Varian$|^Varian[^t\\n].*$|^Variant$|^Variant[^B\\n].*$|^VariantB.+$)(?=^[^V\\n].*$|^V$|^V[^a\\n].*$|^Va$|^Va[^r\\n].*$|^Var$|^Var[^i\\n].*$|^Vari$|^Vari[^a\\n].*$|^Varia$|^Varia[^n\\n].*$|^Varian$|^Varian[^t\\n].*$|^Variant$|^Variant[^D\\n].*$|^VariantD.+$)^.*$"
                            }
                        },
                        "required": ["type"]
                    },
                ]
            },
            "Wrapped": {
                "type": "object",
                "description": "A description",
                "required": ["r", "s"],
                "properties": {
                    "r": {
                        "description": "Field r",
                        "example": 3.5,
                        "type": "number",
                    },
                    "s": {
                        "description": "Field s",
                        "example": 3.5,
                        "nullable": true,
                        "type": "number",
                    }
                },
                "example":  {
                    "r": 3.5,
                    "s": 3.5,
                },
            }
        })
    );
}

#[test]
fn test_openapi_3_1() {
    use nexustack::openapi::json::{Specification, build_schema};
    let schema = build_schema::<Message>(Specification::OpenAPI3_1).unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "description": "A test enum",
            "examples": [
                { "type": "VariantA" },
                {
                    "type": "VariantB",
                    "r": 3.5,
                    "s": 3.5,
                },
                {
                    "type": "VariantB",
                    "r": 27.0,
                    "s": 27.0,
                },
                {
                    "type": "VariantB",
                    "r": -113.75,
                    "s": -113.75,
                },
                {
                    "type": "VariantB",
                    "r": 0.007_812_5,
                    "s": 0.007_812_5,
                },
                {
                    "type": "VariantB",
                    "r": 34_359_738_368.0,
                    "s": 34_359_738_368.0,
                },
                {
                    "type": "VariantB",
                    "r": 0.0,
                    "s": 0.0,
                },
                {
                    "type": "VariantB",
                    "r": -1.0,
                    "s": -1.0,
                },
                {
                    "type": "VariantD",
                    "x": -2_147_483_648,
                    "y": -2_147_483_648
                },
                {
                    "type": "VariantD",
                    "x": -1,
                    "y": -1
                },
                {
                    "type": "VariantD",
                    "x": 0,
                    "y": 0
                },
                {
                    "type": "VariantD",
                    "x": 1,
                    "y": 1
                },
                {
                    "type": "VariantD",
                    "x": 2_147_483_647,
                    "y": 2_147_483_647
                }
            ],
            "anyOf": [
                {
                    "type": "object",
                    "description": "Variant A",
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": ["VariantA"]
                        }
                    },
                    "required": ["type"]
                },
                {
                    "type": "object",
                    "description": "Variant B",
                    "required": ["r", "s", "type"],
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": ["VariantB"]
                        },
                        "r": {
                            "description": "Field r",
                            "examples": [
                                3.5,
                                27.0,
                                -113.75,
                                0.007_812_5,
                                34_359_738_368.0,
                                0.0,
                                -1.0,
                            ],
                            "type": "number",
                        },
                        "s": {
                            "description": "Field s",
                            "examples": [
                                3.5,
                                27.0,
                                -113.75,
                                0.007_812_5,
                                34_359_738_368.0,
                                0.0,
                                -1.0,
                                null
                            ],
                            "type": [
                                "number",
                                "null",
                            ],
                        }
                    },
                    "examples": [
                        {
                            "type": "VariantB",
                            "r": 3.5,
                            "s": 3.5,
                        },
                        {
                            "type": "VariantB",
                            "r": 27.0,
                            "s": 27.0,
                        },
                        {
                            "type": "VariantB",
                            "r": -113.75,
                            "s": -113.75,
                        },
                        {
                            "type": "VariantB",
                            "r": 0.007_812_5,
                            "s": 0.007_812_5,
                        },
                        {
                            "type": "VariantB",
                            "r": 34_359_738_368.0,
                            "s": 34_359_738_368.0,
                        },
                        {
                            "type": "VariantB",
                            "r": 0.0,
                            "s": 0.0,
                        },
                        {
                            "type": "VariantB",
                            "r": -1.0,
                            "s": -1.0,
                        },
                    ],
                },
                {
                    "description": "Variant D",
                    "type": "object",
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": ["VariantD"]
                        },
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
                        "type",
                        "x",
                        "y",
                    ]
                },
                {
                    "type": "object",
                    "properties": {
                        "type": {
                            "type": "string",
                            "pattern": "(?=^[^V\\n].*$|^V$|^V[^a\\n].*$|^Va$|^Va[^r\\n].*$|^Var$|^Var[^i\\n].*$|^Vari$|^Vari[^a\\n].*$|^Varia$|^Varia[^n\\n].*$|^Varian$|^Varian[^t\\n].*$|^Variant$|^Variant[^A\\n].*$|^VariantA.+$)(?=^[^V\\n].*$|^V$|^V[^a\\n].*$|^Va$|^Va[^r\\n].*$|^Var$|^Var[^i\\n].*$|^Vari$|^Vari[^a\\n].*$|^Varia$|^Varia[^n\\n].*$|^Varian$|^Varian[^t\\n].*$|^Variant$|^Variant[^B\\n].*$|^VariantB.+$)(?=^[^V\\n].*$|^V$|^V[^a\\n].*$|^Va$|^Va[^r\\n].*$|^Var$|^Var[^i\\n].*$|^Vari$|^Vari[^a\\n].*$|^Varia$|^Varia[^n\\n].*$|^Varian$|^Varian[^t\\n].*$|^Variant$|^Variant[^D\\n].*$|^VariantD.+$)^.*$"
                        }
                    },
                    "required": ["type"]
                },
            ]
        })
    );
}

#[test]
fn test_openapi_3_1_with_collection() {
    use nexustack::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
    use std::{cell::RefCell, rc::Rc};

    let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));
    let schema = build_schema_with_collection::<Message>(
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
            "$ref": "#/components/schemas/Message"
        })
    );

    pretty_assertions::assert_eq!(
        serde_json::to_value(schemas_object).unwrap(),
        serde_json::json!({
            "Message": {
                "description": "A test enum",
                "examples": [
                    { "type": "VariantA" },
                    {
                        "type": "VariantB",
                        "r": 3.5,
                        "s": 3.5,
                    },
                    {
                        "type": "VariantB",
                        "r": 27.0,
                        "s": 27.0,
                    },
                    {
                        "type": "VariantB",
                        "r": -113.75,
                        "s": -113.75,
                    },
                    {
                        "type": "VariantB",
                        "r": 0.007_812_5,
                        "s": 0.007_812_5,
                    },
                    {
                        "type": "VariantB",
                        "r": 34_359_738_368.0,
                        "s": 34_359_738_368.0,
                    },
                    {
                        "type": "VariantB",
                        "r": 0.0,
                        "s": 0.0,
                    },
                    {
                        "type": "VariantB",
                        "r": -1.0,
                        "s": -1.0,
                    },
                    {
                        "type": "VariantD",
                        "x": -2_147_483_648,
                        "y": -2_147_483_648
                    },
                    {
                        "type": "VariantD",
                        "x": -1,
                        "y": -1
                    },
                    {
                        "type": "VariantD",
                        "x": 0,
                        "y": 0
                    },
                    {
                        "type": "VariantD",
                        "x": 1,
                        "y": 1
                    },
                    {
                        "type": "VariantD",
                        "x": 2_147_483_647,
                        "y": 2_147_483_647
                    }
                ],
                "anyOf": [
                    {
                        "type": "object",
                        "description": "Variant A",
                        "properties": {
                            "type": {
                                "type": "string",
                                "enum": ["VariantA"]
                            }
                        },
                        "required": ["type"]
                    },
                    {
                        "description": "Variant B",
                        "allOf": [
                            {
                                "$ref": "#/components/schemas/Wrapped",
                            },
                            {
                                "type": "object",
                                "required": ["type"],
                                "properties": {
                                    "type": {
                                        "type": "string",
                                        "enum": ["VariantB"]
                                    },
                                },
                            }
                        ]
                    },
                    {
                        "description": "Variant D",
                        "type": "object",
                        "properties": {
                            "type": {
                                "type": "string",
                                "enum": ["VariantD"]
                            },
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
                            "type",
                            "x",
                            "y",
                        ]
                    },
                    {
                        "type": "object",
                        "properties": {
                            "type": {
                                "type": "string",
                                "pattern": "(?=^[^V\\n].*$|^V$|^V[^a\\n].*$|^Va$|^Va[^r\\n].*$|^Var$|^Var[^i\\n].*$|^Vari$|^Vari[^a\\n].*$|^Varia$|^Varia[^n\\n].*$|^Varian$|^Varian[^t\\n].*$|^Variant$|^Variant[^A\\n].*$|^VariantA.+$)(?=^[^V\\n].*$|^V$|^V[^a\\n].*$|^Va$|^Va[^r\\n].*$|^Var$|^Var[^i\\n].*$|^Vari$|^Vari[^a\\n].*$|^Varia$|^Varia[^n\\n].*$|^Varian$|^Varian[^t\\n].*$|^Variant$|^Variant[^B\\n].*$|^VariantB.+$)(?=^[^V\\n].*$|^V$|^V[^a\\n].*$|^Va$|^Va[^r\\n].*$|^Var$|^Var[^i\\n].*$|^Vari$|^Vari[^a\\n].*$|^Varia$|^Varia[^n\\n].*$|^Varian$|^Varian[^t\\n].*$|^Variant$|^Variant[^D\\n].*$|^VariantD.+$)^.*$"
                            }
                        },
                        "required": ["type"]
                    },
                ]
            },
            "Wrapped": {
                "type": "object",
                "description": "A description",
                "required": ["r", "s"],
                "properties": {
                    "r": {
                        "description": "Field r",
                        "examples": [
                            3.5,
                            27.0,
                            -113.75,
                            0.007_812_5,
                            34_359_738_368.0,
                            0.0,
                            -1.0,
                        ],
                        "type": "number",
                    },
                    "s": {
                        "description": "Field s",
                        "examples": [
                            3.5,
                            27.0,
                            -113.75,
                            0.007_812_5,
                            34_359_738_368.0,
                            0.0,
                            -1.0,
                            null
                        ],
                        "type": [
                            "number",
                            "null",
                        ],
                    }
                },
                "examples": [
                    {
                        "r": 3.5,
                        "s": 3.5,
                    },
                    {
                        "r": 27.0,
                        "s": 27.0,
                    },
                    {
                        "r": -113.75,
                        "s": -113.75,
                    },
                    {
                        "r": 0.007_812_5,
                        "s": 0.007_812_5,
                    },
                    {
                        "r": 34_359_738_368.0,
                        "s": 34_359_738_368.0,
                    },
                    {
                        "r": 0.0,
                        "s": 0.0,
                    },
                    {
                        "r": -1.0,
                        "s": -1.0,
                    },
                ],

            }
        })
    );
}
