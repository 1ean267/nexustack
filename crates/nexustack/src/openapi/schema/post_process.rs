/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::openapi::schema::{
    Schema,
    builder::{
        Combinator, CombinatorSchemaBuilder, EnumSchemaBuilder, FieldMod, IntoSchemaBuilder,
        MapSchemaBuilder, SchemaBuilder, SchemaId, StructSchemaBuilder, TupleSchemaBuilder,
        TupleStructSchemaBuilder, VariantTag,
    },
};
use serde::Serialize;

pub trait Transform<In> {
    type Output;
    type Error;

    fn transform(self, i: In) -> Result<Self::Output, Self::Error>;
}

pub struct PostProcessSchemaBuilder<T, S> {
    transform: T,
    schema_builder: S,
}

impl<T, S> PostProcessSchemaBuilder<T, S> {
    pub const fn new(transform: T, schema_builder: S) -> Self {
        Self {
            transform,
            schema_builder,
        }
    }
}

//
// Struct
//

impl<T, S: StructSchemaBuilder> StructSchemaBuilder for PostProcessSchemaBuilder<T, S>
where
    T: Transform<S::Ok, Error = S::Error>,
{
    type MapKey = S::MapKey;
    type Ok = T::Output;
    type Error = S::Error;

    type FieldSchemaBuilder<'a>
        = S::FieldSchemaBuilder<'a>
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        self.schema_builder
            .describe_field(key, modifier, description, deprecated)
    }

    fn describe_field_optional<'a, F: Serialize>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        default: Option<F>,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        self.schema_builder
            .describe_field_optional(key, modifier, default, description, deprecated)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        (self.transform).transform(self.schema_builder.end()?)
    }
}

//
// Tuple
//

impl<T, S: TupleSchemaBuilder> TupleSchemaBuilder for PostProcessSchemaBuilder<T, S>
where
    T: Transform<S::Ok, Error = S::Error>,
{
    type MapKey = S::MapKey;
    type Ok = T::Output;
    type Error = S::Error;

    type ElementSchemaBuilder<'a>
        = S::ElementSchemaBuilder<'a>
    where
        Self: 'a;

    fn describe_element<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::ElementSchemaBuilder<'a>, Self::Error> {
        self.schema_builder
            .describe_element(description, deprecated)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        (self.transform).transform(self.schema_builder.end()?)
    }
}

//
// Tuple struct
//

impl<T, S: TupleStructSchemaBuilder> TupleStructSchemaBuilder for PostProcessSchemaBuilder<T, S>
where
    T: Transform<S::Ok, Error = S::Error>,
{
    type MapKey = S::MapKey;
    type Ok = T::Output;
    type Error = S::Error;

    type FieldSchemaBuilder<'a>
        = S::FieldSchemaBuilder<'a>
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        self.schema_builder.describe_field(description, deprecated)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        (self.transform).transform(self.schema_builder.end()?)
    }
}

//
// Combinator
//

impl<T, S: CombinatorSchemaBuilder> CombinatorSchemaBuilder for PostProcessSchemaBuilder<T, S>
where
    T: Transform<S::Ok, Error = S::Error>,
{
    type MapKey = S::MapKey;
    type Ok = T::Output;
    type Error = S::Error;

    type SubSchemaBuilder<'a>
        = S::SubSchemaBuilder<'a>
    where
        Self: 'a;

    fn describe_subschema<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::SubSchemaBuilder<'a>, Self::Error> {
        self.schema_builder
            .describe_subschema(description, deprecated)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        (self.transform).transform(self.schema_builder.end()?)
    }
}

//
// Map
//

impl<T, S: MapSchemaBuilder> MapSchemaBuilder for PostProcessSchemaBuilder<T, S>
where
    T: Transform<S::Ok, Error = S::Error>,
{
    type MapKey = S::MapKey;
    type Ok = T::Output;
    type Error = S::Error;

    type MapKeySchemaBuilder = S::MapKeySchemaBuilder;
    type MapValueSchemaBuilder<'a>
        = S::MapValueSchemaBuilder<'a>
    where
        Self: 'a;

    fn describe_element<'a, K: Schema + Serialize>(
        &'a mut self,
        key: K,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'a>, Self::Error> {
        self.schema_builder
            .describe_element(key, modifier, description, deprecated)
    }

    fn describe_element_optional<'a, K: Schema + Serialize, F: Serialize>(
        &'a mut self,
        key: K,
        modifier: FieldMod,
        default: Option<F>,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'a>, Self::Error> {
        self.schema_builder.describe_element_optional(
            key,
            modifier,
            default,
            description,
            deprecated,
        )
    }

    fn describe_additional_elements<'a, K, I: Iterator<Item: Serialize + 'static>>(
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
        self.schema_builder
            .describe_additional_elements(describe_key, description, deprecated)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        (self.transform).transform(self.schema_builder.end()?)
    }
}

