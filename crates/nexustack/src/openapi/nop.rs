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

/// A no-operation (nop) schema builder that implements all schema builder traits
/// but does not perform any actual schema building.
///
/// Useful for testing, mocking,
/// or disabling schema generation in certain contexts.
///
/// # Example
///
/// ```rust
/// use nexustack::openapi::SchemaBuilder;
/// use nexustack::openapi::Nop;
///
/// // Custom error type for demonstration purposes
/// #[derive(Debug, PartialEq)]
/// struct Error(String);
///
/// impl nexustack::openapi::Error for Error {
///     fn custom<T>(msg: T) -> Self
///         where
///             T: std::fmt::Display {
///         Self(msg.to_string())
///     }
/// }
///
/// impl std::fmt::Display for Error {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
///         f.write_str(&self.0)
///     }
/// }
///
/// impl std::error::Error for Error { }
///
/// // Create a Nop schema builder with a unit result value
/// let builder: Nop<(), (), _> = Default::default();
///
/// let schema: Result<_, Error> = builder.describe_bool(
///     None,
///     Some("Description of the schema"),
///     || Ok([true, false]),
///     false
/// );
///
/// assert_eq!(schema, Ok(()));
/// ```
pub struct Nop<MapKey, Ok, Error> {
    is_human_readable: bool,
    _map_key: PhantomData<fn() -> MapKey>,
    result: Ok,
    _error: PhantomData<fn() -> Error>,
}

impl<Error: error::Error> Default for Nop<(), (), Error> {
    fn default() -> Self {
        Self::new((), true)
    }
}

impl<MapKey, Ok, Error: error::Error> Nop<MapKey, Ok, Error> {
    /// Creates a new [`Nop`] schema builder.
    ///
    /// # Arguments
    /// * `result` - The value to be returned by the builder's `end` methods.
    /// * `is_human_readable` - Indicates whether the schema builder should behave as human-readable.
    pub fn new(result: Ok, is_human_readable: bool) -> Self {
        Self {
            is_human_readable,
            _map_key: PhantomData,
            result,
            _error: PhantomData,
        }
    }
}

impl<MapKey, Ok: Clone, Error: error::Error> Clone for Nop<MapKey, Ok, Error> {
    fn clone(&self) -> Self {
        Self {
            is_human_readable: self.is_human_readable,
            _map_key: PhantomData,
            result: self.result.clone(),
            _error: PhantomData,
        }
    }
}

impl<MapKey, Ok: Copy, Error: error::Error> Copy for Nop<MapKey, Ok, Error> {}

//
// Struct
//

impl<MapKey, Ok, Error: error::Error> StructSchemaBuilder for Nop<MapKey, Ok, Error> {
    type MapKey = MapKey;
    type Ok = Ok;
    type Error = Error;

    type FieldSchemaBuilder<'a>
        = Nop<MapKey, (), Error>
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        _key: &'static str,
        _modifier: FieldMod,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        Ok(Nop::new((), self.is_human_readable))
    }

    fn describe_field_optional<'a, F: Serialize>(
        &'a mut self,
        _key: &'static str,
        _modifier: FieldMod,
        _default: Option<F>,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        Ok(Nop::new((), self.is_human_readable))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }
}

//
// Tuple
//

impl<MapKey, Ok, Error: error::Error> TupleSchemaBuilder for Nop<MapKey, Ok, Error> {
    type MapKey = MapKey;
    type Ok = Ok;
    type Error = Error;

    type ElementSchemaBuilder<'a>
        = Nop<MapKey, (), Error>
    where
        Self: 'a;

    fn describe_element<'a>(
        &'a mut self,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::ElementSchemaBuilder<'a>, Self::Error> {
        Ok(Nop::new((), self.is_human_readable))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }
}

//
// Tuple struct
//

impl<MapKey, Ok, Error: error::Error> TupleStructSchemaBuilder for Nop<MapKey, Ok, Error> {
    type MapKey = MapKey;
    type Ok = Ok;
    type Error = Error;

    type FieldSchemaBuilder<'a>
        = Nop<MapKey, (), Error>
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        Ok(Nop::new((), self.is_human_readable))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }
}

//
// Combinator
//

impl<MapKey, Ok, Error: error::Error> CombinatorSchemaBuilder for Nop<MapKey, Ok, Error> {
    type MapKey = MapKey;
    type Ok = Ok;
    type Error = Error;

