/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://docs.rs/serde/latest/src/serde/ser/impls.rs.html
 */

use crate::{
    callsite,
    openapi::{
        schema::Schema,
        schema_builder::{FieldMod, SchemaBuilder, SchemaId, StructSchemaBuilder},
    },
};

struct Nanos;

impl Schema for Nanos {
    type Example = u32;
    type Examples = <[Self::Example; 4] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_u32(
            std::ops::Bound::Unbounded,
            std::ops::Bound::Excluded(1_000_000_000),
            None,
            None,
            None,
            Some("Whole milliseconds that describing a subpart of a whole second"),
            || Ok([0, 300, 621, 1_000_000_000 - 1]),
            false,
        )
    }
}

impl Schema for std::time::Duration {
    type Example = Self;
    type Examples = <[Self::Example; 4] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        let mut struct_schema_builder = schema_builder.describe_struct(
            Some(SchemaId::new("Duration", callsite!())),
            2,
            Some("A span of time"),
            || {
                Ok([
                    Self::ZERO,
                    Self::new(0, 621),
                    Self::new(1890, 0),
                    Self::new(1_645_155_669, 300),
                ])
            },
            false,
        )?;
        struct_schema_builder.collect_field(
            "secs",
            FieldMod::ReadWrite,
            None,
            false,
            <u64 as Schema>::describe,
        )?;
        struct_schema_builder.collect_field(
            "nanos",
            FieldMod::ReadWrite,
            None,
            false,
            <Nanos as Schema>::describe,
        )?;
        struct_schema_builder.end()
    }
}

////////////////////////////////////////////////////////////////////////////////

impl Schema for std::time::SystemTime {
    type Example = Self;
    type Examples = <[Self::Example; 2] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        let mut struct_schema_builder = schema_builder.describe_struct(
            Some(SchemaId::new("SystemTime", callsite!())),
            2,
            Some("A measurement of the system clock"),
            || {
                Ok([
                    Self::UNIX_EPOCH,
                    Self::UNIX_EPOCH + std::time::Duration::new(1_645_155_669, 300),
                ])
            },
            false,
        )?;
        struct_schema_builder.collect_field(
            "secs_since_epoch",
            FieldMod::ReadWrite,
            Some("The whole seconds elapsed since UNIX epoch (defined as 1970-01-01 00:00:00 UTC)"),
            false,
            <u64 as Schema>::describe,
        )?;
        struct_schema_builder.collect_field(
            "nanos_since_epoch",
            FieldMod::ReadWrite,
            Some("The additional nanoseconds elapsed since UNIX epoch (defined as 1970-01-01 00:00:00 UTC)"),
            false,
            <Nanos as Schema>::describe,
        )?;
        struct_schema_builder.end()
    }
}

mod test {
    #[test]
    fn test_nanos_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<super::Nanos>(
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
                "type": "integer",
                "minimum": 0,
                "exclusiveMaximum": 1_000_000_000,
                "examples": [0, 300, 621, 999_999_999],
                "description": "Whole milliseconds that describing a subpart of a whole second"
            })
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    fn test_duration_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<std::time::Duration>(
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
                "$ref": "#/components/schemas/Duration"
            })
        );

        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({
                "Duration": {
                    "description": "A span of time",
                    "examples": [
                        {
                            "secs": 0,
                            "nanos": 0,
                        },
                        {
                            "secs": 0,
                            "nanos": 621,
                        },
                        {
                            "secs": 1890,
                            "nanos": 0,
                        },
                        {
                            "secs": 1_645_155_669,
                            "nanos": 300,
                        },
                    ],
                    "properties": {
                        "secs": {
                            "examples": [
                                0,
                                1,
                                18_446_744_073_709_551_615_u64,
                            ],
                            "maximum": 18_446_744_073_709_551_615_u64,
                            "minimum": 0,
                            "type": "integer",
                        },
                        "nanos": {
                            "description": "Whole milliseconds that describing a subpart of a whole second",
                            "examples": [
                                0,
                                300,
                                621,
                                999_999_999,
                            ],
                            "exclusiveMaximum": 1_000_000_000,
                            "minimum": 0,
                            "type": "integer",
                        },
                    },
                    "required": [
                        "nanos",
                        "secs",
                    ],
                    "type": "object",
                },
            })
        );
    }

    #[test]
    fn test_system_time_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<std::time::SystemTime>(
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
                "$ref": "#/components/schemas/SystemTime"
            })
        );

        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({
                "SystemTime": {
                    "description": "A measurement of the system clock",
                    "examples": [
                        {
                            "nanos_since_epoch": 0,
                            "secs_since_epoch": 0,
                        },
                        {
                            "nanos_since_epoch": 300,
                            "secs_since_epoch": 1_645_155_669,
                        },
                    ],
                    "properties": {
                        "nanos_since_epoch": {
                            "description": "The additional nanoseconds elapsed since UNIX epoch (defined as 1970-01-01 00:00:00 UTC)",
                            "examples": [
                                0,
                                300,
                                621,
                                999_999_999,
                            ],
                            "exclusiveMaximum": 1_000_000_000,
                            "minimum": 0,
                            "type": "integer",
                        },
                        "secs_since_epoch": {
                            "description": "The whole seconds elapsed since UNIX epoch (defined as 1970-01-01 00:00:00 UTC)",
                            "examples": [
                                0,
                                1,
                                18_446_744_073_709_551_615_u64,
                            ],
                            "maximum": 18_446_744_073_709_551_615_u64,
                            "minimum": 0,
                            "type": "integer",
                        },
                    },
                    "required": [
                        "nanos_since_epoch",
                        "secs_since_epoch",
                    ],
                    "type": "object",
                },
            })
        );
    }
}
