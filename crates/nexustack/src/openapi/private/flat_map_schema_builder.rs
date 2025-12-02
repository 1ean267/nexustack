/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::openapi::{
    error,
    schema::{
        Schema,
        builder::{
            Combinator, FieldMod, IntoSchemaBuilder, MapSchemaBuilder, SchemaBuilder, SchemaId,
            StructSchemaBuilder, VariantTag,
        },
        impossible::Impossible,
    },
};
use serde::Serialize;

#[derive(Clone, Copy)]
enum Unsupported {
    Boolean,
    Integer,
    Float,
    Char,
    String,
    ByteArray,
    Sequence,
    Tuple,
    TupleStruct,
    Enum,
    Not,
    Combinator,
}

impl std::fmt::Display for Unsupported {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Boolean => formatter.write_str("a boolean"),
            Self::Integer => formatter.write_str("an integer"),
            Self::Float => formatter.write_str("a float"),
            Self::Char => formatter.write_str("a char"),
            Self::String => formatter.write_str("a string"),
            Self::ByteArray => formatter.write_str("a byte array"),
            Self::Sequence => formatter.write_str("a sequence"),
            Self::Tuple => formatter.write_str("a tuple"),
            Self::TupleStruct => formatter.write_str("a tuple struct"),
            Self::Enum => formatter.write_str("an enum"),
            Self::Not => formatter.write_str("a not combinator"),
            Self::Combinator => formatter.write_str("a combinator"),
        }
    }
}

//
// Struct
//

pub struct StructFlatMapSchemaBuilder<'a, B>(&'a mut B);

impl<B: MapSchemaBuilder> StructSchemaBuilder for StructFlatMapSchemaBuilder<'_, B> {
    type MapKey = B::MapKey;
    type Ok = ();
    type Error = B::Error;

    type FieldSchemaBuilder<'b>
        = B::MapValueSchemaBuilder<'b>
    where
        Self: 'b;

    fn describe_field<'b>(
        &'b mut self,
        key: &'static str,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'b>, Self::Error> {
        MapSchemaBuilder::describe_element(self.0, key, modifier, description, deprecated)
    }

    fn describe_field_optional<'b, F: Serialize>(
        &'b mut self,
        key: &'static str,
        modifier: FieldMod,
        default: Option<F>,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'b>, Self::Error> {
        MapSchemaBuilder::describe_element_optional(
            self.0,
            key,
            modifier,
            default,
            description,
            deprecated,
        )
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

//
// Map
//

pub struct MapFlatMapSchemaBuilder<'a, B>(&'a mut B);