    type SubSchemaBuilder<'a>
        = Nop<MapKey, (), Error>
    where
        Self: 'a;

    fn describe_subschema<'a>(
        &'a mut self,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::SubSchemaBuilder<'a>, Self::Error> {
        Ok(Nop::new((), self.is_human_readable))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }
}

//
// Map
//

impl<MapKey, Ok, Error: error::Error> MapSchemaBuilder for Nop<MapKey, Ok, Error> {
    type MapKey = MapKey;
    type Ok = Ok;
    type Error = Error;

    type MapKeySchemaBuilder = Nop<MapKey, MapKey, Error>;
    type MapValueSchemaBuilder<'a>
        = Nop<MapKey, (), Error>
    where
        Self: 'a;

    fn describe_element<'a, K: Schema + Serialize>(
        &'a mut self,
        _key: K,
        _modifier: FieldMod,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'a>, Self::Error> {
        Ok(Nop::new((), self.is_human_readable))
    }

    fn describe_element_optional<'a, K: Schema + Serialize, F: Serialize>(
        &'a mut self,
        _key: K,
        _modifier: FieldMod,
        _default: Option<F>,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'a>, Self::Error> {
        Ok(Nop::new((), self.is_human_readable))
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
        Ok(Nop::new((), self.is_human_readable))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }
}

//
// Enum
//

impl<MapKey, Error: error::Error> StructVariantSchemaBuilder for Nop<MapKey, (), Error> {
    type MapKey = MapKey;
    type Error = Error;

    type FieldSchemaBuilder<'a>
        = Self
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        _key: &'static str,
        _modifier: FieldMod,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        Ok(Self::new((), self.is_human_readable))
    }

    fn describe_field_optional<'a, F: Serialize>(
        &'a mut self,
        _key: &'static str,
        _modifier: FieldMod,
        _default: Option<F>,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        Ok(Self::new((), self.is_human_readable))
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<MapKey, Error: error::Error> TupleVariantSchemaBuilder for Nop<MapKey, (), Error> {
    type MapKey = MapKey;
    type Error = Error;

    type FieldSchemaBuilder<'a>
        = Self
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        Ok(Self::new((), self.is_human_readable))
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<MapKey, Ok, Error: error::Error> EnumSchemaBuilder for Nop<MapKey, Ok, Error> {
    type MapKey = MapKey;
    type Ok = Ok;
    type Error = Error;

    type TupleVariantSchemaBuilder<'a>
        = Nop<MapKey, (), Error>
    where
        Self: 'a;

    type StructVariantSchemaBuilder<'a>
        = Nop<MapKey, (), Error>
    where
        Self: 'a;

    type NewTypeVariantSchemaBuilder<'a>
        = Nop<MapKey, (), Error>
    where
        Self: 'a;

    fn describe_unit_variant(
        &mut self,
        _index: u32,
        _id: SchemaId,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn describe_newtype_variant<'a>(
        &'a mut self,
        _index: u32,
        _id: SchemaId,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::NewTypeVariantSchemaBuilder<'a>, Self::Error> {
        Ok(Nop::new((), self.is_human_readable))
    }

    fn describe_tuple_variant<'a>(
        &'a mut self,
        _index: u32,
        _id: SchemaId,
        _len: usize,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::TupleVariantSchemaBuilder<'a>, Self::Error> {
        Ok(Nop::new((), self.is_human_readable))
    }

    fn describe_struct_variant<'a>(
        &'a mut self,
        _index: u32,
        _id: SchemaId,
        _len: usize,
        _description: Option<&'static str>,
        _deprecated: bool,
    ) -> Result<Self::StructVariantSchemaBuilder<'a>, Self::Error> {
        Ok(Nop::new((), self.is_human_readable))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }
}

//
// Schema
//

impl<MapKey, Ok, Error: error::Error> IntoSchemaBuilder for Nop<MapKey, Ok, Error> {
    type MapKey = MapKey;
    type Ok = Ok;
    type Error = Error;
    type SchemaBuilder<Examples>
        = Self
    where
        Examples: Iterator<Item: Serialize + 'static>;

    fn into_schema_builder<Examples>(self) -> Self::SchemaBuilder<Examples>
    where
        Examples: Iterator<Item: Serialize + 'static>,
    {
        self
    }
}

