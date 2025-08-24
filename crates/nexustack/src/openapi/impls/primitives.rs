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
        example::SchemaExamples,
        schema::Schema,
        schema_builder::{
            CombinatorSchemaBuilder, EnumSchemaBuilder, FieldMod, IntoSchemaBuilder, SchemaBuilder,
            SchemaId, StructSchemaBuilder, VariantTag,
        },
    },
};

macro_rules! count_tts {
    () => { 0 };
    ($odd:tt, $($a:tt,$b:tt),* $(,)?) => { (count_tts!($($a,)*) << 1) | 1 };
    ($($a:tt, $even:tt),* $(,)?) => { count_tts!($($a,)*) << 1 };
}

impl Schema for bool {
    type Example = Self;
    type Examples = <[Self; 2] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_bool(None, None, || Ok([true, false]), false)
    }
}

macro_rules! primitive_impl {
    ($ty:path, $method:ident, $($example:expr),+ $(,)?) => {
        impl Schema for $ty {
            type Example = Self;
            type Examples = <[Self; count_tts!($($example,)+)] as IntoIterator>::IntoIter;

            #[inline]
            fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error> where
                B: SchemaBuilder<Self::Examples>
            {
                schema_builder.$method(
                    std::ops::Bound::Unbounded,
                    std::ops::Bound::Unbounded,
                    None,
                    None,
                    None,
                    None,
                    || Ok([$($example,)+]),
                    false,
                )
            }
        }
    };
}

primitive_impl!(i8, describe_i8, i8::MIN, -1, 0, 1, i8::MAX,);
primitive_impl!(i16, describe_i16, i16::MIN, -1, 0, 1, i16::MAX);
primitive_impl!(i32, describe_i32, i32::MIN, -1, 0, 1, i32::MAX);
primitive_impl!(i64, describe_i64, i64::MIN, -1, 0, 1, i64::MAX);
primitive_impl!(i128, describe_i128, i128::MIN, -1, 0, 1, i128::MAX);
primitive_impl!(u8, describe_u8, 0, 1, u8::MAX);
primitive_impl!(u16, describe_u16, 0, 1, u16::MAX);
primitive_impl!(u32, describe_u32, 0, 1, u32::MAX);
primitive_impl!(u64, describe_u64, 0, 1, u64::MAX);
primitive_impl!(u128, describe_u128, 0, 1, u128::MAX);

impl Schema for f32 {
    type Example = Self;
    type Examples = <[Self; 7] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_f32(
            true,
            true,
            std::ops::Bound::Unbounded,
            std::ops::Bound::Unbounded,
            None,
            None,
            || {
                Ok([
                    3.5,
                    27f32,
                    -113.75,
                    0.007_812_5,
                    34_359_738_368_f32,
                    0f32,
                    -1f32,
                ])
            },
            false,
        )
    }
}

impl Schema for f64 {
    type Example = Self;
    type Examples = <[Self; 7] as IntoIterator>::IntoIter;

    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_f64(
            true,
            true,
            std::ops::Bound::Unbounded,
            std::ops::Bound::Unbounded,
            None,
            None,
            || {
                Ok([
                    3.5,
                    27f64,
                    -113.75,
                    0.007_812_5,
                    34_359_738_368_f64,
                    0f64,
                    -1f64,
                ])
            },
            false,
        )
    }
}

impl Schema for char {
    type Example = Self;
    type Examples = <[Self; 9] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_char(
            None,
            None,
            None,
            None,
            || Ok(['h', 'e', 'l', 'o', '\\', 'ß', ':', '\0', '\u{10ffff}']),
            false,
        )
    }
}

impl Schema for str {
    type Example = &'static Self;
    type Examples = <[Self::Example; 12] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            None,
            None,
            None,
            None,
            || {
                Ok([
                    "",
                    "h",
                    "e",
                    "l",
                    "o",
                    "\\",
                    "ß",
                    ":",
                    "\0",
                    "\u{10ffff}",
                    "Hello",
                    "💖",
                ])
            },
            false,
        )
    }
}

impl Schema for String {
    type Example = <str as Schema>::Example;
    type Examples = <str as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        <str as Schema>::describe(schema_builder)
    }
}

impl Schema for std::fmt::Arguments<'_> {
    type Example = <str as Schema>::Example;
    type Examples = <str as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        <str as Schema>::describe(schema_builder)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<T> Schema for Option<T>
