/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::openapi::Schema;
use uuid::{
    NonNilUuid, Uuid,
    fmt::{Braced, Hyphenated, Simple, Urn},
};

impl Schema for Uuid {
    type Example = either::Either<&'static str, &'static [u8]>;
    type Examples = <[Self::Example; 10] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        if schema_builder.is_human_readable() {
            schema_builder.describe_str(
                Some(36),
                Some(36),
                Some(r"^([0-9a-fA-F]{8}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{12})$"),
                Some("uuid"),
                None,
                Some("A universally unique identifier (UUID)."),
                || Ok([
                    either::Either::Left("550e8400-e29b-41d4-a716-446655440000"),
                    either::Either::Left("f47ac10b-58cc-4372-a567-0e02b2c3d479"),
                    either::Either::Left("123e4567-e89b-12d3-a456-426614174000"),
                    either::Either::Left("987fbc97-4bed-5078-9f07-9141ba07c9f3"),
                    either::Either::Left("00000000-0000-0000-0000-000000000000"),
                    either::Either::Left("ffffffff-ffff-ffff-ffff-ffffffffffff"),
                    either::Either::Left("550E8400-E29B-41D4-A716-446655440000"),
                    either::Either::Left("00000001-0002-0003-0004-000000000005"),
                    either::Either::Left("6ba7b810-9dad-11d1-80b4-00c04fd430c8"),
                    either::Either::Left("3f2504e0-4f89-11d3-9a0c-0305e82c3301"),
                ]),
                false,
            )
        } else {
            // TODO: Set min-length, max-length
            schema_builder.describe_bytes(
                Some("A universally unique identifier (UUID)."),
                || {
                    Ok([
                        either::Either::Right(
                            &[
                                0x55u8, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44,
                                0x66, 0x55, 0x44, 0x00, 0x00,
                            ][..],
                        ),
                        either::Either::Right(
                            &[
                                0xf4u8, 0x7a, 0xc1, 0x0b, 0x58, 0xcc, 0x43, 0x72, 0xa5, 0x67, 0x0e,
                                0x02, 0xb2, 0xc3, 0xd4, 0x79,
                            ][..],
                        ),
                        either::Either::Right(
                            &[
                                0x12u8, 0x3e, 0x45, 0x67, 0xe8, 0x9b, 0x12, 0xd3, 0xa4, 0x56, 0x42,
                                0x66, 0x14, 0x17, 0x40, 0x00,
                            ][..],
                        ),
                        either::Either::Right(
                            &[
                                0x98u8, 0x7f, 0xbc, 0x97, 0x4b, 0xed, 0x50, 0x78, 0x9f, 0x07, 0x91,
                                0x41, 0xba, 0x07, 0xc9, 0xf3,
                            ][..],
                        ),
                        either::Either::Right(
                            &[
                                0x00u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                0x00, 0x00, 0x00, 0x00, 0x00,
                            ][..],
                        ),
                        either::Either::Right(
                            &[
                                0xffu8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                                0xff, 0xff, 0xff, 0xff, 0xff,
                            ][..],
                        ),
                        either::Either::Right(
                            &[
                                0x55u8, 0x0E, 0x84, 0x00, 0xE2, 0x9B, 0x41, 0xD4, 0xA7, 0x16, 0x44,
                                0x66, 0x55, 0x44, 0x00, 0x00,
                            ][..],
                        ),
                        either::Either::Right(
                            &[
                                0x00u8, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00,
                                0x00, 0x00, 0x00, 0x00, 0x05,
                            ][..],
                        ),
                        either::Either::Right(
                            &[
                                0x6bu8, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00,
                                0xc0, 0x4f, 0xd4, 0x30, 0xc8,
                            ][..],
                        ),
                        either::Either::Right(
                            &[
                                0x3fu8, 0x25, 0x04, 0xe0, 0x4f, 0x89, 0x11, 0xd3, 0x9a, 0x0c, 0x03,
                                0x05, 0xe8, 0x2c, 0x33, 0x01,
                            ][..],
                        ),
                    ])
                },
                false,
            )
        }
    }
}

