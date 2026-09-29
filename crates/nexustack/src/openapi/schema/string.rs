/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::openapi::{
    schema::{
        builder::{Combinator, IntoSchemaBuilder, SchemaBuilder, SchemaId, VariantTag},
        error::SchemaGenerationError,
        impossible::Impossible,
    },
    string_serializer::StringSerializer,
};
use std::{borrow::Cow, fmt::Write};

macro_rules! describe_signed_integer {
    ($only:ident, $examples:ident) => {{
        let examples = build_examples($examples)?;
        let pattern = if let Some(only) = $only {
            let mut f = String::new();
            f.push('^');
            f.push('(');

            for (index, variant) in only.iter().enumerate() {
                if index > 0 {
                    f.push('|');
                }

                write!(f, "({variant}([eE][+-]?0+)?)").map_err(SchemaGenerationError::custom)?;
            }

            f.push(')');
            f.push('$');

            Some(Cow::Owned(f))
        } else {
            Some(Cow::Borrowed(r"^(-?(0|[1-9]\d*)([eE][+-]?0+)?)$"))
        };

        Ok(StringSchema { pattern, examples })
    }};
}

macro_rules! describe_unsigned_integer {
    ($only:ident, $examples:ident) => {{
        let examples = build_examples($examples)?;
        let pattern = if let Some(only) = $only {
            let mut f = String::new();
            f.push('^');
            f.push('(');

            for (index, variant) in only.iter().enumerate() {
                if index > 0 {
                    f.push('|');
                }

                write!(f, "({variant}([eE][+-]?0+)?)").map_err(SchemaGenerationError::custom)?;
            }

            f.push(')');
            f.push('$');

            Some(Cow::Owned(f))
        } else {
            Some(Cow::Borrowed(r"^((0|[1-9]\d*)([eE][+-]?0+)?)$"))
        };

        Ok(StringSchema { pattern, examples })
    }};
}

pub(crate) struct StringSchema {
    pub pattern: Option<Cow<'static, str>>,
    pub examples: Vec<String>,
}

pub(crate) struct StringSchemaBuilder;

fn must_be_a_string() -> SchemaGenerationError {
    SchemaGenerationError::custom("must be a string")
}

impl IntoSchemaBuilder for StringSchemaBuilder {
    type MapKey = StringSchema;
    type Ok = StringSchema;
    type Error = SchemaGenerationError;
    type SchemaBuilder<E: Iterator<Item: serde::Serialize + 'static>> = Self;

    fn into_schema_builder<E: Iterator<Item: serde::Serialize + 'static>>(
        self,
    ) -> Self::SchemaBuilder<E> {
        self
    }
}

fn build_examples<I>(
    examples: impl Fn() -> Result<I, SchemaGenerationError>,
) -> Result<Vec<String>, SchemaGenerationError>
where
    I: IntoIterator<IntoIter: Iterator<Item: serde::Serialize + 'static>>,
{
    Ok(examples()?
        .into_iter()
        .map(|example| {
            serde::Serialize::serialize(&example, StringSerializer)
                .map_err(|err| SchemaGenerationError::custom(err))
        })
        .collect::<Result<Vec<_>, _>>()?)
}

