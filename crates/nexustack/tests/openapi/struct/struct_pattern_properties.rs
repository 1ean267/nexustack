/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use nexustack::openapi::api_schema;
use std::{collections::HashMap, net::Ipv4Addr, net::Ipv6Addr};

/// A flattened point with additional a coordinate
#[api_schema]
pub struct Flatten {
    /// The additional a coordinate
    a: i32,

    /// The extra elements
    #[api_property(flatten)]
    extra: HashMap<Ipv6Addr, Ipv4Addr>,
}

#[test]
fn test_openapi_3_0() {
    use nexustack::openapi::{SpecificationVersion, generator::build_schema};

    let schema = build_schema::<Flatten>(SpecificationVersion::OpenAPI3_0).unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "additionalProperties": {
                "description": "An IPv4 address according to [IETF RFC 791](https://tools.ietf.org/html/rfc791)",
                "example":  "1.2.3.4",
                "format": "ipv4",
                "maxLength": 15,
                "minLength": 7,
                "pattern": "^(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)$",
                "type": "string"
            },
            "description": "A flattened point with additional a coordinate",
            "example": {
                "a": -2_147_483_648
            },
            "properties": {
                "a": {
                    "description": "The additional a coordinate",
                    "example": -2_147_483_648,
                    "maximum": 2_147_483_647,
                    "minimum": -2_147_483_648,
                    "type": "integer"
                }
            },
            "required": [
                "a"
            ],
            "type": "object"
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
    let schema = build_schema_with_collection::<Flatten>(
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
            "$ref": "#/components/schemas/Flatten"
        })
    );

    pretty_assertions::assert_eq!(
        serde_json::to_value(schemas_object).unwrap(),
        serde_json::json!({
            "Flatten": {
                "additionalProperties": {
                    "description": "An IPv4 address according to [IETF RFC 791](https://tools.ietf.org/html/rfc791)",
                    "example": "1.2.3.4",
                    "format": "ipv4",
                    "maxLength": 15,
                    "minLength": 7,
                    "pattern": "^(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)$",
                    "type": "string"
                },
                "description": "A flattened point with additional a coordinate",
                "example": {
                    "a": -2_147_483_648
                },
                "properties": {
                    "a": {
                        "description": "The additional a coordinate",
                        "example": -2_147_483_648,
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer"
                    }
                },
                "required": [
                    "a"
                ],
                "type": "object"
            }
        })
    );
}

#[test]
fn test_openapi_3_1() {
    use nexustack::openapi::{SpecificationVersion, generator::build_schema};

    let schema = build_schema::<Flatten>(SpecificationVersion::OpenAPI3_1).unwrap();

    pretty_assertions::assert_eq!(
        serde_json::to_value(schema).unwrap(),
        serde_json::json!({
            "description": "A flattened point with additional a coordinate",
            "examples": [
                {
                    "a": -2_147_483_648
                },
                {
                    "2001:db8:3333:4444:5555:6666:7777:8888": "1.2.3.4",
                    "2001:db8:3333:4444:cccc:dddd:eeee:ffff": "101.102.103.104",
                    "a": -1
                },
                {
                    "2001:db8:3333:4444:5555:6666:7777:8888": "1.2.3.4",
                    "2001:db8:3333:4444:cccc:dddd:eeee:ffff": "101.102.103.104",
                    "a": 0
                }
            ],
            "patternProperties": {
                "^(?:(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,7}:|(?:[0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,5}(?::[0-9a-fA-F]{1,4}){1,2}|(?:[0-9a-fA-F]{1,4}:){1,4}(?::[0-9a-fA-F]{1,4}){1,3}|(?:[0-9a-fA-F]{1,4}:){1,3}(?::[0-9a-fA-F]{1,4}){1,4}|(?:[0-9a-fA-F]{1,4}:){1,2}(?::[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:(?:(?::[0-9a-fA-F]{1,4}){1,6})|:(?:(?::[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(?::[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(?:ffff(?::0{1,4}){0,1}:){0,1}(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)|(?:[0-9a-fA-F]{1,4}:){1,4}:(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d))$": {
                    "description": "An IPv4 address according to [IETF RFC 791](https://tools.ietf.org/html/rfc791)",
                    "examples": [
                        "1.2.3.4",
                        "101.102.103.104"
                    ],
                    "format": "ipv4",
                    "maxLength": 15,
                    "minLength": 7,
                    "pattern": "^(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)$",
                    "type": "string"
                }
            },
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
                }
            },
            "required": [
                "a"
            ],
            "type": "object"
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
    let schema = build_schema_with_collection::<Flatten>(
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
                        "a": -2_147_483_648
                    },
                    {
                        "2001:db8:3333:4444:5555:6666:7777:8888": "1.2.3.4",
                        "2001:db8:3333:4444:cccc:dddd:eeee:ffff": "101.102.103.104",
                        "a": -1
                    },
                    {
                        "2001:db8:3333:4444:5555:6666:7777:8888": "1.2.3.4",
                        "2001:db8:3333:4444:cccc:dddd:eeee:ffff": "101.102.103.104",
                        "a": 0
                    }
                ],
                "patternProperties": {
                    "^(?:(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,7}:|(?:[0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,5}(?::[0-9a-fA-F]{1,4}){1,2}|(?:[0-9a-fA-F]{1,4}:){1,4}(?::[0-9a-fA-F]{1,4}){1,3}|(?:[0-9a-fA-F]{1,4}:){1,3}(?::[0-9a-fA-F]{1,4}){1,4}|(?:[0-9a-fA-F]{1,4}:){1,2}(?::[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:(?:(?::[0-9a-fA-F]{1,4}){1,6})|:(?:(?::[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(?::[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(?:ffff(?::0{1,4}){0,1}:){0,1}(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)|(?:[0-9a-fA-F]{1,4}:){1,4}:(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d))$": {
                        "description": "An IPv4 address according to [IETF RFC 791](https://tools.ietf.org/html/rfc791)",
                        "examples": [
                            "1.2.3.4",
                            "101.102.103.104"
                        ],
                        "format": "ipv4",
                        "maxLength": 15,
                        "minLength": 7,
                        "pattern": "^(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)$",
                        "type": "string"
                    }
                },
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
                    }
                },
                "required": [
                    "a"
                ],
                "type": "object"
            }
        })
    );
}