where
    T: Schema,
{
    type Example = Option<<T as Schema>::Example>;
    type Examples = std::iter::Chain<
        std::iter::Map<<T as Schema>::Examples, fn(<T as Schema>::Example) -> Self::Example>,
        std::iter::Once<Self::Example>,
    >;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        let is_human_readable = schema_builder.is_human_readable();

        <T as Schema>::describe(
            schema_builder
                .describe_option(
                    None,
                    || {
                        Ok(<T as SchemaExamples>::examples(is_human_readable)?
                            .map(Some as _)
                            .chain(std::iter::once(None)))
                    },
                    false,
                )?
                .into_schema_builder(),
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<T> Schema for std::marker::PhantomData<T>
where
    T: ?Sized,
{
    type Example = std::marker::PhantomData<()>;
    type Examples = std::iter::Once<Self::Example>;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_unit_struct(
            Some(SchemaId::new("PhantomData", callsite!())),
            None,
            || Ok(std::iter::once(std::marker::PhantomData)),
            false,
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<Idx> Schema for std::ops::RangeFrom<Idx>
where
    Idx: Schema,
{
    type Example = std::ops::RangeFrom<<Idx as Schema>::Example>;
    type Examples =
        std::iter::Map<<Idx as Schema>::Examples, fn(<Idx as Schema>::Example) -> Self::Example>;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        let is_human_readable = schema_builder.is_human_readable();
        let mut struct_schema_builder = schema_builder.describe_struct(
            Some(SchemaId::new("RangeFrom", callsite!())),
            1,
            Some("A range only bounded inclusively below"),
            || {
                Ok(<Idx as SchemaExamples>::examples(is_human_readable)?
                    .map((|start| std::ops::RangeFrom { start }) as _))
            },
            false,
        )?;
        struct_schema_builder.collect_field(
            "start",
            FieldMod::ReadWrite,
            Some("The lower bound of the range"),
            false,
            <Idx as Schema>::describe,
        )?;
        struct_schema_builder.end()
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<Idx> Schema for std::ops::RangeInclusive<Idx>
where
    Idx: Schema,
{
    type Example = std::ops::RangeInclusive<<Idx as Schema>::Example>;
    type Examples = std::iter::Map<
        std::iter::Zip<<Idx as Schema>::Examples, <Idx as Schema>::Examples>,
        fn((<Idx as Schema>::Example, <Idx as Schema>::Example)) -> Self::Example,
    >;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        let is_human_readable = schema_builder.is_human_readable();
        let mut struct_schema_builder = schema_builder.describe_struct(
            Some(SchemaId::new("RangeInclusive", callsite!())),
            2,
            Some("A range bounded inclusively below and above"),
            || {
                Ok(std::iter::zip(
                    <Idx as SchemaExamples>::examples(is_human_readable)?,
                    <Idx as SchemaExamples>::examples(is_human_readable)?,
                )
                .map((|(start, end)| std::ops::RangeInclusive::new(start, end)) as _))
            },
            false,
        )?;
        struct_schema_builder.collect_field(
            "start",
            FieldMod::ReadWrite,
            Some("The lower bound of the range (inclusive)"),
            false,
            <Idx as Schema>::describe,
        )?;
        struct_schema_builder.collect_field(
            "end",
            FieldMod::ReadWrite,
            Some("The upper bound of the range (inclusive)"),
            false,
            <Idx as Schema>::describe,
        )?;
        struct_schema_builder.end()
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<Idx> Schema for std::ops::RangeTo<Idx>
where
    Idx: Schema,
{
    type Example = std::ops::RangeTo<<Idx as Schema>::Example>;
    type Examples =
        std::iter::Map<<Idx as Schema>::Examples, fn(<Idx as Schema>::Example) -> Self::Example>;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        let is_human_readable = schema_builder.is_human_readable();
        let mut struct_schema_builder = schema_builder.describe_struct(
            Some(SchemaId::new("RangeTo", callsite!())),
            1,
            Some("A range only bounded exclusively above"),
            || {
                Ok(<Idx as SchemaExamples>::examples(is_human_readable)?
                    .map((|end| std::ops::RangeTo { end }) as _))
            },
            false,
        )?;
        struct_schema_builder.collect_field(
            "end",
            FieldMod::ReadWrite,
            Some("The upper bound of the range (exclusive)"),
            false,
            <Idx as Schema>::describe,
        )?;
        struct_schema_builder.end()
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<T> Schema for std::ops::Bound<T>
where
    T: Schema,
{
    type Example = std::ops::Bound<<T as Schema>::Example>;
    type Examples = std::iter::Chain<
        std::iter::Chain<
            std::iter::Map<<T as Schema>::Examples, fn(<T as Schema>::Example) -> Self::Example>,
            std::iter::Map<<T as Schema>::Examples, fn(<T as Schema>::Example) -> Self::Example>,
        >,
        std::iter::Once<std::ops::Bound<<T as Schema>::Example>>,
    >;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        let is_human_readable = schema_builder.is_human_readable();
        let mut enum_schema_builder = schema_builder.describe_enum(
            Some(SchemaId::new("Bound", callsite!())),
            3,
            true,
            VariantTag::default(),
            Some("An endpoint of a range of keys"),
            || {
                Ok(<T as SchemaExamples>::examples(is_human_readable)?
                    .map(std::ops::Bound::Included as _)
                    .chain(
                        <T as SchemaExamples>::examples(is_human_readable)?
                            .map(std::ops::Bound::Excluded as _),
                    )
                    .chain(std::iter::once(std::ops::Bound::Unbounded)))
            },
            false,
        )?;
        enum_schema_builder.describe_unit_variant(
            0,
            SchemaId::new("Unbounded", callsite!()),
            Some("An infinite endpoint. Indicates that there is no bound in this direction"),
            false,
        )?;
        enum_schema_builder.collect_newtype_variant(
            1,
            SchemaId::new("Included", callsite!()),
            Some("An inclusive bound"),
            false,
            <T as Schema>::describe,
        )?;
        enum_schema_builder.collect_newtype_variant(
            2,
            SchemaId::new("Excluded", callsite!()),
            Some("An exclusive bound"),
            false,
            <T as Schema>::describe,
        )?;
        enum_schema_builder.end()
    }
}

////////////////////////////////////////////////////////////////////////////////

impl Schema for () {
    type Example = Self;
    type Examples = std::iter::Once<Self>;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_unit(None, || Ok(std::iter::once(())), false)
    }
}

macro_rules! nonzero_unsigned_integers {
    ($ty:path, $method:ident, $underlying:ident $(,)?) => {
        impl Schema for $ty {
            type Example = <$underlying as Schema>::Example;
            type Examples = std::iter::Filter<
                <$underlying as Schema>::Examples,
                for<'a> fn(&'a <$underlying as Schema>::Example) -> bool,
            >;

            #[inline]
            fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
            where
                B: SchemaBuilder<Self::Examples>,
            {
                let is_human_readable = schema_builder.is_human_readable();
                schema_builder.$method(
                    std::ops::Bound::Included(1),
                    std::ops::Bound::Unbounded,
                    None,
                    None,
                    None,
                    None,
                    || {
                        Ok(
                            <$underlying as SchemaExamples>::examples(is_human_readable)?
                                .filter((|val: &<$underlying as Schema>::Example| *val != 0) as _),
                        )
                    },
                    false,
                )
            }
        }
    };
}

macro_rules! nonzero_signed_integers {
    ($ty:path, $method:ident, $underlying:ident $(,)?) => {
        impl Schema for $ty {
            type Example = <$underlying as Schema>::Example;
            type Examples = std::iter::Filter<
                <$underlying as Schema>::Examples,
                for<'a> fn(&'a <$underlying as Schema>::Example) -> bool,
            >;

            #[inline]
            fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
            where
                B: SchemaBuilder<Self::Examples>,
            {
                //
                // oneOf: [{
                //   type: ...,
                //   max: -1
                // }, {
                //   type: ...,
                //   min: 1,
                // }]
                //

                let is_human_readable = schema_builder.is_human_readable();
                let mut variant_builder = schema_builder.describe_one_of(
                    2,
                    None,
                    || {
                        Ok(
                            <$underlying as SchemaExamples>::examples(is_human_readable)?.filter(
                                (|val: &<$underlying as Schema>::Example| *val != 0)
                                    as for<'a> fn(&'a <$underlying as Schema>::Example) -> bool,
                            ),
                        )
                    },
                    false,
                )?;

                variant_builder.collect_subschema(None, false, |subschema_builder| {
                    subschema_builder.$method(
                        std::ops::Bound::Unbounded,
                        std::ops::Bound::Excluded(0),
                        None,
                        None,
                        None,
                        None,
                        || {
                            Ok(
                                <$underlying as SchemaExamples>::examples(is_human_readable)?
                                    .filter(
                                        (|val: &<$underlying as Schema>::Example| *val < 0)
                                            as for<'a> fn(
                                                &'a <$underlying as Schema>::Example,
                                            )
                                                -> bool,
                                    ),
                            )
                        },
                        false,
                    )
                })?;

                variant_builder.collect_subschema(None, false, |subschema_builder| {
                    subschema_builder.$method(
                        std::ops::Bound::Excluded(0),
                        std::ops::Bound::Unbounded,
                        None,
                        None,
                        None,
                        None,
                        || {
                            Ok(
                                <$underlying as SchemaExamples>::examples(is_human_readable)?
                                    .filter(
                                        (|val: &<$underlying as Schema>::Example| *val > 0)
                                            as for<'a> fn(
                                                &'a <$underlying as Schema>::Example,
                                            )
                                                -> bool,
                                    ),
                            )
                        },
                        false,
                    )
                })?;

                variant_builder.end()
            }
        }
    };
}

