/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

//
// Struct
//

use crate::openapi::{
    error,
    schema::{
        Schema,
        builder::{
            Combinator, CombinatorSchemaBuilder, EnumSchemaBuilder, FieldMod, IntoSchemaBuilder,
            MapSchemaBuilder, SchemaBuilder, SchemaId, StructSchemaBuilder,
            StructVariantSchemaBuilder, TupleSchemaBuilder, TupleStructSchemaBuilder,
            TupleVariantSchemaBuilder, VariantTag,
        },
    },
};
use either::Either;

impl<M, O, E, L, R> StructSchemaBuilder for either::Either<L, R>
where
    E: error::Error,
    L: StructSchemaBuilder<MapKey = M, Ok = O, Error = E>,
    R: StructSchemaBuilder<MapKey = M, Ok = O, Error = E>,
{
    type MapKey = M;
    type Ok = O;
    type Error = E;

    type FieldSchemaBuilder<'a>
        = Either<L::FieldSchemaBuilder<'a>, R::FieldSchemaBuilder<'a>>
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_field(
                key,
                modifier,
                description,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_field(
                key,
                modifier,
                description,
                deprecated,
            )?)),
        }
    }

    fn describe_field_optional<'a, F: serde::Serialize>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        default: Option<F>,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_field_optional(
                key,
                modifier,
                default,
                description,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_field_optional(
                key,
                modifier,
                default,
                description,
                deprecated,
            )?)),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.end(),
            Self::Right(right) => right.end(),
        }
    }
}

//
// Tuple
//

impl<M, O, E, L, R> TupleSchemaBuilder for either::Either<L, R>
where
    E: error::Error,
    L: TupleSchemaBuilder<MapKey = M, Ok = O, Error = E>,
    R: TupleSchemaBuilder<MapKey = M, Ok = O, Error = E>,
{
    type MapKey = M;
    type Ok = O;
    type Error = E;

    type ElementSchemaBuilder<'a>
        = Either<L::ElementSchemaBuilder<'a>, R::ElementSchemaBuilder<'a>>
    where
        Self: 'a;

    fn describe_element<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::ElementSchemaBuilder<'a>, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(
                left.describe_element(description, deprecated)?,
            )),
            Self::Right(right) => Ok(Either::Right(
                right.describe_element(description, deprecated)?,
            )),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.end(),
            Self::Right(right) => right.end(),
        }
    }
}

//
// Tuple struct
//

impl<M, O, E, L, R> TupleStructSchemaBuilder for either::Either<L, R>
where
    E: error::Error,
    L: TupleStructSchemaBuilder<MapKey = M, Ok = O, Error = E>,
    R: TupleStructSchemaBuilder<MapKey = M, Ok = O, Error = E>,
{
    type MapKey = M;
    type Ok = O;
    type Error = E;

    type FieldSchemaBuilder<'a>
        = Either<L::FieldSchemaBuilder<'a>, R::FieldSchemaBuilder<'a>>
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_field(description, deprecated)?)),
            Self::Right(right) => Ok(Either::Right(
                right.describe_field(description, deprecated)?,
            )),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.end(),
            Self::Right(right) => right.end(),
        }
    }
}

//
// Combinator
//

impl<M, O, E, L, R> CombinatorSchemaBuilder for either::Either<L, R>
where
    E: error::Error,
    L: CombinatorSchemaBuilder<MapKey = M, Ok = O, Error = E>,
    R: CombinatorSchemaBuilder<MapKey = M, Ok = O, Error = E>,
{
    type MapKey = M;
    type Ok = O;
    type Error = E;

    type SubSchemaBuilder<'a>
        = Either<L::SubSchemaBuilder<'a>, R::SubSchemaBuilder<'a>>
    where
        Self: 'a;

    fn describe_subschema<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::SubSchemaBuilder<'a>, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(
                left.describe_subschema(description, deprecated)?,
            )),
            Self::Right(right) => Ok(Either::Right(
                right.describe_subschema(description, deprecated)?,
            )),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.end(),
            Self::Right(right) => right.end(),
        }
    }
}

//
// Map
//