//
// Enum
//

impl<T, S> EnumSchemaBuilder for PostProcessSchemaBuilder<T, S>
where
    T: Transform<S::Ok, Error = S::Error>,
    S: EnumSchemaBuilder,
{
    type MapKey = S::MapKey;
    type Ok = T::Output;
    type Error = S::Error;

    type TupleVariantSchemaBuilder<'a>
        = S::TupleVariantSchemaBuilder<'a>
    where
        Self: 'a;

    type StructVariantSchemaBuilder<'a>
        = S::StructVariantSchemaBuilder<'a>
    where
        Self: 'a;

    type NewTypeVariantSchemaBuilder<'a>
        = S::NewTypeVariantSchemaBuilder<'a>
    where
        Self: 'a;

    fn describe_unit_variant(
        &mut self,
        index: u32,
        id: SchemaId,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<(), Self::Error> {
        self.schema_builder
            .describe_unit_variant(index, id, description, deprecated)
    }

    fn describe_newtype_variant<'a>(
        &'a mut self,
        index: u32,
        id: SchemaId,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::NewTypeVariantSchemaBuilder<'a>, Self::Error> {
        self.schema_builder
            .describe_newtype_variant(index, id, description, deprecated)
    }

    fn describe_tuple_variant<'a>(
        &'a mut self,
        index: u32,
        id: SchemaId,
        len: usize,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::TupleVariantSchemaBuilder<'a>, Self::Error> {
        self.schema_builder
            .describe_tuple_variant(index, id, len, description, deprecated)
    }

    fn describe_struct_variant<'a>(
        &'a mut self,
        index: u32,
        id: SchemaId,
        len: usize,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::StructVariantSchemaBuilder<'a>, Self::Error> {
        self.schema_builder
            .describe_struct_variant(index, id, len, description, deprecated)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        (self.transform).transform(self.schema_builder.end()?)
    }
}

//
// Schema
//

impl<T, S> IntoSchemaBuilder for PostProcessSchemaBuilder<T, S>
where
    T: Transform<S::Ok, Error = S::Error>,
    S: IntoSchemaBuilder,
{
    type MapKey = S::MapKey;
    type Ok = T::Output;
    type Error = S::Error;

    type SchemaBuilder<E: Iterator<Item: Serialize + 'static>> =
        PostProcessSchemaBuilder<T, S::SchemaBuilder<E>>;

    fn into_schema_builder<E: Iterator<Item: Serialize + 'static>>(self) -> Self::SchemaBuilder<E> {
        PostProcessSchemaBuilder::new(self.transform, self.schema_builder.into_schema_builder())
    }
}

