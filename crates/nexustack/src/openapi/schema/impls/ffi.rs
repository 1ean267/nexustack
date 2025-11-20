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
    SchemaExamples,
    schema::{
        Schema,
        builder::{EnumSchemaBuilder, SchemaBuilder, SchemaId, VariantTag},
    },
};

#[cfg(any(unix, windows))]
use crate::callsite;

impl Schema for std::ffi::CStr {
    type Example = <[u8] as Schema>::Example;
    type Examples = <[u8] as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        <[u8] as Schema>::describe(schema_builder)
    }
}

impl Schema for std::ffi::CString {
    type Example = <[u8] as Schema>::Example;
    type Examples = <[u8] as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        <[u8] as Schema>::describe(schema_builder)
    }
}

#[cfg(any(unix, windows))]
callsite!(OsStrCallsite);

#[cfg(any(unix, windows))]
callsite!(OsStrUnixVariantCallsite);

#[cfg(any(unix, windows))]
callsite!(OsStrWindowsVariantCallsite);

#[cfg(any(unix, windows))]
impl Schema for std::ffi::OsStr {
    type Example = &'static Self;
    type Examples =
        std::iter::Map<<str as Schema>::Examples, fn(<str as Schema>::Example) -> Self::Example>;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        // TODO: Dedicated examples?

        let is_human_readable = schema_builder.is_human_readable();
        let mut enum_schema_builder = schema_builder.describe_enum(
            Some(SchemaId::new("OsString", *OsStrCallsite)),
            2,
            true,
            VariantTag::default(),
            Some("An platform-native string"),
            || {
                Ok(<str as SchemaExamples>::examples(is_human_readable)?
                    .map((|str| Self::new(str)) as _))
            },
            false,
        )?;
        enum_schema_builder.collect_newtype_variant(
            0,
            SchemaId::new("Unix", *OsStrUnixVariantCallsite),
            Some("A Unix system platform-native string. An arbitrary sequences of non-zero bytes, in many cases interpreted as UTF-8"),
            false,
            <[u8] as Schema>::describe,
        )?;
        enum_schema_builder.collect_newtype_variant(
            1,
            SchemaId::new("Windows", *OsStrWindowsVariantCallsite),
            Some("A Windows system platform-native string. An arbitrary sequences of non-zero 16-bit values, interpreted as UTF-16 when it is valid to do so"),
            false,
            <Vec<u16> as Schema>::describe,
        )?;
        enum_schema_builder.end()
    }
}

#[cfg(any(unix, windows))]
impl Schema for std::ffi::OsString {
    type Example = <std::ffi::OsStr as Schema>::Example;
    type Examples = <std::ffi::OsStr as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        <std::ffi::OsStr as Schema>::describe(schema_builder)
    }
}