impl<M, O, E, L, R> MapSchemaBuilder for either::Either<L, R>
where
    E: error::Error,
    L: MapSchemaBuilder<MapKey = M, Ok = O, Error = E>,
    R: MapSchemaBuilder<MapKey = M, Ok = O, Error = E>,
{
    type MapKey = M;
    type Ok = O;
    type Error = E;

    type MapKeySchemaBuilder = Either<L::MapKeySchemaBuilder, R::MapKeySchemaBuilder>;

    type MapValueSchemaBuilder<'a>
        = Either<L::MapValueSchemaBuilder<'a>, R::MapValueSchemaBuilder<'a>>
    where
        Self: 'a;

    fn describe_element<'a, K: Schema + serde::Serialize>(
        &'a mut self,
        key: K,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'a>, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_element(
                key,
                modifier,
                description,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_element(
                key,
                modifier,
                description,
                deprecated,
            )?)),
        }
    }

    fn describe_element_optional<'a, K: Schema + serde::Serialize, F: serde::Serialize>(
        &'a mut self,
        key: K,
        modifier: FieldMod,
        default: Option<F>,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'a>, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_element_optional(
                key,
                modifier,
                default,
                description,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_element_optional(
                key,
                modifier,
                default,
                description,
                deprecated,
            )?)),
        }
    }

    fn describe_additional_elements<'a, K, I: Iterator<Item: serde::Serialize + 'static>>(
        &'a mut self,
        describe_key: K,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'a>, Self::Error>
    where
        K: FnOnce(
            <Self::MapKeySchemaBuilder as IntoSchemaBuilder>::SchemaBuilder<I>,
        )
            -> Result<<Self::MapKeySchemaBuilder as IntoSchemaBuilder>::Ok, Self::Error>,
    {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_additional_elements(
                |schema_builder| describe_key(Either::Left(schema_builder)),
                description,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_additional_elements(
                |schema_builder| describe_key(Either::Right(schema_builder)),
                description,
                deprecated,
            )?)),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.end(),
            Self::Right(right) => right.end(),
        }
    }
}

//
// Enum
//

impl<M, E, L, R> StructVariantSchemaBuilder for either::Either<L, R>
where
    E: error::Error,
    L: StructVariantSchemaBuilder<MapKey = M, Error = E>,
    R: StructVariantSchemaBuilder<MapKey = M, Error = E>,
{
    type MapKey = M;
    type Error = E;

    type FieldSchemaBuilder<'a>
        = Either<L::FieldSchemaBuilder<'a>, R::FieldSchemaBuilder<'a>>
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_field(
                key,
                modifier,
                description,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_field(
                key,
                modifier,
                description,
                deprecated,
            )?)),
        }
    }

    fn describe_field_optional<'a, F: serde::Serialize>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        default: Option<F>,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_field_optional(
                key,
                modifier,
                default,
                description,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_field_optional(
                key,
                modifier,
                default,
                description,
                deprecated,
            )?)),
        }
    }

    fn end(self) -> Result<(), Self::Error> {
        match self {
            Self::Left(left) => left.end(),
            Self::Right(right) => right.end(),
        }
    }
}

impl<M, E, L, R> TupleVariantSchemaBuilder for either::Either<L, R>
where
    E: error::Error,
    L: TupleVariantSchemaBuilder<MapKey = M, Error = E>,
    R: TupleVariantSchemaBuilder<MapKey = M, Error = E>,
{
    type MapKey = M;
    type Error = E;

    type FieldSchemaBuilder<'a>
        = Either<L::FieldSchemaBuilder<'a>, R::FieldSchemaBuilder<'a>>
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_field(description, deprecated)?)),
            Self::Right(right) => Ok(Either::Right(
                right.describe_field(description, deprecated)?,
            )),
        }
    }

    fn end(self) -> Result<(), Self::Error> {
        match self {
            Self::Left(left) => left.end(),
            Self::Right(right) => right.end(),
        }
    }
}