impl Schema for NonNilUuid {
    type Example = either::Either<&'static str, &'static [u8]>;
    type Examples = <[Self::Example; 9] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        if schema_builder.is_human_readable() {
            // TODO: Represent that nil UUID is excluded
            schema_builder.describe_str(
                Some(36),
                Some(36),
                Some(r"^([0-9a-fA-F]{8}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{12})$"),
                Some("uuid"),
                None,
                Some("A non-nil universally unique identifier (UUID)."),
                || Ok([
                    either::Either::Left("550e8400-e29b-41d4-a716-446655440000"),
                    either::Either::Left("f47ac10b-58cc-4372-a567-0e02b2c3d479"),
                    either::Either::Left("123e4567-e89b-12d3-a456-426614174000"),
                    either::Either::Left("987fbc97-4bed-5078-9f07-9141ba07c9f3"),
                    either::Either::Left("ffffffff-ffff-ffff-ffff-ffffffffffff"),
                    either::Either::Left("550E8400-E29B-41D4-A716-446655440000"),
                    either::Either::Left("00000001-0002-0003-0004-000000000005"),
                    either::Either::Left("6ba7b810-9dad-11d1-80b4-00c04fd430c8"),
                    either::Either::Left("3f2504e0-4f89-11d3-9a0c-0305e82c3301"),
                ]),
                false,
            )
        } else {
            // TODO: Set min-length, max-length
            schema_builder.describe_bytes(
                Some("A non-nil universally unique identifier (UUID)."),
                || {
                    Ok([
                        either::Either::Right(
                            &[
                                0x55u8, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44,
                                0x66, 0x55, 0x44, 0x00, 0x00,
                            ][..],
                        ),
                        either::Either::Right(
                            &[
                                0xf4u8, 0x7a, 0xc1, 0x0b, 0x58, 0xcc, 0x43, 0x72, 0xa5, 0x67, 0x0e,
                                0x02, 0xb2, 0xc3, 0xd4, 0x79,
                            ][..],
                        ),
                        either::Either::Right(
                            &[
                                0x12u8, 0x3e, 0x45, 0x67, 0xe8, 0x9b, 0x12, 0xd3, 0xa4, 0x56, 0x42,
                                0x66, 0x14, 0x17, 0x40, 0x00,
                            ][..],
                        ),
                        either::Either::Right(
                            &[
                                0x98u8, 0x7f, 0xbc, 0x97, 0x4b, 0xed, 0x50, 0x78, 0x9f, 0x07, 0x91,
                                0x41, 0xba, 0x07, 0xc9, 0xf3,
                            ][..],
                        ),
                        either::Either::Right(
                            &[
                                0xffu8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                                0xff, 0xff, 0xff, 0xff, 0xff,
                            ][..],
                        ),
                        either::Either::Right(
                            &[
                                0x55u8, 0x0E, 0x84, 0x00, 0xE2, 0x9B, 0x41, 0xD4, 0xA7, 0x16, 0x44,
                                0x66, 0x55, 0x44, 0x00, 0x00,
                            ][..],
                        ),
                        either::Either::Right(
                            &[
                                0x00u8, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00,
                                0x00, 0x00, 0x00, 0x00, 0x05,
                            ][..],
                        ),
                        either::Either::Right(
                            &[
                                0x6bu8, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00,
                                0xc0, 0x4f, 0xd4, 0x30, 0xc8,
                            ][..],
                        ),
                        either::Either::Right(
                            &[
                                0x3fu8, 0x25, 0x04, 0xe0, 0x4f, 0x89, 0x11, 0xd3, 0x9a, 0x0c, 0x03,
                                0x05, 0xe8, 0x2c, 0x33, 0x01,
                            ][..],
                        ),
                    ])
                },
                false,
            )
        }
    }
}