impl<T, E, S> SchemaBuilder<E> for PostProcessSchemaBuilder<T, S>
where
    T: Transform<S::Ok, Error = S::Error>,
    E: Iterator<Item: Serialize + 'static>,
    S: SchemaBuilder<E>,
{
    type MapKey = S::MapKey;
    type Ok = T::Output;
    type Error = S::Error;

    type TupleSchemaBuilder = PostProcessSchemaBuilder<T, S::TupleSchemaBuilder>;
    type TupleStructSchemaBuilder = PostProcessSchemaBuilder<T, S::TupleStructSchemaBuilder>;
    type StructSchemaBuilder = PostProcessSchemaBuilder<T, S::StructSchemaBuilder>;
    type CombinatorSchemaBuilder = PostProcessSchemaBuilder<T, S::CombinatorSchemaBuilder>;
    type EnumSchemaBuilder = PostProcessSchemaBuilder<T, S::EnumSchemaBuilder>;
    type MapSchemaBuilder = PostProcessSchemaBuilder<T, S::MapSchemaBuilder>;
    type OptionSchemaBuilder = PostProcessSchemaBuilder<T, S::OptionSchemaBuilder>;
    type NewtypeStructSchemaBuilder = PostProcessSchemaBuilder<T, S::NewtypeStructSchemaBuilder>;
    type SeqSchemaBuilder = PostProcessSchemaBuilder<T, S::SeqSchemaBuilder>;
    type NotSchemaBuilder = PostProcessSchemaBuilder<T, S::NotSchemaBuilder>;

    fn describe_option<I: IntoIterator<IntoIter = E>>(
        self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::OptionSchemaBuilder, Self::Error> {
        Ok(PostProcessSchemaBuilder::new(
            self.transform,
            self.schema_builder
                .describe_option(description, examples, deprecated)?,
        ))
    }

    fn describe_bool<I: IntoIterator<IntoIter = E>>(
        self,
        only: Option<bool>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        (self.transform).transform(self.schema_builder.describe_bool(
            only,
            description,
            examples,
            deprecated,
        )?)
    }

    fn describe_i8<I: IntoIterator<IntoIter = E>>(
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
        (self.transform).transform(self.schema_builder.describe_i8(
            min,
            max,
            multiple_of,
            format,
            only,
            description,
            examples,
            deprecated,
        )?)
    }

    fn describe_i16<I: IntoIterator<IntoIter = E>>(
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
        (self.transform).transform(self.schema_builder.describe_i16(
            min,
            max,
            multiple_of,
            format,
            only,
            description,
            examples,
            deprecated,
        )?)
    }

    fn describe_i32<I: IntoIterator<IntoIter = E>>(
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
        (self.transform).transform(self.schema_builder.describe_i32(
            min,
            max,
            multiple_of,
            format,
            only,
            description,
            examples,
            deprecated,
        )?)
    }

    fn describe_i64<I: IntoIterator<IntoIter = E>>(
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
        (self.transform).transform(self.schema_builder.describe_i64(
            min,
            max,
            multiple_of,
            format,
            only,
            description,
            examples,
            deprecated,
        )?)
    }

    fn describe_i128<I: IntoIterator<IntoIter = E>>(
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
        (self.transform).transform(self.schema_builder.describe_i128(
            min,
            max,
            multiple_of,
            format,
            only,
            description,
            examples,
            deprecated,
        )?)
    }

    fn describe_u8<I: IntoIterator<IntoIter = E>>(
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
        (self.transform).transform(self.schema_builder.describe_u8(
            min,
            max,
            multiple_of,
            format,
            only,
            description,
            examples,
            deprecated,
        )?)
    }

    fn describe_u16<I: IntoIterator<IntoIter = E>>(
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
        (self.transform).transform(self.schema_builder.describe_u16(
            min,
            max,
            multiple_of,
            format,
            only,
            description,
            examples,
            deprecated,
        )?)
    }

    fn describe_u32<I: IntoIterator<IntoIter = E>>(
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
        (self.transform).transform(self.schema_builder.describe_u32(
            min,
            max,
            multiple_of,
            format,
            only,
            description,
            examples,
            deprecated,
        )?)
    }

    fn describe_u64<I: IntoIterator<IntoIter = E>>(
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
        (self.transform).transform(self.schema_builder.describe_u64(
            min,
            max,
            multiple_of,
            format,
            only,
            description,
            examples,
            deprecated,
        )?)
    }

    fn describe_u128<I: IntoIterator<IntoIter = E>>(
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
        (self.transform).transform(self.schema_builder.describe_u128(
            min,
            max,
            multiple_of,
            format,
            only,
            description,
            examples,
            deprecated,
        )?)
    }

    fn describe_f32<I: IntoIterator<IntoIter = E>>(
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
        (self.transform).transform(self.schema_builder.describe_f32(
            allow_nan,
            allow_inf,
            min,
            max,
            format,
            description,
            examples,
            deprecated,
        )?)
    }

    fn describe_f64<I: IntoIterator<IntoIter = E>>(
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
        (self.transform).transform(self.schema_builder.describe_f64(
            allow_nan,
            allow_inf,
            min,
            max,
            format,
            description,
            examples,
            deprecated,
        )?)
    }

    fn describe_char<I: IntoIterator<IntoIter = E>>(
        self,
        pattern: Option<&'static str>,
        format: Option<&'static str>,
        only: Option<&'static [char]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        (self.transform).transform(self.schema_builder.describe_char(
            pattern,
            format,
            only,
            description,
            examples,
            deprecated,
        )?)
    }

    fn describe_str<I: IntoIterator<IntoIter = E>>(
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
        (self.transform).transform(self.schema_builder.describe_str(
            min_len,
            max_len,
            pattern,
            format,
            only,
            description,
            examples,
            deprecated,
        )?)
    }

    fn describe_bytes<I: IntoIterator<IntoIter = E>>(
        self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        (self.transform).transform(self.schema_builder.describe_bytes(
            description,
            examples,
            deprecated,
        )?)
    }

    fn describe_unit<I: IntoIterator<IntoIter = E>>(
        self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        (self.transform).transform(self.schema_builder.describe_unit(
            description,
            examples,
            deprecated,
        )?)
    }

    fn describe_unit_struct<I: IntoIterator<IntoIter = E>>(
        self,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        (self.transform).transform(self.schema_builder.describe_unit_struct(
            id,
            description,
            examples,
            deprecated,
        )?)
    }

    fn describe_newtype_struct<I: IntoIterator<IntoIter = E>>(
        self,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::NewtypeStructSchemaBuilder, Self::Error> {
        Ok(PostProcessSchemaBuilder::new(
            self.transform,
            self.schema_builder
                .describe_newtype_struct(id, description, examples, deprecated)?,
        ))
    }

    fn describe_seq<I: IntoIterator<IntoIter = E>>(
        self,
        min_len: Option<usize>,
        max_len: Option<usize>,
        unique: bool,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::SeqSchemaBuilder, Self::Error> {
        Ok(PostProcessSchemaBuilder::new(
            self.transform,
            self.schema_builder.describe_seq(
                min_len,
                max_len,
                unique,
                description,
                examples,
                deprecated,
            )?,
        ))
    }

    fn describe_tuple<I: IntoIterator<IntoIter = E>>(
        self,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::TupleSchemaBuilder, Self::Error> {
        Ok(PostProcessSchemaBuilder::new(
            self.transform,
            self.schema_builder
                .describe_tuple(len, description, examples, deprecated)?,
        ))
    }

    fn describe_tuple_struct<I: IntoIterator<IntoIter = E>>(
        self,
        id: Option<SchemaId>,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::TupleStructSchemaBuilder, Self::Error> {
        Ok(PostProcessSchemaBuilder::new(
            self.transform,
            self.schema_builder.describe_tuple_struct(
                id,
                len,
                description,
                examples,
                deprecated,
            )?,
        ))
    }

    fn describe_map<I: IntoIterator<IntoIter = E>>(
        self,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::MapSchemaBuilder, Self::Error> {
        Ok(PostProcessSchemaBuilder::new(
            self.transform,
            self.schema_builder
                .describe_map(id, description, examples, deprecated)?,
        ))
    }

    fn describe_struct<I: IntoIterator<IntoIter = E>>(
        self,
        id: Option<SchemaId>,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::StructSchemaBuilder, Self::Error> {
        Ok(PostProcessSchemaBuilder::new(
            self.transform,
            self.schema_builder
                .describe_struct(id, len, description, examples, deprecated)?,
        ))
    }

    fn describe_enum<I: IntoIterator<IntoIter = E>>(
        self,
        id: Option<SchemaId>,
        len: usize,
        exhaustive: bool,
        tag: VariantTag,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::EnumSchemaBuilder, Self::Error> {
        Ok(PostProcessSchemaBuilder::new(
            self.transform,
            self.schema_builder.describe_enum(
                id,
                len,
                exhaustive,
                tag,
                description,
                examples,
                deprecated,
            )?,
        ))
    }

    fn describe_not<I: IntoIterator<IntoIter = E>>(
        self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::NotSchemaBuilder, Self::Error> {
        Ok(PostProcessSchemaBuilder::new(
            self.transform,
            self.schema_builder
                .describe_not(description, examples, deprecated)?,
        ))
    }

    fn describe_combinator<I: IntoIterator<IntoIter = E>>(
        self,
        combinator: Combinator,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::CombinatorSchemaBuilder, Self::Error> {
        Ok(PostProcessSchemaBuilder::new(
            self.transform,
            self.schema_builder.describe_combinator(
                combinator,
                len,
                description,
                examples,
                deprecated,
            )?,
        ))
    }
}