nonzero_unsigned_integers!(std::num::NonZeroU8, describe_u8, u8,);
nonzero_unsigned_integers!(std::num::NonZeroU16, describe_u16, u16,);
nonzero_unsigned_integers!(std::num::NonZeroU32, describe_u32, u32,);
nonzero_unsigned_integers!(std::num::NonZeroU64, describe_u64, u64,);
nonzero_unsigned_integers!(std::num::NonZeroU128, describe_u128, u128,);

nonzero_signed_integers!(std::num::NonZeroI8, describe_i8, i8,);
nonzero_signed_integers!(std::num::NonZeroI16, describe_i16, i16,);
nonzero_signed_integers!(std::num::NonZeroI32, describe_i32, i32,);
nonzero_signed_integers!(std::num::NonZeroI64, describe_i64, i64,);
nonzero_signed_integers!(std::num::NonZeroI128, describe_i128, i128,);

////////////////////////////////////////////////////////////////////////////////

impl<T> Schema for std::cell::Cell<T>
where
    T: Schema + Copy,
{
    type Example = <T as Schema>::Example;
    type Examples = <T as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        <T as Schema>::describe(schema_builder)
    }
}

impl<T> Schema for std::cell::RefCell<T>
where
    T: ?Sized + Schema,
{
    type Example = <T as Schema>::Example;
    type Examples = <T as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        <T as Schema>::describe(schema_builder)
    }
}

