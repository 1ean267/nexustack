/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::openapi::{
    Combinator, CombinatorSchemaBuilder, EnumSchemaBuilder, FieldMod, IntoSchemaBuilder,
    MapSchemaBuilder, SchemaBuilder, SchemaId, StructSchemaBuilder, TupleSchemaBuilder,
    TupleStructSchemaBuilder, VariantTag,
};
use serde::Serialize;

/// A wrapper type that represents an optional schema element.
///
/// The `Optional` struct is used to indicate whether a schema element is optional
/// in the context of `OpenAPI` schema generation. It wraps another schema builder
/// and provides additional metadata about the optionality of the schema element.
///
/// # Type Parameters
/// - `S`: The inner schema builder type that this `Optional` wraps.
pub struct Optional<S> {
    is_root: bool,
    #[allow(clippy::struct_field_names)]
    is_optional: bool,
    inner: S,
}

impl<S> Optional<S> {
    /// Creates a new `Optional` schema builder.
    ///
    /// # Parameters
    /// - `inner`: The inner schema builder to wrap.
    pub const fn new(inner: S) -> Self {
        Self {
            is_root: true,
            is_optional: false,
            inner,
        }
    }
}

impl<S> IntoSchemaBuilder for Optional<S>
where
    S: IntoSchemaBuilder,
{
    type MapKey = <S as IntoSchemaBuilder>::MapKey;
    type Ok = (bool, <S as IntoSchemaBuilder>::Ok);
    type Error = <S as IntoSchemaBuilder>::Error;

    type SchemaBuilder<E: Iterator<Item: Serialize + 'static>> = Optional<S::SchemaBuilder<E>>;

    fn into_schema_builder<E: Iterator<Item: Serialize + 'static>>(self) -> Self::SchemaBuilder<E> {
        Optional {
            is_root: self.is_root,
            is_optional: self.is_optional,
            inner: self.inner.into_schema_builder(),
        }
    }
}

impl<S> TupleSchemaBuilder for Optional<S>
where
    S: TupleSchemaBuilder,
{
    type MapKey = <S as TupleSchemaBuilder>::MapKey;
    type Ok = (bool, <S as TupleSchemaBuilder>::Ok);
    type Error = <S as TupleSchemaBuilder>::Error;

    type ElementSchemaBuilder<'a>
        = <S as TupleSchemaBuilder>::ElementSchemaBuilder<'a>
    where
        Self: 'a;

    fn describe_element<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::ElementSchemaBuilder<'a>, Self::Error> {
        self.inner.describe_element(description, deprecated)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok((self.is_optional, self.inner.end()?))
    }
}

impl<S> TupleStructSchemaBuilder for Optional<S>
where
    S: TupleStructSchemaBuilder,
{
    type MapKey = <S as TupleStructSchemaBuilder>::MapKey;
    type Ok = (bool, <S as TupleStructSchemaBuilder>::Ok);
    type Error = <S as TupleStructSchemaBuilder>::Error;

    type FieldSchemaBuilder<'a>
        = <S as TupleStructSchemaBuilder>::FieldSchemaBuilder<'a>
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        self.inner.describe_field(description, deprecated)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok((self.is_optional, self.inner.end()?))
    }
}

impl<S> StructSchemaBuilder for Optional<S>
where
    S: StructSchemaBuilder,
{
    type MapKey = <S as StructSchemaBuilder>::MapKey;
    type Ok = (bool, <S as StructSchemaBuilder>::Ok);
    type Error = <S as StructSchemaBuilder>::Error;

    type FieldSchemaBuilder<'a>
        = <S as StructSchemaBuilder>::FieldSchemaBuilder<'a>
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        self.inner
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
        self.inner
            .describe_field_optional(key, modifier, default, description, deprecated)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok((self.is_optional, self.inner.end()?))
    }
}

impl<S> CombinatorSchemaBuilder for Optional<S>
where
    S: CombinatorSchemaBuilder,
{
    type MapKey = <S as CombinatorSchemaBuilder>::MapKey;
    type Ok = (bool, <S as CombinatorSchemaBuilder>::Ok);
    type Error = <S as CombinatorSchemaBuilder>::Error;

    type SubSchemaBuilder<'a>
        = <S as CombinatorSchemaBuilder>::SubSchemaBuilder<'a>
    where
        Self: 'a;

    fn describe_subschema<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::SubSchemaBuilder<'a>, Self::Error> {
        self.inner.describe_subschema(description, deprecated)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok((self.is_optional, self.inner.end()?))
    }
}