impl Schema for Hyphenated {
    type Example = &'static str;
    type Examples = <[Self::Example; 10] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
                Some(36),
                Some(36),
                Some(r"^([0-9a-fA-F]{8}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{12})$"),
                None,
                None,
                Some("A universally unique identifier (UUID) in hyphenated notation."),
                || Ok([
                    "550e8400-e29b-41d4-a716-446655440000",
                    "f47ac10b-58cc-4372-a567-0e02b2c3d479",
                    "123e4567-e89b-12d3-a456-426614174000",
                    "987fbc97-4bed-5078-9f07-9141ba07c9f3",
                    "00000000-0000-0000-0000-000000000000",
                    "ffffffff-ffff-ffff-ffff-ffffffffffff",
                    "550E8400-E29B-41D4-A716-446655440000",
                    "00000001-0002-0003-0004-000000000005",
                    "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
                    "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
                ]),
                false,
            )
    }
}

impl Schema for Simple {
    type Example = &'static str;
    type Examples = <[Self::Example; 10] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            Some(32),
            Some(32),
            Some(r"^[0-9a-fA-F]{32}$"),
            None,
            None,
            Some("A universally unique identifier (UUID) in simple notation."),
            || {
                Ok([
                    "550e8400e29b41d4a716446655440000",
                    "f47ac10b58cc4372a5670e02b2c3d479",
                    "123e4567e89b12d3a456426614174000",
                    "987fbc974bed50789f079141ba07c9f3",
                    "00000000000000000000000000000000",
                    "ffffffffffffffffffffffffffffffff",
                    "550E8400E29B41D4A716446655440000",
                    "00000001000200030004000000000005",
                    "6ba7b8109dad11d180b400c04fd430c8",
                    "3f2504e04f8911d39a0c0305e82c3301",
                ])
            },
            false,
        )
    }
}

impl Schema for Urn {
    type Example = &'static str;
    type Examples = <[Self::Example; 10] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
                Some(45),
                Some(45),
                Some(r"^(urn:uuid:[0-9a-fA-F]{8}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{12})$"),
                None,
                None,
                Some("A universally unique identifier (UUID) in URN notation."),
                || Ok([
                    "urn:uuid:550e8400-e29b-41d4-a716-446655440000",
                    "urn:uuid:f47ac10b-58cc-4372-a567-0e02b2c3d479",
                    "urn:uuid:123e4567-e89b-12d3-a456-426614174000",
                    "urn:uuid:987fbc97-4bed-5078-9f07-9141ba07c9f3",
                    "urn:uuid:00000000-0000-0000-0000-000000000000",
                    "urn:uuid:ffffffff-ffff-ffff-ffff-ffffffffffff",
                    "urn:uuid:550E8400-E29B-41D4-A716-446655440000",
                    "urn:uuid:00000001-0002-0003-0004-000000000005",
                    "urn:uuid:6ba7b810-9dad-11d1-80b4-00c04fd430c8",
                    "urn:uuid:3f2504e0-4f89-11d3-9a0c-0305e82c3301",
                ]),
                false,
            )
    }
}

impl Schema for Braced {
    type Example = &'static str;
    type Examples = <[Self::Example; 10] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
                Some(38),
                Some(38),
                Some(r"^({[0-9a-fA-F]{8}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{12}})$"),
                None,
                None,
                Some("A universally unique identifier (UUID) in braced notation."),
                || Ok([
                    "{550e8400-e29b-41d4-a716-446655440000}",
                    "{f47ac10b-58cc-4372-a567-0e02b2c3d479}",
                    "{123e4567-e89b-12d3-a456-426614174000}",
                    "{987fbc97-4bed-5078-9f07-9141ba07c9f3}",
                    "{00000000-0000-0000-0000-000000000000}",
                    "{ffffffff-ffff-ffff-ffff-ffffffffffff}",
                    "{550E8400-E29B-41D4-A716-446655440000}",
                    "{00000001-0002-0003-0004-000000000005}",
                    "{6ba7b810-9dad-11d1-80b4-00c04fd430c8}",
                    "{3f2504e0-4f89-11d3-9a0c-0305e82c3301}",
                ]),
                false,
            )
    }
}