impl<T> Schema for std::sync::Mutex<T>
where
    T: ?Sized + Schema,
{
    type Example = <T as Schema>::Example;
    type Examples = <T as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        <T as Schema>::describe(schema_builder)
    }
}

impl<T> Schema for std::sync::RwLock<T>
where
    T: ?Sized + Schema,
{
    type Example = <T as Schema>::Example;
    type Examples = <T as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        <T as Schema>::describe(schema_builder)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<T, E> Schema for Result<T, E>
where
    T: Schema,
    E: Schema,
{
    type Example = Result<<T as Schema>::Example, <E as Schema>::Example>;
    type Examples = std::iter::Chain<
        std::iter::Map<<T as Schema>::Examples, fn(<T as Schema>::Example) -> Self::Example>,
        std::iter::Map<<E as Schema>::Examples, fn(<E as Schema>::Example) -> Self::Example>,
    >;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        let is_human_readable = schema_builder.is_human_readable();
        let mut enum_builder = schema_builder.describe_enum(
            Some(SchemaId::new("Result", callsite!())),
            2,
            true,
            VariantTag::default(),
            Some("A potential error result, i.e. either a success result or an error result"),
            || {
                Ok(<T as SchemaExamples>::examples(is_human_readable)?
                    .map(Result::Ok as _)
                    .chain(
                        <E as SchemaExamples>::examples(is_human_readable)?.map(Result::Err as _),
                    ))
            },
            false,
        )?;
        enum_builder.collect_newtype_variant(
            0,
            SchemaId::new("Ok", callsite!()),
            Some("The success result"),
            false,
            <T as Schema>::describe,
        )?;
        enum_builder.collect_newtype_variant(
            1,
            SchemaId::new("Err", callsite!()),
            Some("The error result"),
            false,
            <E as Schema>::describe,
        )?;
        enum_builder.end()
    }
}

////////////////////////////////////////////////////////////////////////////////

impl Schema for std::path::Path {
    type Example = <str as Schema>::Example;
    type Examples = <str as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        // TODO: Dedicated examples?
        <str as Schema>::describe(schema_builder)
    }
}

impl Schema for std::path::PathBuf {
    type Example = <std::path::Path as Schema>::Example;
    type Examples = <std::path::Path as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        <std::path::Path as Schema>::describe(schema_builder)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<T> Schema for std::num::Wrapping<T>
where
    T: Schema,
{
    type Example = <T as Schema>::Example;
    type Examples = <T as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        T::describe(schema_builder)
    }
}

impl<T> Schema for std::num::Saturating<T>
where
    T: Schema,
{
    type Example = <T as Schema>::Example;
    type Examples = <T as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        <T as Schema>::describe(schema_builder)
    }
}

impl<T: Schema> Schema for std::cmp::Reverse<T> {
    type Example = <T as Schema>::Example;
    type Examples = <T as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        <T as Schema>::describe(schema_builder)
    }
}