impl<E: Iterator<Item: serde::Serialize + 'static>> SchemaBuilder<E> for StringSchemaBuilder {
    type MapKey = StringSchema;
    type Ok = StringSchema;
    type Error = SchemaGenerationError;

    type TupleSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type TupleStructSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type StructSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type CombinatorSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type EnumSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type MapSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type OptionSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type NewtypeStructSchemaBuilder = Self;
    type SeqSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type NotSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;

    fn describe_option<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::OptionSchemaBuilder, Self::Error> {
        Err(must_be_a_string())
    }

    fn describe_bool<I: IntoIterator<IntoIter = E>>(
        self,
        only: Option<bool>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        let examples = build_examples(examples)?;

        if let Some(only) = only {
            if only {
                return Ok(StringSchema {
                    pattern: Some(Cow::Borrowed("^(true)$")),
                    examples,
                });
            }

            return Ok(StringSchema {
                pattern: Some(Cow::Borrowed("^(false)$")),
                examples,
            });
        }

        Ok(StringSchema {
            pattern: Some(Cow::Borrowed("^(true|false)$")),
            examples,
        })
    }

    fn describe_i8<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<i8>,
        _max: std::ops::Bound<i8>,
        _multiple_of: Option<i8>,
        _format: Option<&'static str>,
        only: Option<&'static [i8]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_signed_integer!(only, examples)
    }

    fn describe_i16<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<i16>,
        _max: std::ops::Bound<i16>,
        _multiple_of: Option<i16>,
        _format: Option<&'static str>,
        only: Option<&'static [i16]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_signed_integer!(only, examples)
    }

    fn describe_i32<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<i32>,
        _max: std::ops::Bound<i32>,
        _multiple_of: Option<i32>,
        _format: Option<&'static str>,
        only: Option<&'static [i32]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_signed_integer!(only, examples)
    }

    fn describe_i64<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<i64>,
        _max: std::ops::Bound<i64>,
        _multiple_of: Option<i64>,
        _format: Option<&'static str>,
        only: Option<&'static [i64]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_signed_integer!(only, examples)
    }

    fn describe_u8<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<u8>,
        _max: std::ops::Bound<u8>,
        _multiple_of: Option<u8>,
        _format: Option<&'static str>,
        only: Option<&'static [u8]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_unsigned_integer!(only, examples)
    }

    fn describe_u16<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<u16>,
        _max: std::ops::Bound<u16>,
        _multiple_of: Option<u16>,
        _format: Option<&'static str>,
        only: Option<&'static [u16]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_unsigned_integer!(only, examples)
    }

    fn describe_u32<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<u32>,
        _max: std::ops::Bound<u32>,
        _multiple_of: Option<u32>,
        _format: Option<&'static str>,
        only: Option<&'static [u32]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_unsigned_integer!(only, examples)
    }

    fn describe_u64<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<u64>,
        _max: std::ops::Bound<u64>,
        _multiple_of: Option<u64>,
        _format: Option<&'static str>,
        only: Option<&'static [u64]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_unsigned_integer!(only, examples)
    }

    fn describe_f32<I: IntoIterator<IntoIter = E>>(
        self,
        _allow_nan: bool,
        _allow_inf: bool,
        _min: std::ops::Bound<f32>,
        _max: std::ops::Bound<f32>,
        _format: Option<&'static str>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        let examples = build_examples(examples)?;

        Ok(StringSchema {
            pattern: Some(Cow::Borrowed(r"^(-?(0|[1-9]\d*)(\.\d+)?([eE][+-]?\d+)?)$")),
            examples,
        })
    }

    fn describe_f64<I: IntoIterator<IntoIter = E>>(
        self,
        _allow_nan: bool,
        _allow_inf: bool,
        _min: std::ops::Bound<f64>,
        _max: std::ops::Bound<f64>,
        _format: Option<&'static str>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        let examples = build_examples(examples)?;

        Ok(StringSchema {
            pattern: Some(Cow::Borrowed(r"^(-?(0|[1-9]\d*)(\.\d+)?([eE][+-]?\d+)?)$")),
            examples,
        })
    }

    fn describe_char<I: IntoIterator<IntoIter = E>>(
        self,
        pattern: Option<&'static str>,
        _format: Option<&'static str>,
        only: Option<&'static [char]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        let examples = build_examples(examples)?;
        let pattern = if let Some(only) = only {
            let mut f = String::new();
            f.push('^');
            f.push('(');

            for (index, variant) in only.iter().enumerate() {
                if index > 0 {
                    f.push('|');
                }

                f.push(*variant);
            }

            f.push(')');
            f.push('$');

            Some(Cow::Owned(f))
        } else if let Some(pattern) = pattern {
            Some(Cow::Borrowed(pattern))
        } else {
            Some(Cow::Borrowed("^(.{1})$"))
        };

        Ok(StringSchema { pattern, examples })
    }

    fn describe_str<I: IntoIterator<IntoIter = E>>(
        self,
        min_len: Option<usize>,
        max_len: Option<usize>,
        pattern: Option<&'static str>,
        _format: Option<&'static str>,
        only: Option<&'static [&'static str]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        let examples = build_examples(examples)?;
        let pattern = if let Some(only) = only {
            let mut f = String::new();
            f.push('^');
            f.push('(');

            for (index, variant) in only.iter().enumerate() {
                if index > 0 {
                    f.push('|');
                }

                f.push_str(variant);
            }

            f.push(')');
            f.push('$');

            Some(Cow::Owned(f))
        } else if let Some(pattern) = pattern {
            Some(Cow::Borrowed(pattern))
        } else if let Some(min_len) = min_len {
            if let Some(max_len) = max_len {
                if min_len == max_len {
                    Some(Cow::Owned(format!("^(.{{{min_len}}})$")))
                } else {
                    Some(Cow::Owned(format!("^(.{{{min_len},{max_len}}})$")))
                }
            } else {
                Some(Cow::Owned(format!("^(.{{{min_len},}})$")))
            }
        } else if let Some(max_len) = max_len {
            Some(Cow::Owned(format!("^(.{{,{max_len}}})$")))
        } else {
            None
        };

        Ok(StringSchema { pattern, examples })
    }

    fn describe_bytes<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(must_be_a_string())
    }

    fn describe_unit<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(must_be_a_string())
    }

    fn describe_unit_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(must_be_a_string())
    }

    fn describe_newtype_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::NewtypeStructSchemaBuilder, Self::Error> {
        Ok(self)
    }

    fn describe_seq<I: IntoIterator<IntoIter = E>>(
        self,
        _min_len: Option<usize>,
        _max_len: Option<usize>,
        _unique: bool,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::SeqSchemaBuilder, Self::Error> {
        Err(must_be_a_string())
    }

    fn describe_tuple<I: IntoIterator<IntoIter = E>>(
        self,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::TupleSchemaBuilder, Self::Error> {
        Err(must_be_a_string())
    }

    fn describe_tuple_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::TupleStructSchemaBuilder, Self::Error> {
        Err(must_be_a_string())
    }

    fn describe_map<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::MapSchemaBuilder, Self::Error> {
        Err(must_be_a_string())
    }

    fn describe_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::StructSchemaBuilder, Self::Error> {
        Err(must_be_a_string())
    }

    fn describe_enum<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _len: usize,
        _exhaustive: bool,
        _tag: VariantTag,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::EnumSchemaBuilder, Self::Error> {
        Err(must_be_a_string()) // TODO: Unit variant is ok!
    }

    fn describe_not<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::NotSchemaBuilder, Self::Error> {
        Err(must_be_a_string()) // TODO: Can we implement this?
    }

    fn describe_combinator<I: IntoIterator<IntoIter = E>>(
        self,
        _combinator: Combinator,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::CombinatorSchemaBuilder, Self::Error> {
        Err(must_be_a_string()) // TODO: Can we implement this?
    }
}