impl<M, O, E, L, R> EnumSchemaBuilder for either::Either<L, R>
where
    E: error::Error,
    L: EnumSchemaBuilder<MapKey = M, Ok = O, Error = E>,
    R: EnumSchemaBuilder<MapKey = M, Ok = O, Error = E>,
{
    type MapKey = M;
    type Ok = O;
    type Error = E;

    type TupleVariantSchemaBuilder<'a>
        = Either<L::TupleVariantSchemaBuilder<'a>, R::TupleVariantSchemaBuilder<'a>>
    where
        Self: 'a;

    type StructVariantSchemaBuilder<'a>
        = Either<L::StructVariantSchemaBuilder<'a>, R::StructVariantSchemaBuilder<'a>>
    where
        Self: 'a;

    type NewTypeVariantSchemaBuilder<'a>
        = Either<L::NewTypeVariantSchemaBuilder<'a>, R::NewTypeVariantSchemaBuilder<'a>>
    where
        Self: 'a;

    fn describe_unit_variant(
        &mut self,
        index: u32,
        id: SchemaId, // TODO: Replace with name: &'static str
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<(), Self::Error> {
        match self {
            Self::Left(left) => left.describe_unit_variant(index, id, description, deprecated),
            Self::Right(right) => right.describe_unit_variant(index, id, description, deprecated),
        }
    }

    fn describe_newtype_variant<'a>(
        &'a mut self,
        index: u32,
        id: SchemaId, // TODO: Replace with name: &'static str
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::NewTypeVariantSchemaBuilder<'a>, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_newtype_variant(
                index,
                id,
                description,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_newtype_variant(
                index,
                id,
                description,
                deprecated,
            )?)),
        }
    }

    fn describe_tuple_variant<'a>(
        &'a mut self,
        index: u32,
        id: SchemaId, // TODO: Replace with name: &'static str
        len: usize,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::TupleVariantSchemaBuilder<'a>, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_tuple_variant(
                index,
                id,
                len,
                description,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_tuple_variant(
                index,
                id,
                len,
                description,
                deprecated,
            )?)),
        }
    }

    fn describe_struct_variant<'a>(
        &'a mut self,
        index: u32,
        id: SchemaId, // TODO: Replace with name: &'static str
        len: usize,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::StructVariantSchemaBuilder<'a>, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_struct_variant(
                index,
                id,
                len,
                description,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_struct_variant(
                index,
                id,
                len,
                description,
                deprecated,
            )?)),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.end(),
            Self::Right(right) => right.end(),
        }
    }
}

//
// Schema
//

impl<M, O, E, L, R> IntoSchemaBuilder for either::Either<L, R>
where
    E: error::Error,
    L: IntoSchemaBuilder<MapKey = M, Ok = O, Error = E>,
    R: IntoSchemaBuilder<MapKey = M, Ok = O, Error = E>,
{
    type MapKey = M;
    type Ok = O;
    type Error = E;

    type SchemaBuilder<Ex: Iterator<Item: serde::Serialize + 'static>> =
        Either<L::SchemaBuilder<Ex>, R::SchemaBuilder<Ex>>;

    fn into_schema_builder<Ex: Iterator<Item: serde::Serialize + 'static>>(
        self,
    ) -> Self::SchemaBuilder<Ex> {
        match self {
            Self::Left(left) => Either::Left(left.into_schema_builder()),
            Self::Right(right) => Either::Right(right.into_schema_builder()),
        }
    }
}