impl<S> EnumSchemaBuilder for Optional<S>
where
    S: EnumSchemaBuilder,
{
    type MapKey = <S as EnumSchemaBuilder>::MapKey;
    type Ok = (bool, <S as EnumSchemaBuilder>::Ok);
    type Error = <S as EnumSchemaBuilder>::Error;

    type TupleVariantSchemaBuilder<'a>
        = <S as EnumSchemaBuilder>::TupleVariantSchemaBuilder<'a>
    where
        Self: 'a;

    type StructVariantSchemaBuilder<'a>
        = <S as EnumSchemaBuilder>::StructVariantSchemaBuilder<'a>
    where
        Self: 'a;

    type NewTypeVariantSchemaBuilder<'a>
        = <S as EnumSchemaBuilder>::NewTypeVariantSchemaBuilder<'a>
    where
        Self: 'a;

    fn describe_unit_variant(
        &mut self,
        index: u32,
        id: SchemaId,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<(), Self::Error> {
        self.inner
            .describe_unit_variant(index, id, description, deprecated)
    }

    fn describe_newtype_variant<'a>(
        &'a mut self,
        index: u32,
        id: SchemaId,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::NewTypeVariantSchemaBuilder<'a>, Self::Error> {
        self.inner
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
        self.inner
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
        self.inner
            .describe_struct_variant(index, id, len, description, deprecated)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok((self.is_optional, self.inner.end()?))
    }
}

impl<S> MapSchemaBuilder for Optional<S>
where
    S: MapSchemaBuilder,
{
    type MapKey = <S as MapSchemaBuilder>::MapKey;
    type Ok = (bool, <S as MapSchemaBuilder>::Ok);
    type Error = <S as MapSchemaBuilder>::Error;

    type MapKeySchemaBuilder = <S as MapSchemaBuilder>::MapKeySchemaBuilder;

    type MapValueSchemaBuilder<'a>
        = <S as MapSchemaBuilder>::MapValueSchemaBuilder<'a>
    where
        Self: 'a;

    fn describe_element<'a, K: super::Schema + Serialize>(
        &'a mut self,
        key: K,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'a>, Self::Error> {
        self.inner
            .describe_element(key, modifier, description, deprecated)
    }

    fn describe_element_optional<'a, K: super::Schema + Serialize, F: Serialize>(
        &'a mut self,
        key: K,
        modifier: FieldMod,
        default: Option<F>,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'a>, Self::Error> {
        self.inner
            .describe_element_optional(key, modifier, default, description, deprecated)
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
        self.inner
            .describe_additional_elements(describe_key, description, deprecated)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok((self.is_optional, self.inner.end()?))
    }
}

