/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::openapi::{
    error,
    schema::Schema,
    schema_builder::{
        Combinator, CombinatorSchemaBuilder, EnumSchemaBuilder, FieldMod, IntoSchemaBuilder,
        MapSchemaBuilder, SchemaBuilder, SchemaId, StructSchemaBuilder, StructVariantSchemaBuilder,
        TupleSchemaBuilder, TupleStructSchemaBuilder, TupleVariantSchemaBuilder, VariantTag,
    },
};
use serde::Serialize;
use std::marker::PhantomData;

#[non_exhaustive]
enum Void {}

/// A schema builder that represents an impossible or uninhabited type.
///
/// This struct is used as a marker for types that cannot be constructed or described.
/// An instance of the Impossible type is not constructible.
///
/// # Type Arguments
/// * `MapKey` - The type of map keys (unused).
/// * `Ok` - The success type (unused).
/// * `Error` - The error type.
///
/// This is typically used internally to indicate unreachable code paths in schema building.
#[non_exhaustive]
pub struct Impossible<MapKey, Ok, Error> {
    void: Void,
    map_key: PhantomData<MapKey>,
    ok: PhantomData<Ok>,
    error: PhantomData<Error>,
}

//
// Struct
//

impl<MapKey, Ok, Error: error::Error> StructSchemaBuilder for Impossible<MapKey, Ok, Error> {
    type MapKey = MapKey;
    type Ok = Ok;
    type Error = Error;

    type FieldSchemaBuilder<'a>
        = Impossible<MapKey, (), Error>
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        _key: &'static str,
        _modifier: FieldMod,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        match self.void {}
    }

    fn describe_field_optional<'a, F: Serialize>(
        &'a mut self,
        _key: &'static str,
        _modifier: FieldMod,
        _default: Option<F>,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        match self.void {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.void {}
    }
}

//
// Tuple
//

impl<MapKey, Ok, Error: error::Error> TupleSchemaBuilder for Impossible<MapKey, Ok, Error> {
    type MapKey = MapKey;
    type Ok = Ok;
    type Error = Error;

    type ElementSchemaBuilder<'a>
        = Impossible<MapKey, (), Error>
    where
        Self: 'a;

    fn describe_element<'a>(
        &'a mut self,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::ElementSchemaBuilder<'a>, Self::Error> {
        match self.void {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.void {}
    }
}

//
// Tuple struct
//

impl<MapKey, Ok, Error: error::Error> TupleStructSchemaBuilder for Impossible<MapKey, Ok, Error> {
    type MapKey = MapKey;
    type Ok = Ok;
    type Error = Error;

    type FieldSchemaBuilder<'a>
        = Impossible<MapKey, (), Error>
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        match self.void {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.void {}
    }
}

//
// Combinator
//

impl<MapKey, Ok, Error: error::Error> CombinatorSchemaBuilder for Impossible<MapKey, Ok, Error> {
    type MapKey = MapKey;
    type Ok = Ok;
    type Error = Error;

    type SubSchemaBuilder<'a>
        = Impossible<MapKey, (), Error>
    where
        Self: 'a;

    fn describe_subschema<'a>(
        &'a mut self,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::SubSchemaBuilder<'a>, Self::Error> {
        match self.void {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.void {}
    }
}

//
// Map
//

impl<MapKey, Ok, Error: error::Error> MapSchemaBuilder for Impossible<MapKey, Ok, Error> {
    type MapKey = MapKey;
    type Ok = Ok;
    type Error = Error;

    type MapKeySchemaBuilder = Impossible<MapKey, MapKey, Error>;

    type MapValueSchemaBuilder<'a>
        = Impossible<MapKey, (), Error>
    where
        Self: 'a;

    fn describe_element<'a, K: Schema + Serialize>(
        &'a mut self,
        _key: K,
        _modifier: FieldMod,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'a>, Self::Error> {
        match self.void {}
    }

    fn describe_element_optional<'a, K: Schema + Serialize, F: Serialize>(
        &'a mut self,
        _key: K,
        _modifier: FieldMod,
        _default: Option<F>,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'a>, Self::Error> {
        match self.void {}
    }

    fn describe_additional_elements<'a, K, I: Iterator<Item: Serialize + 'static>>(
        &'a mut self,
        _describe_key: K,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'a>, Self::Error>
    where
        K: FnOnce(
            <Self::MapKeySchemaBuilder as IntoSchemaBuilder>::SchemaBuilder<I>,
        )
            -> Result<<Self::MapKeySchemaBuilder as IntoSchemaBuilder>::Ok, Self::Error>,
    {
        match self.void {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.void {}
    }
}

//
// Enum
//

impl<MapKey, Ok, Error: error::Error> StructVariantSchemaBuilder for Impossible<MapKey, Ok, Error> {
    type MapKey = MapKey;
    type Error = Error;

    type FieldSchemaBuilder<'a>
        = Impossible<MapKey, (), Error>
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        _key: &'static str,
        _modifier: FieldMod,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        match self.void {}
    }

    fn describe_field_optional<'a, F: Serialize>(
        &'a mut self,
        _key: &'static str,
        _modifier: FieldMod,
        _default: Option<F>,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        match self.void {}
    }