impl<M, O, E, L, R, Ex> SchemaBuilder<Ex> for either::Either<L, R>
where
    E: error::Error,
    L: SchemaBuilder<Ex, MapKey = M, Ok = O, Error = E>,
    R: SchemaBuilder<Ex, MapKey = M, Ok = O, Error = E>,
    Ex: Iterator<Item: serde::Serialize + 'static>,
{
    type MapKey = M;
    type Ok = O;
    type Error = E;

    type TupleSchemaBuilder = Either<L::TupleSchemaBuilder, R::TupleSchemaBuilder>;
    type TupleStructSchemaBuilder =
        Either<L::TupleStructSchemaBuilder, R::TupleStructSchemaBuilder>;
    type StructSchemaBuilder = Either<L::StructSchemaBuilder, R::StructSchemaBuilder>;
    type CombinatorSchemaBuilder = Either<L::CombinatorSchemaBuilder, R::CombinatorSchemaBuilder>;
    type EnumSchemaBuilder = Either<L::EnumSchemaBuilder, R::EnumSchemaBuilder>;
    type MapSchemaBuilder = Either<L::MapSchemaBuilder, R::MapSchemaBuilder>;
    type OptionSchemaBuilder = Either<L::OptionSchemaBuilder, R::OptionSchemaBuilder>;
    type NewtypeStructSchemaBuilder =
        Either<L::NewtypeStructSchemaBuilder, R::NewtypeStructSchemaBuilder>;
    type SeqSchemaBuilder = Either<L::SeqSchemaBuilder, R::SeqSchemaBuilder>;
    type NotSchemaBuilder = Either<L::NotSchemaBuilder, R::NotSchemaBuilder>;

    fn describe_option<I: IntoIterator<IntoIter = Ex>>(
        self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::OptionSchemaBuilder, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_option(
                description,
                examples,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_option(
                description,
                examples,
                deprecated,
            )?)),
        }
    }

    fn describe_bool<I: IntoIterator<IntoIter = Ex>>(
        self,
        only: Option<bool>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.describe_bool(only, description, examples, deprecated),
            Self::Right(right) => right.describe_bool(only, description, examples, deprecated),
        }
    }

    fn describe_i8<I: IntoIterator<IntoIter = Ex>>(
        self,
        min: std::ops::Bound<i8>,
        max: std::ops::Bound<i8>,
        multiple_of: Option<i8>,
        format: Option<&'static str>,
        only: Option<&'static [i8]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.describe_i8(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
            Self::Right(right) => right.describe_i8(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
        }
    }

    fn describe_i16<I: IntoIterator<IntoIter = Ex>>(
        self,
        min: std::ops::Bound<i16>,
        max: std::ops::Bound<i16>,
        multiple_of: Option<i16>,
        format: Option<&'static str>,
        only: Option<&'static [i16]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.describe_i16(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
            Self::Right(right) => right.describe_i16(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
        }
    }

    fn describe_i32<I: IntoIterator<IntoIter = Ex>>(
        self,
        min: std::ops::Bound<i32>,
        max: std::ops::Bound<i32>,
        multiple_of: Option<i32>,
        format: Option<&'static str>,
        only: Option<&'static [i32]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.describe_i32(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
            Self::Right(right) => right.describe_i32(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
        }
    }

    fn describe_i64<I: IntoIterator<IntoIter = Ex>>(
        self,
        min: std::ops::Bound<i64>,
        max: std::ops::Bound<i64>,
        multiple_of: Option<i64>,
        format: Option<&'static str>,
        only: Option<&'static [i64]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.describe_i64(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
            Self::Right(right) => right.describe_i64(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
        }
    }

    fn describe_i128<I: IntoIterator<IntoIter = Ex>>(
        self,
        min: std::ops::Bound<i128>,
        max: std::ops::Bound<i128>,
        multiple_of: Option<i128>,
        format: Option<&'static str>,
        only: Option<&'static [i128]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.describe_i128(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
            Self::Right(right) => right.describe_i128(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
        }
    }

    fn describe_u8<I: IntoIterator<IntoIter = Ex>>(
        self,
        min: std::ops::Bound<u8>,
        max: std::ops::Bound<u8>,
        multiple_of: Option<u8>,
        format: Option<&'static str>,
        only: Option<&'static [u8]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.describe_u8(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
            Self::Right(right) => right.describe_u8(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
        }
    }

    fn describe_u16<I: IntoIterator<IntoIter = Ex>>(
        self,
        min: std::ops::Bound<u16>,
        max: std::ops::Bound<u16>,
        multiple_of: Option<u16>,
        format: Option<&'static str>,
        only: Option<&'static [u16]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.describe_u16(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
            Self::Right(right) => right.describe_u16(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
        }
    }

    fn describe_u32<I: IntoIterator<IntoIter = Ex>>(
        self,
        min: std::ops::Bound<u32>,
        max: std::ops::Bound<u32>,
        multiple_of: Option<u32>,
        format: Option<&'static str>,
        only: Option<&'static [u32]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.describe_u32(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
            Self::Right(right) => right.describe_u32(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
        }
    }

    fn describe_u64<I: IntoIterator<IntoIter = Ex>>(
        self,
        min: std::ops::Bound<u64>,
        max: std::ops::Bound<u64>,
        multiple_of: Option<u64>,
        format: Option<&'static str>,
        only: Option<&'static [u64]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.describe_u64(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
            Self::Right(right) => right.describe_u64(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
        }
    }

    fn describe_u128<I: IntoIterator<IntoIter = Ex>>(
        self,
        min: std::ops::Bound<u128>,
        max: std::ops::Bound<u128>,
        multiple_of: Option<u128>,
        format: Option<&'static str>,
        only: Option<&'static [u128]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.describe_u128(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
            Self::Right(right) => right.describe_u128(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
        }
    }

    fn describe_f32<I: IntoIterator<IntoIter = Ex>>(
        self,
        allow_nan: bool,
        allow_inf: bool,
        min: std::ops::Bound<f32>,
        max: std::ops::Bound<f32>,
        format: Option<&'static str>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.describe_f32(
                allow_nan,
                allow_inf,
                min,
                max,
                format,
                description,
                examples,
                deprecated,
            ),
            Self::Right(right) => right.describe_f32(
                allow_nan,
                allow_inf,
                min,
                max,
                format,
                description,
                examples,
                deprecated,
            ),
        }
    }

    fn describe_f64<I: IntoIterator<IntoIter = Ex>>(
        self,
        allow_nan: bool,
        allow_inf: bool,
        min: std::ops::Bound<f64>,
        max: std::ops::Bound<f64>,
        format: Option<&'static str>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.describe_f64(
                allow_nan,
                allow_inf,
                min,
                max,
                format,
                description,
                examples,
                deprecated,
            ),
            Self::Right(right) => right.describe_f64(
                allow_nan,
                allow_inf,
                min,
                max,
                format,
                description,
                examples,
                deprecated,
            ),
        }
    }

    fn describe_char<I: IntoIterator<IntoIter = Ex>>(
        self,
        pattern: Option<&'static str>,
        format: Option<&'static str>,
        only: Option<&'static [char]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => {
                left.describe_char(pattern, format, only, description, examples, deprecated)
            }
            Self::Right(right) => {
                right.describe_char(pattern, format, only, description, examples, deprecated)
            }
        }
    }

    fn describe_str<I: IntoIterator<IntoIter = Ex>>(
        self,
        min_len: Option<usize>,
        max_len: Option<usize>,
        pattern: Option<&'static str>,
        format: Option<&'static str>,
        only: Option<&'static [&'static str]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.describe_str(
                min_len,
                max_len,
                pattern,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
            Self::Right(right) => right.describe_str(
                min_len,
                max_len,
                pattern,
                format,
                only,
                description,
                examples,
                deprecated,
            ),
        }
    }

    fn describe_bytes<I: IntoIterator<IntoIter = Ex>>(
        self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.describe_bytes(description, examples, deprecated),
            Self::Right(right) => right.describe_bytes(description, examples, deprecated),
        }
    }

    fn describe_unit<I: IntoIterator<IntoIter = Ex>>(
        self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.describe_unit(description, examples, deprecated),
            Self::Right(right) => right.describe_unit(description, examples, deprecated),
        }
    }

    fn describe_unit_struct<I: IntoIterator<IntoIter = Ex>>(
        self,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Left(left) => left.describe_unit_struct(id, description, examples, deprecated),
            Self::Right(right) => right.describe_unit_struct(id, description, examples, deprecated),
        }
    }

    fn describe_newtype_struct<I: IntoIterator<IntoIter = Ex>>(
        self,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::NewtypeStructSchemaBuilder, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_newtype_struct(
                id,
                description,
                examples,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_newtype_struct(
                id,
                description,
                examples,
                deprecated,
            )?)),
        }
    }

    fn describe_seq<I: IntoIterator<IntoIter = Ex>>(
        self,
        min_len: Option<usize>,
        max_len: Option<usize>,
        unique: bool,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::SeqSchemaBuilder, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_seq(
                min_len,
                max_len,
                unique,
                description,
                examples,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_seq(
                min_len,
                max_len,
                unique,
                description,
                examples,
                deprecated,
            )?)),
        }
    }

    fn describe_tuple<I: IntoIterator<IntoIter = Ex>>(
        self,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::TupleSchemaBuilder, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_tuple(
                len,
                description,
                examples,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_tuple(
                len,
                description,
                examples,
                deprecated,
            )?)),
        }
    }

    fn describe_tuple_struct<I: IntoIterator<IntoIter = Ex>>(
        self,
        id: Option<SchemaId>,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::TupleStructSchemaBuilder, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_tuple_struct(
                id,
                len,
                description,
                examples,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_tuple_struct(
                id,
                len,
                description,
                examples,
                deprecated,
            )?)),
        }
    }

    fn describe_map<I: IntoIterator<IntoIter = Ex>>(
        self,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::MapSchemaBuilder, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_map(
                id,
                description,
                examples,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_map(
                id,
                description,
                examples,
                deprecated,
            )?)),
        }
    }

    fn describe_struct<I: IntoIterator<IntoIter = Ex>>(
        self,
        id: Option<SchemaId>,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::StructSchemaBuilder, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_struct(
                id,
                len,
                description,
                examples,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_struct(
                id,
                len,
                description,
                examples,
                deprecated,
            )?)),
        }
    }

    fn describe_enum<I: IntoIterator<IntoIter = Ex>>(
        self,
        id: Option<SchemaId>,
        len: usize,
        exhaustive: bool,
        tag: VariantTag,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::EnumSchemaBuilder, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_enum(
                id,
                len,
                exhaustive,
                tag,
                description,
                examples,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_enum(
                id,
                len,
                exhaustive,
                tag,
                description,
                examples,
                deprecated,
            )?)),
        }
    }

    fn describe_not<I: IntoIterator<IntoIter = Ex>>(
        self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::NotSchemaBuilder, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_not(
                description,
                examples,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_not(
                description,
                examples,
                deprecated,
            )?)),
        }
    }

    fn describe_combinator<I: IntoIterator<IntoIter = Ex>>(
        self,
        combinator: Combinator,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::CombinatorSchemaBuilder, Self::Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.describe_combinator(
                combinator,
                len,
                description,
                examples,
                deprecated,
            )?)),
            Self::Right(right) => Ok(Either::Right(right.describe_combinator(
                combinator,
                len,
                description,
                examples,
                deprecated,
            )?)),
        }
    }
}