impl<S, E> SchemaBuilder<E> for Optional<S>
where
    S: SchemaBuilder<E>,
    E: Iterator<Item: Serialize + 'static>,
{
    type MapKey = <S as SchemaBuilder<E>>::MapKey;
    type Ok = (bool, <S as SchemaBuilder<E>>::Ok);
    type Error = <S as SchemaBuilder<E>>::Error;
    type TupleSchemaBuilder = Optional<<S as SchemaBuilder<E>>::TupleSchemaBuilder>;
    type TupleStructSchemaBuilder = Optional<<S as SchemaBuilder<E>>::TupleStructSchemaBuilder>;
    type StructSchemaBuilder = Optional<<S as SchemaBuilder<E>>::StructSchemaBuilder>;
    type CombinatorSchemaBuilder = Optional<<S as SchemaBuilder<E>>::CombinatorSchemaBuilder>;
    type EnumSchemaBuilder = Optional<<S as SchemaBuilder<E>>::EnumSchemaBuilder>;
    type MapSchemaBuilder = Optional<<S as SchemaBuilder<E>>::MapSchemaBuilder>;
    type OptionSchemaBuilder = Optional<<S as SchemaBuilder<E>>::OptionSchemaBuilder>;
    type NewtypeStructSchemaBuilder = Optional<<S as SchemaBuilder<E>>::NewtypeStructSchemaBuilder>;
    type SeqSchemaBuilder = Optional<<S as SchemaBuilder<E>>::SeqSchemaBuilder>;
    type NotSchemaBuilder = Optional<<S as SchemaBuilder<E>>::NotSchemaBuilder>;

    fn describe_option<I: IntoIterator<IntoIter = E>>(
        self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::OptionSchemaBuilder, Self::Error> {
        Ok(Optional {
            is_root: false,
            is_optional: self.is_optional || self.is_root,
            inner: self
                .inner
                .describe_option(description, examples, deprecated)?,
        })
    }

    fn describe_bool<I: IntoIterator<IntoIter = E>>(
        self,
        only: Option<bool>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok((
            self.is_optional,
            self.inner
                .describe_bool(only, description, examples, deprecated)?,
        ))
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
        Ok((
            self.is_optional,
            self.inner.describe_i8(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            )?,
        ))
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
        Ok((
            self.is_optional,
            self.inner.describe_i16(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            )?,
        ))
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
        Ok((
            self.is_optional,
            self.inner.describe_i32(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            )?,
        ))
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
        Ok((
            self.is_optional,
            self.inner.describe_i64(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            )?,
        ))
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
        Ok((
            self.is_optional,
            self.inner.describe_i128(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            )?,
        ))
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
        Ok((
            self.is_optional,
            self.inner.describe_u8(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            )?,
        ))
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
        Ok((
            self.is_optional,
            self.inner.describe_u16(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            )?,
        ))
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
        Ok((
            self.is_optional,
            self.inner.describe_u32(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            )?,
        ))
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
        Ok((
            self.is_optional,
            self.inner.describe_u64(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            )?,
        ))
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
        Ok((
            self.is_optional,
            self.inner.describe_u128(
                min,
                max,
                multiple_of,
                format,
                only,
                description,
                examples,
                deprecated,
            )?,
        ))
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
        Ok((
            self.is_optional,
            self.inner.describe_f32(
                allow_nan,
                allow_inf,
                min,
                max,
                format,
                description,
                examples,
                deprecated,
            )?,
        ))
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
        Ok((
            self.is_optional,
            self.inner.describe_f64(
                allow_nan,
                allow_inf,
                min,
                max,
                format,
                description,
                examples,
                deprecated,
            )?,
        ))
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
        Ok((
            self.is_optional,
            self.inner
                .describe_char(pattern, format, only, description, examples, deprecated)?,
        ))
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
        Ok((
            self.is_optional,
            self.inner.describe_str(
                min_len,
                max_len,
                pattern,
                format,
                only,
                description,
                examples,
                deprecated,
            )?,
        ))
    }

    fn describe_bytes<I: IntoIterator<IntoIter = E>>(
        self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok((
            self.is_optional,
            self.inner
                .describe_bytes(description, examples, deprecated)?,
        ))
    }

    fn describe_unit<I: IntoIterator<IntoIter = E>>(
        self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok((
            self.is_optional,
            self.inner
                .describe_unit(description, examples, deprecated)?,
        ))
    }

    fn describe_unit_struct<I: IntoIterator<IntoIter = E>>(
        self,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok((
            self.is_optional,
            self.inner
                .describe_unit_struct(id, description, examples, deprecated)?,
        ))
    }

    fn describe_newtype_struct<I: IntoIterator<IntoIter = E>>(
        self,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::NewtypeStructSchemaBuilder, Self::Error> {
        Ok(Optional {
            is_root: self.is_root,
            is_optional: self.is_optional,
            inner: self
                .inner
                .describe_newtype_struct(id, description, examples, deprecated)?,
        })
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
        Ok(Optional {
            is_root: false,
            is_optional: self.is_optional,
            inner: self.inner.describe_seq(
                min_len,
                max_len,
                unique,
                description,
                examples,
                deprecated,
            )?,
        })
    }

    fn describe_tuple<I: IntoIterator<IntoIter = E>>(
        // TODO: Example in doc
        self,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::TupleSchemaBuilder, Self::Error> {
        Ok(Optional {
            is_root: false,
            is_optional: self.is_optional,
            inner: self
                .inner
                .describe_tuple(len, description, examples, deprecated)?,
        })
    }

    fn describe_tuple_struct<I: IntoIterator<IntoIter = E>>(
        // TODO: Example in doc
        self,
        id: Option<SchemaId>,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::TupleStructSchemaBuilder, Self::Error> {
        Ok(Optional {
            is_root: false,
            is_optional: self.is_optional,
            inner: self
                .inner
                .describe_tuple_struct(id, len, description, examples, deprecated)?,
        })
    }

    fn describe_map<I: IntoIterator<IntoIter = E>>(
        // TODO: Example in doc
        self,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::MapSchemaBuilder, Self::Error> {
        Ok(Optional {
            is_root: false,
            is_optional: self.is_optional,
            inner: self
                .inner
                .describe_map(id, description, examples, deprecated)?,
        })
    }

    fn describe_struct<I: IntoIterator<IntoIter = E>>(
        self,
        id: Option<SchemaId>,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::StructSchemaBuilder, Self::Error> {
        Ok(Optional {
            is_root: false,
            is_optional: self.is_optional,
            inner: self
                .inner
                .describe_struct(id, len, description, examples, deprecated)?,
        })
    }

    fn describe_enum<I: IntoIterator<IntoIter = E>>(
        // TODO: Example in doc
        self,
        id: Option<SchemaId>,
        len: usize,
        exhaustive: bool,
        tag: VariantTag,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::EnumSchemaBuilder, Self::Error> {
        Ok(Optional {
            is_root: false,
            is_optional: self.is_optional,
            inner: self.inner.describe_enum(
                id,
                len,
                exhaustive,
                tag,
                description,
                examples,
                deprecated,
            )?,
        })
    }

    fn describe_not<I: IntoIterator<IntoIter = E>>(
        self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::NotSchemaBuilder, Self::Error> {
        Ok(Optional {
            is_root: false,
            is_optional: self.is_optional,
            inner: self.inner.describe_not(description, examples, deprecated)?,
        })
    }

    fn describe_combinator<I: IntoIterator<IntoIter = E>>(
        // TODO: Example in doc
        self,
        combinator: Combinator,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::CombinatorSchemaBuilder, Self::Error> {
        Ok(Optional {
            is_root: false,
            is_optional: self.is_optional,
            inner: self.inner.describe_combinator(
                combinator,
                len,
                description,
                examples,
                deprecated,
            )?,
        })
    }
}
