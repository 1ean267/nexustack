/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::openapi::{
    Nop, error,
    schema::{
        Schema,
        builder::{Combinator, SchemaBuilder, SchemaId, VariantTag},
    },
};
use serde::Serialize;
use std::marker::PhantomData;

/// Extension trait for [`Schema`] that provides access to example values for a type's `OpenAPI` schema.
///
/// This trait is automatically implemented for all types that implement [`Schema`].
/// It allows you to retrieve example values for a schema, which can be used for documentation,
/// testing, or generating sample data.
///
/// # Associated Types
///
/// - [`Schema::Example`]: The type of a single example value for this schema.
/// - [`Schema::Examples`]: An iterator over example values of type [`Schema::Example`].
///
/// # Usage
///
/// Use [`SchemaExamples::examples`] to obtain example values for a type's schema. You can specify
/// whether you want human-readable examples (e.g., strings for IP addresses) or machine-oriented
/// examples (e.g., tuples or byte arrays).
///
/// ## Example
///
/// ```rust
/// use nexustack::openapi::{api_schema, Schema, SchemaExamples};
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
/// // For a primitive type
/// let bool_examples: Vec<bool> = bool::examples::<Error>(true).unwrap().collect();
/// assert_eq!(bool_examples, vec![true, false]);
///
/// // For a custom struct
///
/// /// Custom type
/// #[api_schema]
/// struct MyStruct {
///     /// Field id
///     id: u32,
///     /// Field name
///     name: String,
/// }
///
/// let examples = MyStruct::examples::<Error>(true).unwrap().collect::<Vec<_>>();
/// assert_eq!(bool_examples.is_empty(), false);
/// ```
///
/// # See Also
///
/// - [`Schema`]: The trait for describing `OpenAPI` schemas.
/// - [`SchemaExamples::examples`]: The method to retrieve example values.
///
pub trait SchemaExamples: Schema {
    /// Returns an iterator over example values for this schema.
    ///
    /// # Paramaters
    ///
    /// - `is_human_readable` - Determines whether examples should be produced in human-readable form.
    ///   Some types have a human-readable form that may be somewhat expensive to construct, as well as a binary form that is compact and efficient.
    ///   Generally, text-based formats like JSON and YAML will prefer to use the human-readable one, and binary formats like Postcard will prefer the compact one.
    ///   See [`serde::Serializer::is_human_readable`] for details.
    ///
    /// # Errors
    ///
    /// Returns an error if example extraction fails, such as when the schema description cannot be produced,
    /// or if the underlying type's example generation encounters an error. The error type is determined by the
    /// [`Error`] type parameter.
    ///
    fn examples<Error: error::Error>(
        is_human_readable: bool,
    ) -> Result<<Self as Schema>::Examples, Error>;
}

/// Blanket implementation for all types that implement [`Schema`].
impl<T: ?Sized + Schema> SchemaExamples for T {
    fn examples<Error: error::Error>(
        is_human_readable: bool,
    ) -> Result<<Self as Schema>::Examples, Error> {
        <Self as Schema>::describe(ExampleExtractor::new(is_human_readable))
    }
}

pub(crate) struct ExampleExtractor<Error: error::Error> {
    is_human_readable: bool,
    _err: PhantomData<fn() -> Error>,
}

impl<Error: error::Error> ExampleExtractor<Error> {
    fn new(is_human_readable: bool) -> Self {
        Self {
            is_human_readable,
            _err: PhantomData,
        }
    }
}

impl<Error: error::Error> Default for ExampleExtractor<Error> {
    fn default() -> Self {
        Self::new(true)
    }
}

impl<Error: error::Error> Clone for ExampleExtractor<Error> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Error: error::Error> Copy for ExampleExtractor<Error> {}

