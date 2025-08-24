/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://docs.rs/serde/latest/src/serde/ser/impls.rs.html
 */

use crate::openapi::{
    example::SchemaExamples,
    schema::Schema,
    schema_builder::{SchemaBuilder, TupleSchemaBuilder},
};

macro_rules! tuple_impls {
    ($($len:expr => ($($name:ident)+))+) => {
        $(
            #[cfg_attr(docsrs, doc(hidden))]
            impl<$($name),+> Schema for ($($name,)+)
            where
                $($name: Schema,)+
            {
                tuple_impl_body!($len => ($($name)+));
            }
        )+
    };
}

macro_rules! tuple_impl_body {
    ($len:expr => ($($name:ident)+)) => {
        type Example = ($(<$name as Schema>::Example,)+);
        type Examples = tuple_examples_type!($($name,)+);

        #[inline]
        #[allow(non_snake_case)]
        fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
        where
            B: SchemaBuilder<Self::Examples>,
        {
            let is_human_readable = schema_builder.is_human_readable();
            let mut tuple_schema_builder = schema_builder.describe_tuple(
                $len,
                None,
                || Ok(tuple_examples!(is_human_readable, $($name,)+)),
                false,
            )?;

            $(
                TupleSchemaBuilder::collect_element(
                    &mut tuple_schema_builder,
                    None,
                    false,
                    $name::describe,
                )?;
            )+

            tuple_schema_builder.end()
        }
    };
}

macro_rules! tuple_examples {
    ($is_human_readable:ident, $name:ident $(,)?) => {
        <$name as SchemaExamples>::examples($is_human_readable)?.map((|e| (e,)) as _)
    };
    ($is_human_readable:ident, $additional:ident, $($name:ident),+ $(,)?) => {
        tuple_examples!($is_human_readable, $($name,)+)
            .zip(<$additional as SchemaExamples>::examples($is_human_readable)?)
            .map((|(($($name,)+), $additional)| ($additional, $($name,)+)) as _)
    };
}

macro_rules! tuple_examples_type {
    ($name:ident $(,)?) => {
        std::iter::Map<<$name as Schema>::Examples, fn (<$name as Schema>::Example) -> (<$name as Schema>::Example, )>
    };
    ($additional:ident, $($name:ident),+ $(,)?) => {
        std::iter::Map<
            std::iter::Zip<
                tuple_examples_type!($($name,)+),
                <$additional as Schema>::Examples,
            >,
            fn((($(<$name as Schema>::Example,)+), <$additional as Schema>::Example)) -> (<$additional as Schema>::Example, $(<$name as Schema>::Example,)+)
        >
    };
}

#[cfg_attr(docsrs, doc(fake_variadic))]
#[cfg_attr(
    docsrs,
    doc = "This trait is implemented for tuples up to 16 items long."
)]
impl<T> Schema for (T,)
where
    T: Schema,
{
    tuple_impl_body!(1 => (T));
}

tuple_impls! {
    2 => (T0 T1)
    3 => (T0 T1 T2)
    4 => (T0 T1 T2 T3)
    5 => (T0 T1 T2 T3 T4)
    6 => (T0 T1 T2 T3 T4 T5)
    7 => (T0 T1 T2 T3 T4 T5 T6)
    8 => (T0 T1 T2 T3 T4 T5 T6 T7)
    9 => (T0 T1 T2 T3 T4 T5 T6 T7 T8)
    10 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9)
    11 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10)
    12 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11)
    13 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12)
    14 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13)
    15 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14)
    16 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15)
}