    fn end(self) -> Result<(), Self::Error> {
        match self.void {}
    }
}

impl<MapKey, Ok, Error: error::Error> TupleVariantSchemaBuilder for Impossible<MapKey, Ok, Error> {
    type MapKey = MapKey;
    type Error = Error;

    type FieldSchemaBuilder<'a>
        = Impossible<MapKey, (), Error>
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        match self.void {}
    }

    fn end(self) -> Result<(), Self::Error> {
        match self.void {}
    }
}

impl<MapKey, Ok, Error: error::Error> EnumSchemaBuilder for Impossible<MapKey, Ok, Error> {
    type MapKey = MapKey;
    type Ok = Ok;
    type Error = Error;

    type TupleVariantSchemaBuilder<'a>
        = Self
    where
        Self: 'a;

    type StructVariantSchemaBuilder<'a>
        = Self
    where
        Self: 'a;

    type NewTypeVariantSchemaBuilder<'a>
        = Impossible<MapKey, (), Error>
    where
        Self: 'a;

    fn describe_unit_variant(
        &mut self,
        _index: u32,
        _id: SchemaId,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<(), Self::Error> {
        match self.void {}
    }

    fn describe_newtype_variant<'a>(
        &'a mut self,
        _index: u32,
        _id: SchemaId,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::NewTypeVariantSchemaBuilder<'a>, Self::Error> {
        match self.void {}
    }

    fn describe_tuple_variant<'a>(
        &'a mut self,
        _index: u32,
        _id: SchemaId,
        _len: usize,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::TupleVariantSchemaBuilder<'a>, Self::Error> {
        match self.void {}
    }

    fn describe_struct_variant<'a>(
        &'a mut self,
        _index: u32,
        _id: SchemaId,
        _len: usize,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::StructVariantSchemaBuilder<'a>, Self::Error> {
        match self.void {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.void {}
    }
}

//
// Schema
//

impl<MapKey, Ok, Error: error::Error> IntoSchemaBuilder for Impossible<MapKey, Ok, Error> {
    type MapKey = MapKey;
    type Ok = Ok;
    type Error = Error;

    type SchemaBuilder<E: Iterator<Item: Serialize + 'static>> = Self;

    fn into_schema_builder<E: Iterator<Item: Serialize + 'static>>(self) -> Self::SchemaBuilder<E> {
        match self.void {}
    }
}

impl<MapKey, Ok, Error: error::Error, E: Iterator<Item: Serialize + 'static>> SchemaBuilder<E>
    for Impossible<MapKey, Ok, Error>
{
    type MapKey = MapKey;
    type Ok = Ok;
    type Error = Error;

    type TupleSchemaBuilder = Self;
    type TupleStructSchemaBuilder = Self;
    type StructSchemaBuilder = Self;
    type CombinatorSchemaBuilder = Self;
    type EnumSchemaBuilder = Self;
    type MapSchemaBuilder = Self;
    type OptionSchemaBuilder = Self;
    type NewtypeStructSchemaBuilder = Self;
    type SeqSchemaBuilder = Self;
    type NotSchemaBuilder = Self;

    fn describe_option<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::OptionSchemaBuilder, Self::Error> {
        match self.void {}
    }

    fn describe_bool<I: IntoIterator<IntoIter = E>>(
        self,
        _only: Option<bool>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self.void {}
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
        match self.void {}
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
        match self.void {}
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
        match self.void {}
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
        match self.void {}
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
        match self.void {}
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
        match self.void {}
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
        match self.void {}
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
        match self.void {}
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
        match self.void {}
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
        match self.void {}
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
        match self.void {}
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
        match self.void {}
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
        match self.void {}
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
        match self.void {}
    }

    fn describe_bytes<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self.void {}
    }

    fn describe_unit<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self.void {}
    }

    fn describe_unit_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self.void {}
    }

    fn describe_newtype_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::NewtypeStructSchemaBuilder, Self::Error> {
        match self.void {}
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
        match self.void {}
    }

    fn describe_tuple<I: IntoIterator<IntoIter = E>>(
        self,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::TupleSchemaBuilder, Self::Error> {
        match self.void {}
    }

    fn describe_tuple_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::TupleStructSchemaBuilder, Self::Error> {
        match self.void {}
    }

    fn describe_map<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::MapSchemaBuilder, Self::Error> {
        match self.void {}
    }

    fn describe_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::StructSchemaBuilder, Self::Error> {
        match self.void {}
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
        match self.void {}
    }

    fn describe_not<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::NotSchemaBuilder, Self::Error> {
        match self.void {}
    }

    fn describe_combinator<I: IntoIterator<IntoIter = E>>(
        self,
        _combinator: Combinator,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::CombinatorSchemaBuilder, Self::Error> {
        match self.void {}
    }
}