impl<E: Iterator<Item: Serialize + 'static>, Error: error::Error> SchemaBuilder<E>
    for ExampleExtractor<Error>
{
    type MapKey = ();
    type Ok = E;
    type Error = Error;

    type TupleSchemaBuilder = Nop<(), Self::Ok, Self::Error>;
    type TupleStructSchemaBuilder = Nop<(), Self::Ok, Self::Error>;
    type StructSchemaBuilder = Nop<(), Self::Ok, Self::Error>;
    type CombinatorSchemaBuilder = Nop<(), Self::Ok, Self::Error>;
    type EnumSchemaBuilder = Nop<(), Self::Ok, Self::Error>;
    type MapSchemaBuilder = Nop<(), Self::Ok, Self::Error>;
    type OptionSchemaBuilder = Nop<(), Self::Ok, Self::Error>;
    type NewtypeStructSchemaBuilder = Nop<(), Self::Ok, Self::Error>;
    type SeqSchemaBuilder = Nop<(), Self::Ok, Self::Error>;
    type NotSchemaBuilder = Nop<(), Self::Ok, Self::Error>;

    fn describe_option<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::OptionSchemaBuilder, Self::Error> {
        Ok(Nop::new(examples()?.into_iter(), self.is_human_readable))
    }

    fn describe_bool<I: IntoIterator<IntoIter = E>>(
        self,
        _only: Option<bool>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(examples()?.into_iter())
    }

    fn describe_i8<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<i8>,
        _max: std::ops::Bound<i8>,
        _multiple_of: Option<i8>,
        _format: Option<&'static str>,
        _only: Option<&'static [i8]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(examples()?.into_iter())
    }

    fn describe_i16<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<i16>,
        _max: std::ops::Bound<i16>,
        _multiple_of: Option<i16>,
        _format: Option<&'static str>,
        _only: Option<&'static [i16]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(examples()?.into_iter())
    }

    fn describe_i32<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<i32>,
        _max: std::ops::Bound<i32>,
        _multiple_of: Option<i32>,
        _format: Option<&'static str>,
        _only: Option<&'static [i32]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(examples()?.into_iter())
    }

    fn describe_i64<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<i64>,
        _max: std::ops::Bound<i64>,
        _multiple_of: Option<i64>,
        _format: Option<&'static str>,
        _only: Option<&'static [i64]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(examples()?.into_iter())
    }

    fn describe_i128<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<i128>,
        _max: std::ops::Bound<i128>,
        _multiple_of: Option<i128>,
        _format: Option<&'static str>,
        _only: Option<&'static [i128]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(examples()?.into_iter())
    }

    fn describe_u8<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<u8>,
        _max: std::ops::Bound<u8>,
        _multiple_of: Option<u8>,
        _format: Option<&'static str>,
        _only: Option<&'static [u8]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(examples()?.into_iter())
    }

    fn describe_u16<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<u16>,
        _max: std::ops::Bound<u16>,
        _multiple_of: Option<u16>,
        _format: Option<&'static str>,
        _only: Option<&'static [u16]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(examples()?.into_iter())
    }

    fn describe_u32<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<u32>,
        _max: std::ops::Bound<u32>,
        _multiple_of: Option<u32>,
        _format: Option<&'static str>,
        _only: Option<&'static [u32]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(examples()?.into_iter())
    }

    fn describe_u64<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<u64>,
        _max: std::ops::Bound<u64>,
        _multiple_of: Option<u64>,
        _format: Option<&'static str>,
        _only: Option<&'static [u64]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(examples()?.into_iter())
    }

    fn describe_u128<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<u128>,
        _max: std::ops::Bound<u128>,
        _multiple_of: Option<u128>,
        _format: Option<&'static str>,
        _only: Option<&'static [u128]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(examples()?.into_iter())
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
        Ok(examples()?.into_iter())
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
        Ok(examples()?.into_iter())
    }

    fn describe_char<I: IntoIterator<IntoIter = E>>(
        self,
        _pattern: Option<&'static str>,
        _format: Option<&'static str>,
        _only: Option<&'static [char]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(examples()?.into_iter())
    }

    fn describe_str<I: IntoIterator<IntoIter = E>>(
        self,
        _min_len: Option<usize>,
        _max_len: Option<usize>,
        _pattern: Option<&'static str>,
        _format: Option<&'static str>,
        _only: Option<&'static [&'static str]>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(examples()?.into_iter())
    }

    fn describe_bytes<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(examples()?.into_iter())
    }

    fn describe_unit<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(examples()?.into_iter())
    }

    fn describe_unit_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(examples()?.into_iter())
    }

    fn describe_newtype_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::NewtypeStructSchemaBuilder, Self::Error> {
        Ok(Nop::new(examples()?.into_iter(), self.is_human_readable))
    }

    fn describe_seq<I: IntoIterator<IntoIter = E>>(
        self,
        _min_len: Option<usize>,
        _max_len: Option<usize>,
        _unique: bool,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::SeqSchemaBuilder, Self::Error> {
        Ok(Nop::new(examples()?.into_iter(), self.is_human_readable))
    }

    fn describe_tuple<I: IntoIterator<IntoIter = E>>(
        self,
        _len: usize,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::TupleSchemaBuilder, Self::Error> {
        Ok(Nop::new(examples()?.into_iter(), self.is_human_readable))
    }

    fn describe_tuple_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _len: usize,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::TupleStructSchemaBuilder, Self::Error> {
        Ok(Nop::new(examples()?.into_iter(), self.is_human_readable))
    }

    fn describe_map<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::MapSchemaBuilder, Self::Error> {
        Ok(Nop::new(examples()?.into_iter(), self.is_human_readable))
    }

    fn describe_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _len: usize,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::StructSchemaBuilder, Self::Error> {
        Ok(Nop::new(examples()?.into_iter(), self.is_human_readable))
    }

    fn describe_enum<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _len: usize,
        _exhaustive: bool,
        _tag: VariantTag,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::EnumSchemaBuilder, Self::Error> {
        Ok(Nop::new(examples()?.into_iter(), self.is_human_readable))
    }

    fn describe_not<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::NotSchemaBuilder, Self::Error> {
        Ok(Nop::new(examples()?.into_iter(), self.is_human_readable))
    }

    fn describe_combinator<I: IntoIterator<IntoIter = E>>(
        self,
        _combinator: Combinator,
        _len: usize,
        _description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::CombinatorSchemaBuilder, Self::Error> {
        Ok(Nop::new(examples()?.into_iter(), self.is_human_readable))
    }

    fn is_human_readable(&self) -> bool {
        self.is_human_readable
    }
}