mod test {
    #[test]
    fn test_tuple_1_i32_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<(i32,)>(
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
                "examples": [
                    [
                        -2_147_483_648,
                    ],
                    [
                        -1,
                    ],
                    [
                        0,
                    ],
                    [
                        1,
                    ],
                    [
                        2_147_483_647,
                    ],
                ],
                "maxItems": 1,
                "minItems": 1,
                "prefixItems": [
                {
                        "examples": [
                            -2_147_483_648,
                            -1,
                            0,
                            1,
                            2_147_483_647,
                        ],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                ],
                "type": "array",
            })
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    fn test_tuple_2_i32_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<(i32, i32)>(
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
                "examples": [
                    [
                        -2_147_483_648,
                        -2_147_483_648,
                    ],
                    [
                        -1,
                        -1,
                    ],
                    [
                        0,
                        0,
                    ],
                    [
                        1,
                        1,
                    ],
                    [
                        2_147_483_647,
                        2_147_483_647,
                    ],
                ],
                "maxItems": 2,
                "minItems": 2,
                "prefixItems": [
                    {
                        "examples": [
                            -2_147_483_648,
                            -1,
                            0,
                            1,
                            2_147_483_647,
                        ],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [
                            -2_147_483_648,
                            -1,
                            0,
                            1,
                            2_147_483_647,
                        ],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                ],
                "type": "array",
            })
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    fn test_tuple_3_i32_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<(i32, i32, i32)>(
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
                "examples": [
                    [-2_147_483_648, -2_147_483_648, -2_147_483_648],
                    [-1, -1, -1],
                    [0, 0, 0],
                    [1, 1, 1],
                    [2_147_483_647, 2_147_483_647, 2_147_483_647],
                ],
                "maxItems": 3,
                "minItems": 3,
                "prefixItems": [
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                ],
                "type": "array",
            })
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    fn test_tuple_4_i32_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<(i32, i32, i32, i32)>(
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
                "examples": [
                    [-2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648],
                    [-1, -1, -1, -1],
                    [0, 0, 0, 0],
                    [1, 1, 1, 1],
                    [2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647],
                ],
                "maxItems": 4,
                "minItems": 4,
                "prefixItems": [
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                ],
                "type": "array",
            })
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    fn test_tuple_5_i32_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<(i32, i32, i32, i32, i32)>(
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
                "examples": [
                    [-2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648],
                    [-1, -1, -1, -1, -1],
                    [0, 0, 0, 0, 0],
                    [1, 1, 1, 1, 1],
                    [2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647],
                ],
                "maxItems": 5,
                "minItems": 5,
                "prefixItems": [
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                ],
                "type": "array",
            })
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    fn test_tuple_6_i32_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<(i32, i32, i32, i32, i32, i32)>(
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
                "examples": [
                    [-2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648],
                    [-1, -1, -1, -1, -1, -1],
                    [0, 0, 0, 0, 0, 0],
                    [1, 1, 1, 1, 1, 1],
                    [2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647],
                ],
                "maxItems": 6,
                "minItems": 6,
                "prefixItems": [
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                ],
                "type": "array",
            })
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    fn test_tuple_7_i32_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<(i32, i32, i32, i32, i32, i32, i32)>(
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
                "examples": [
                    [-2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648],
                    [-1, -1, -1, -1, -1, -1, -1],
                    [0, 0, 0, 0, 0, 0, 0],
                    [1, 1, 1, 1, 1, 1, 1],
                    [2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647],
                ],
                "maxItems": 7,
                "minItems": 7,
                "prefixItems": [
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                     {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                ],
                "type": "array",
            })
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    fn test_tuple_8_i32_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<(i32, i32, i32, i32, i32, i32, i32, i32)>(
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
                "examples": [
                    [-2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648],
                    [-1, -1, -1, -1, -1, -1, -1, -1],
                    [0, 0, 0, 0, 0, 0, 0, 0],
                    [1, 1, 1, 1, 1, 1, 1, 1],
                    [2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647],
                ],
                "maxItems": 8,
                "minItems": 8,
                "prefixItems": [
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                ],
                "type": "array",
            })
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    fn test_tuple_9_i32_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<(i32, i32, i32, i32, i32, i32, i32, i32, i32)>(
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
                "examples": [
                    [-2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648],
                    [-1, -1, -1, -1, -1, -1, -1, -1, -1],
                    [0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [1, 1, 1, 1, 1, 1, 1, 1, 1],
                    [2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647],
                ],
                "maxItems": 9,
                "minItems": 9,
                "prefixItems": [
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                ],
                "type": "array",
            })
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    fn test_tuple_10_i32_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema =
            build_schema_with_collection::<(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32)>(
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
                "examples": [
                    [-2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648],
                    [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
                    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                    [2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647],
                ],
                "maxItems": 10,
                "minItems": 10,
                "prefixItems": [
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                     {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                     {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                ],
                "type": "array",
            })
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_tuple_11_i32_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<(
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
        )>(Specification::OpenAPI3_1, schema_collection.clone())
        .unwrap();

        let schemas_object = Rc::try_unwrap(schema_collection)
            .map_err(|_| "Should be the only Rc strong reference")
            .unwrap()
            .into_inner()
            .to_schemas_object();

        pretty_assertions::assert_eq!(
            serde_json::to_value(schema).unwrap(),
            serde_json::json!({
                "examples": [
                    [-2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648],
                    [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
                    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                    [2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647],
                ],
                "maxItems": 11,
                "minItems": 11,
                "prefixItems": [
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                ],
                "type": "array",
            })
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_tuple_12_i32_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<(
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
        )>(Specification::OpenAPI3_1, schema_collection.clone())
        .unwrap();

        let schemas_object = Rc::try_unwrap(schema_collection)
            .map_err(|_| "Should be the only Rc strong reference")
            .unwrap()
            .into_inner()
            .to_schemas_object();

        pretty_assertions::assert_eq!(
            serde_json::to_value(schema).unwrap(),
            serde_json::json!({
                "examples": [
                    [-2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648],
                    [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
                    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                    [2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647],
                ],
                "maxItems": 12,
                "minItems": 12,
                "prefixItems": [
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                ],
                "type": "array",
            })
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_tuple_13_i32_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<(
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
        )>(Specification::OpenAPI3_1, schema_collection.clone())
        .unwrap();

        let schemas_object = Rc::try_unwrap(schema_collection)
            .map_err(|_| "Should be the only Rc strong reference")
            .unwrap()
            .into_inner()
            .to_schemas_object();

        pretty_assertions::assert_eq!(
            serde_json::to_value(schema).unwrap(),
            serde_json::json!({
                "examples": [
                    [-2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648],
                    [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
                    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                    [2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647],
                ],
                "maxItems": 13,
                "minItems": 13,
                "prefixItems": [
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                ],
                "type": "array",
            })
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_tuple_14_i32_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<(
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
        )>(Specification::OpenAPI3_1, schema_collection.clone())
        .unwrap();

        let schemas_object = Rc::try_unwrap(schema_collection)
            .map_err(|_| "Should be the only Rc strong reference")
            .unwrap()
            .into_inner()
            .to_schemas_object();

        pretty_assertions::assert_eq!(
            serde_json::to_value(schema).unwrap(),
            serde_json::json!({
                "examples": [
                    [-2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648],
                    [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
                    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                    [2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647],
                ],
                "maxItems": 14,
                "minItems": 14,
                "prefixItems": [
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                     {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                     {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                     {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                ],
                "type": "array",
            })
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_tuple_15_i32_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<(
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
        )>(Specification::OpenAPI3_1, schema_collection.clone())
        .unwrap();

        let schemas_object = Rc::try_unwrap(schema_collection)
            .map_err(|_| "Should be the only Rc strong reference")
            .unwrap()
            .into_inner()
            .to_schemas_object();

        pretty_assertions::assert_eq!(
            serde_json::to_value(schema).unwrap(),
            serde_json::json!({
                "examples": [
                    [-2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648],
                    [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
                    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                    [2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647],
                ],
                "maxItems": 15,
                "minItems": 15,
                "prefixItems": [
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                ],
                "type": "array",
            })
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_tuple_16_i32_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<(
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
        )>(Specification::OpenAPI3_1, schema_collection.clone())
        .unwrap();

        let schemas_object = Rc::try_unwrap(schema_collection)
            .map_err(|_| "Should be the only Rc strong reference")
            .unwrap()
            .into_inner()
            .to_schemas_object();

        pretty_assertions::assert_eq!(
            serde_json::to_value(schema).unwrap(),
            serde_json::json!({
                "examples": [
                    [-2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648, -2_147_483_648],
                    [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
                    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                    [2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647, 2_147_483_647],
                ],
                "maxItems": 16,
                "minItems": 16,
                "prefixItems": [
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                    {
                        "examples": [-2_147_483_648, -1, 0, 1, 2_147_483_647],
                        "maximum": 2_147_483_647,
                        "minimum": -2_147_483_648,
                        "type": "integer",
                    },
                ],
                "type": "array",
            })
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }
}