impl<B: MapSchemaBuilder> MapSchemaBuilder for MapFlatMapSchemaBuilder<'_, B> {
    type MapKey = B::MapKey;
    type Ok = ();
    type Error = B::Error;

    type MapKeySchemaBuilder = B::MapKeySchemaBuilder;
    type MapValueSchemaBuilder<'b>
        = B::MapValueSchemaBuilder<'b>
    where
        Self: 'b;

    fn describe_element<'b, K: Schema + Serialize>(
        &'b mut self,
        key: K,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'b>, Self::Error> {
        MapSchemaBuilder::describe_element(self.0, key, modifier, description, deprecated)
    }

    fn describe_element_optional<'b, K: Schema + Serialize, F: Serialize>(
        &'b mut self,
        key: K,
        modifier: FieldMod,
        default: Option<F>,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'b>, Self::Error> {
        MapSchemaBuilder::describe_element_optional(
            self.0,
            key,
            modifier,
            default,
            description,
            deprecated,
        )
    }

    fn describe_additional_elements<'b, K, I: Iterator<Item: Serialize + 'static>>(
        &'b mut self,
        describe_key: K,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'b>, Self::Error>
    where
        K: FnOnce(
            <Self::MapKeySchemaBuilder as IntoSchemaBuilder>::SchemaBuilder<I>,
        )
            -> Result<<Self::MapKeySchemaBuilder as IntoSchemaBuilder>::Ok, Self::Error>,
    {
        MapSchemaBuilder::describe_additional_elements(
            self.0,
            describe_key,
            description,
            deprecated,
        )
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

//
// Schema
//

pub struct FlatMapSchemaBuilder<'a, B: 'a>(pub &'a mut B);

impl<B: MapSchemaBuilder> IntoSchemaBuilder for FlatMapSchemaBuilder<'_, B> {
    type MapKey = B::MapKey;
    type Ok = ();
    type Error = B::Error;

    type SchemaBuilder<E: Iterator<Item: Serialize + 'static>> = Self;

    fn into_schema_builder<E: Iterator<Item: Serialize + 'static>>(self) -> Self::SchemaBuilder<E> {
        self
    }
}

impl<'b, B> FlatMapSchemaBuilder<'b, B>
where
    B: MapSchemaBuilder + 'b,
{
    fn bad_type(what: Unsupported) -> B::Error {
        error::Error::custom(format_args!(
            "can only flatten structs and maps (got {what})"
        ))
    }
}

impl<'a, B: MapSchemaBuilder, E: Iterator<Item: Serialize + 'static>> SchemaBuilder<E>
    for FlatMapSchemaBuilder<'a, B>
{
    type MapKey = B::MapKey;
    type Ok = ();
    type Error = B::Error;

    type TupleSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type TupleStructSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type StructSchemaBuilder = StructFlatMapSchemaBuilder<'a, B>;
    type CombinatorSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type EnumSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    //type EnumSchemaBuilder = EnumFlatMapSchemaBuilder<'a, B>;
    type MapSchemaBuilder = MapFlatMapSchemaBuilder<'a, B>;

    type OptionSchemaBuilder = Self;
    type NewtypeStructSchemaBuilder = Self;
    type SeqSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type NotSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;

    fn describe_option<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::OptionSchemaBuilder, Self::Error> {
        // TODO: Is this correct? Add map element as optional?
        Ok(self)
    }

    fn describe_bool<I: IntoIterator<IntoIter = E>>(
        self,
        _only: Option<bool>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Boolean))
    }

    fn describe_i8<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<i8>,
        _max: std::ops::Bound<i8>,
        _multiple_of: Option<i8>,
        _format: Option<&'static str>,
        _only: Option<&'static [i8]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Integer))
    }

    fn describe_i16<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<i16>,
        _max: std::ops::Bound<i16>,
        _multiple_of: Option<i16>,
        _format: Option<&'static str>,
        _only: Option<&'static [i16]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Integer))
    }

    fn describe_i32<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<i32>,
        _max: std::ops::Bound<i32>,
        _multiple_of: Option<i32>,
        _format: Option<&'static str>,
        _only: Option<&'static [i32]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Integer))
    }

    fn describe_i64<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<i64>,
        _max: std::ops::Bound<i64>,
        _multiple_of: Option<i64>,
        _format: Option<&'static str>,
        _only: Option<&'static [i64]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Integer))
    }

    fn describe_i128<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<i128>,
        _max: std::ops::Bound<i128>,
        _multiple_of: Option<i128>,
        _format: Option<&'static str>,
        _only: Option<&'static [i128]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Integer))
    }

    fn describe_u8<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<u8>,
        _max: std::ops::Bound<u8>,
        _multiple_of: Option<u8>,
        _format: Option<&'static str>,
        _only: Option<&'static [u8]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Integer))
    }

    fn describe_u16<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<u16>,
        _max: std::ops::Bound<u16>,
        _multiple_of: Option<u16>,
        _format: Option<&'static str>,
        _only: Option<&'static [u16]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Integer))
    }

    fn describe_u32<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<u32>,
        _max: std::ops::Bound<u32>,
        _multiple_of: Option<u32>,
        _format: Option<&'static str>,
        _only: Option<&'static [u32]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Integer))
    }

    fn describe_u64<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<u64>,
        _max: std::ops::Bound<u64>,
        _multiple_of: Option<u64>,
        _format: Option<&'static str>,
        _only: Option<&'static [u64]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Integer))
    }

    fn describe_u128<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<u128>,
        _max: std::ops::Bound<u128>,
        _multiple_of: Option<u128>,
        _format: Option<&'static str>,
        _only: Option<&'static [u128]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Integer))
    }

    fn describe_f32<I: IntoIterator<IntoIter = E>>(
        self,
        _allow_nan: bool,
        _allow_inf: bool,
        _min: std::ops::Bound<f32>,
        _max: std::ops::Bound<f32>,
        _format: Option<&'static str>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Float))
    }

    fn describe_f64<I: IntoIterator<IntoIter = E>>(
        self,
        _allow_nan: bool,
        _allow_inf: bool,
        _min: std::ops::Bound<f64>,
        _max: std::ops::Bound<f64>,
        _format: Option<&'static str>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Float))
    }

    fn describe_char<I: IntoIterator<IntoIter = E>>(
        self,
        _pattern: Option<&'static str>,
        _format: Option<&'static str>,
        _only: Option<&'static [char]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Char))
    }

    fn describe_str<I: IntoIterator<IntoIter = E>>(
        self,
        _min_len: Option<usize>,
        _max_len: Option<usize>,
        _pattern: Option<&'static str>,
        _format: Option<&'static str>,
        _only: Option<&'static [&'static str]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::String))
    }

    fn describe_bytes<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::ByteArray))
    }

    fn describe_unit<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn describe_unit_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(())
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
        Err(Self::bad_type(Unsupported::Sequence))
    }

    fn describe_tuple<I: IntoIterator<IntoIter = E>>(
        self,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::TupleSchemaBuilder, Self::Error> {
        Err(Self::bad_type(Unsupported::Tuple))
    }

    fn describe_tuple_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::TupleStructSchemaBuilder, Self::Error> {
        Err(Self::bad_type(Unsupported::TupleStruct))
    }

    fn describe_map<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::MapSchemaBuilder, Self::Error> {
        Ok(MapFlatMapSchemaBuilder(self.0))
    }

    fn describe_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::StructSchemaBuilder, Self::Error> {
        Ok(StructFlatMapSchemaBuilder(self.0))
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
        // Ok(EnumFlatMapSchemaBuilder(self.0))
        Err(Self::bad_type(Unsupported::Enum))
    }

    fn describe_not<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::NotSchemaBuilder, Self::Error> {
        Err(Self::bad_type(Unsupported::Not))
    }

    fn describe_combinator<I: IntoIterator<IntoIter = E>>(
        self,
        _combinator: Combinator,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::CombinatorSchemaBuilder, Self::Error> {
        Err(Self::bad_type(Unsupported::Combinator))
    }
}