impl<MapKey, Ok, Error: error::Error, Examples> SchemaBuilder<Examples> for Nop<MapKey, Ok, Error>
where
    Examples: Iterator<Item: Serialize + 'static>,
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

    fn describe_option<I: IntoIterator<IntoIter = Examples>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::OptionSchemaBuilder, Self::Error> {
        Ok(self)
    }

    fn describe_bool<I: IntoIterator<IntoIter = Examples>>(
        self,
        _only: Option<bool>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }

    fn describe_i8<I: IntoIterator<IntoIter = Examples>>(
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
        Ok(self.result)
    }

    fn describe_i16<I: IntoIterator<IntoIter = Examples>>(
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
        Ok(self.result)
    }

    fn describe_i32<I: IntoIterator<IntoIter = Examples>>(
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
        Ok(self.result)
    }

    fn describe_i64<I: IntoIterator<IntoIter = Examples>>(
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
        Ok(self.result)
    }

    fn describe_i128<I: IntoIterator<IntoIter = Examples>>(
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
        Ok(self.result)
    }

    fn describe_u8<I: IntoIterator<IntoIter = Examples>>(
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
        Ok(self.result)
    }

    fn describe_u16<I: IntoIterator<IntoIter = Examples>>(
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
        Ok(self.result)
    }

    fn describe_u32<I: IntoIterator<IntoIter = Examples>>(
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
        Ok(self.result)
    }

    fn describe_u64<I: IntoIterator<IntoIter = Examples>>(
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
        Ok(self.result)
    }

    fn describe_u128<I: IntoIterator<IntoIter = Examples>>(
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
        Ok(self.result)
    }

    fn describe_f32<I: IntoIterator<IntoIter = Examples>>(
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
        Ok(self.result)
    }

    fn describe_f64<I: IntoIterator<IntoIter = Examples>>(
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
        Ok(self.result)
    }

    fn describe_char<I: IntoIterator<IntoIter = Examples>>(
        self,
        _pattern: Option<&'static str>,
        _format: Option<&'static str>,
        _only: Option<&'static [char]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }

    fn describe_str<I: IntoIterator<IntoIter = Examples>>(
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
        Ok(self.result)
    }

    fn describe_bytes<I: IntoIterator<IntoIter = Examples>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }

    fn describe_unit<I: IntoIterator<IntoIter = Examples>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }

    fn describe_unit_struct<I: IntoIterator<IntoIter = Examples>>(
        self,
        _id: Option<SchemaId>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }

    fn describe_newtype_struct<I: IntoIterator<IntoIter = Examples>>(
        self,
        _id: Option<SchemaId>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::NewtypeStructSchemaBuilder, Self::Error> {
        Ok(self)
    }

    fn describe_seq<I: IntoIterator<IntoIter = Examples>>(
        self,
        _min_len: Option<usize>,
        _max_len: Option<usize>,
        _unique: bool,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::SeqSchemaBuilder, Self::Error> {
        Ok(self)
    }

    fn describe_tuple<I: IntoIterator<IntoIter = Examples>>(
        self,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::TupleSchemaBuilder, Self::Error> {
        Ok(self)
    }

    fn describe_tuple_struct<I: IntoIterator<IntoIter = Examples>>(
        self,
        _id: Option<SchemaId>,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::TupleStructSchemaBuilder, Self::Error> {
        Ok(self)
    }

    fn describe_map<I: IntoIterator<IntoIter = Examples>>(
        self,
        _id: Option<SchemaId>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::MapSchemaBuilder, Self::Error> {
        Ok(self)
    }

    fn describe_struct<I: IntoIterator<IntoIter = Examples>>(
        self,
        _id: Option<SchemaId>,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::StructSchemaBuilder, Self::Error> {
        Ok(self)
    }

    fn describe_enum<I: IntoIterator<IntoIter = Examples>>(
        self,
        _id: Option<SchemaId>,
        _len: usize,
        _exhaustive: bool,
        _tag: VariantTag,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::EnumSchemaBuilder, Self::Error> {
        Ok(self)
    }

    fn describe_not<I: IntoIterator<IntoIter = Examples>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::NotSchemaBuilder, Self::Error> {
        Ok(self)
    }

    fn describe_combinator<I: IntoIterator<IntoIter = Examples>>(
        self,
        _combinator: Combinator,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::CombinatorSchemaBuilder, Self::Error> {
        Ok(self)
    }

    fn is_human_readable(&self) -> bool {
        self.is_human_readable
    }
}
