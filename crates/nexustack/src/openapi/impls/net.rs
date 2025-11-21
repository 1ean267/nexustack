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
            CombinatorSchemaBuilder, EnumSchemaBuilder, SchemaBuilder, SchemaId,
            TupleSchemaBuilder, VariantTag,
        },
    },
};

const IPV4_ADDR_SEGMENT_REGEX: &str = r"(?:25[0-5]|2[0-4]\d|1\d{2}|[1-9]\d|\d)";
const IPV4_ADDR_REGEX: &str =
    const_format::formatcp!(r"(?:{IPV4_ADDR_SEGMENT_REGEX}\.){{3}}{IPV4_ADDR_SEGMENT_REGEX}");

// https://stackoverflow.com/a/17871737/5768018
const IPV6SEG: &str = r"[0-9a-fA-F]{1,4}";
const IPV6_ADDR_REGEX: &str = const_format::formatcp!(
    r"(?:(?:{IPV6SEG}:){{7}}{IPV6SEG}|(?:{IPV6SEG}:){{1,7}}:|(?:{IPV6SEG}:){{1,6}}:{IPV6SEG}|(?:{IPV6SEG}:){{1,5}}(?::{IPV6SEG}){{1,2}}|(?:{IPV6SEG}:){{1,4}}(?::{IPV6SEG}){{1,3}}|(?:{IPV6SEG}:){{1,3}}(?::{IPV6SEG}){{1,4}}|(?:{IPV6SEG}:){{1,2}}(?::{IPV6SEG}){{1,5}}|{IPV6SEG}:(?:(?::{IPV6SEG}){{1,6}})|:(?:(?::{IPV6SEG}){{1,7}}|:)|fe80:(?::[0-9a-fA-F]{{0,4}}){{0,4}}%[0-9a-zA-Z]{{1,}}|::(?:ffff(?::0{{1,4}}){{0,1}}:){{0,1}}{IPV4_ADDR_REGEX}|(?:{IPV6SEG}:){{1,4}}:{IPV4_ADDR_REGEX})"
);
const U16_REGEX: &str =
    r"(?:(?:6553[0-5])|(?:655[0-2]\d)|(?:65[0-4]\d{2})|(?:6[0-4]\d{3})|(?:[0-5]\d{4})|(?:\d{1,4}))";
const U32_REGEX: &str = r"(?:(?:429496729[0-5])|(?:42949672[0-8]\d)|(?:4294967[0-1]\d{2})|(?:429496[0-6]\d{3})|(?:42949[0-5]\d{4})|(?:4294[0-8]\d{5})|(?:429[0-3]\d{6})|(?:42[0-8]\d{7})|(?:4[0-1]\d{8})|(?:[0-3]\d{9})|(?:\d{1,9}))";
const IPV4_SOCKET_ADDR_REGEX: &str = const_format::formatcp!("(?:{IPV4_ADDR_REGEX}:{U16_REGEX})");
const IPV6_SOCKET_ADDR_REGEX: &str =
    const_format::formatcp!(r"(?:\[{IPV6_ADDR_REGEX}(?:%{U32_REGEX})?\]:{U16_REGEX})");

callsite!(IpAddrCallsite);
callsite!(IpAddrV4VariantCallsite);
callsite!(IpAddrV6VariantCallsite);

impl Schema for std::net::IpAddr {
    type Example = Self;
    type Examples = std::iter::Chain<
        std::iter::Map<
            <std::net::Ipv4Addr as Schema>::Examples,
            fn(<std::net::Ipv4Addr as Schema>::Example) -> Self::Example,
        >,
        std::iter::Map<
            <std::net::Ipv6Addr as Schema>::Examples,
            fn(<std::net::Ipv6Addr as Schema>::Example) -> Self::Example,
        >,
    >;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        if schema_builder.is_human_readable() {
            let mut variant_builder = schema_builder.describe_one_of(
                2,
                Some("An IP address"),
                || {
                    Ok(std::net::Ipv4Addr::examples(true)?
                        .map(std::net::IpAddr::V4 as _)
                        .chain(std::net::Ipv6Addr::examples(true)?.map(std::net::IpAddr::V6 as _)))
                },
                false,
            )?;
            variant_builder.collect_subschema(
                None,
                false,
                <std::net::Ipv4Addr as Schema>::describe,
            )?;
            variant_builder.collect_subschema(
                None,
                false,
                <std::net::Ipv6Addr as Schema>::describe,
            )?;
            variant_builder.end()
        } else {
            let mut enum_schema_builder = schema_builder.describe_enum(
                Some(SchemaId::new("IpAddr", *IpAddrCallsite)),
                2,
                true,
                VariantTag::default(),
                Some("An IP address"),
                || {
                    Ok(std::net::Ipv4Addr::examples(false)?
                        .map(std::net::IpAddr::V4 as _)
                        .chain(std::net::Ipv6Addr::examples(false)?.map(std::net::IpAddr::V6 as _)))
                },
                false,
            )?;
            enum_schema_builder.collect_newtype_variant(
                0,
                SchemaId::new("V4", *IpAddrV4VariantCallsite),
                None,
                false,
                <std::net::Ipv4Addr as Schema>::describe,
            )?;
            enum_schema_builder.collect_newtype_variant(
                1,
                SchemaId::new("V6", *IpAddrV6VariantCallsite),
                None,
                false,
                <std::net::Ipv6Addr as Schema>::describe,
            )?;
            enum_schema_builder.end()
        }
    }
}

impl Schema for std::net::Ipv4Addr {
    type Example = Self;
    type Examples = <[Self::Example; 2] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        let description =
            "An IPv4 address according to [IETF RFC 791](https://tools.ietf.org/html/rfc791)";
        let examples = || {
            Ok([
                <Self as std::str::FromStr>::from_str("1.2.3.4").unwrap(),
                <Self as std::str::FromStr>::from_str("101.102.103.104").unwrap(),
            ])
        };

        if schema_builder.is_human_readable() {
            const MIN_LEN: usize = "0.0.0.0".len();
            const MAX_LEN: usize = "255.255.255.255".len();
            const REGEX: &str = const_format::formatcp!("^{IPV4_ADDR_REGEX}$");

            schema_builder.describe_str(
                Some(MIN_LEN),
                Some(MAX_LEN),
                Some(REGEX),
                Some("ipv4"),
                None,
                Some(description),
                examples,
                false,
            )
        } else {
            let mut tuple_schema_builder =
                schema_builder.describe_tuple(2, Some(description), examples, false)?;

            for _ in 0..4 {
                tuple_schema_builder.collect_element(None, false, <u8 as Schema>::describe)?;
            }

            tuple_schema_builder.end()
        }
    }
}

impl Schema for std::net::Ipv6Addr {
    type Example = Self;
    type Examples = <[Self::Example; 13] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        let description =
            "An IPv6 address according to [IETF RFC 4291](https://tools.ietf.org/html/rfc4291)";
        let examples = || {
            Ok([
                <Self as std::str::FromStr>::from_str("2001:db8:3333:4444:5555:6666:7777:8888")
                    .unwrap(),
                <Self as std::str::FromStr>::from_str("2001:db8:3333:4444:CCCC:DDDD:EEEE:FFFF")
                    .unwrap(),
                <Self as std::str::FromStr>::from_str("::").unwrap(),
                <Self as std::str::FromStr>::from_str("2001:db8::").unwrap(),
                <Self as std::str::FromStr>::from_str("::1234:5678").unwrap(),
                <Self as std::str::FromStr>::from_str("2001:db8::1234:5678").unwrap(),
                <Self as std::str::FromStr>::from_str("2001:0db8:0001:0000:0000:0ab9:C0A8:0102")
                    .unwrap(),
                <Self as std::str::FromStr>::from_str("2001:db8:3333:4444:5555:6666:1.2.3.4")
                    .unwrap(),
                <Self as std::str::FromStr>::from_str("::11.22.33.44").unwrap(),
                <Self as std::str::FromStr>::from_str("2001:db8::123.123.123.123").unwrap(),
                <Self as std::str::FromStr>::from_str("::1234:5678:91.123.4.56").unwrap(),
                <Self as std::str::FromStr>::from_str("::1234:5678:1.2.3.4").unwrap(),
                <Self as std::str::FromStr>::from_str("2001:db8::1234:5678:5.6.7.8").unwrap(),
            ])
        };

        if schema_builder.is_human_readable() {
            const MIN_LEN: usize = "::".len();
            const MAX_LEN: usize = "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff".len();
            const REGEX: &str = const_format::formatcp!("^{IPV6_ADDR_REGEX}$");

            schema_builder.describe_str(
                Some(MIN_LEN),
                Some(MAX_LEN),
                Some(REGEX),
                Some("ipv6"),
                None,
                Some(description),
                examples,
                false,
            )
        } else {
            let mut tuple_schema_builder =
                schema_builder.describe_tuple(2, Some(description), examples, false)?;

            for _ in 0..16 {
                tuple_schema_builder.collect_element(None, false, <u8 as Schema>::describe)?;
            }

            tuple_schema_builder.end()
        }
    }
}

callsite!(SocketAddrCallsite);
callsite!(SocketAddrV4VariantCallsite);
callsite!(SocketAddrV6VariantCallsite);

impl Schema for std::net::SocketAddr {
    type Example = Self;
    type Examples = std::iter::Chain<
        std::iter::Map<
            <std::net::SocketAddrV4 as Schema>::Examples,
            fn(<std::net::SocketAddrV4 as Schema>::Example) -> Self::Example,
        >,
        std::iter::Map<
            <std::net::SocketAddrV6 as Schema>::Examples,
            fn(<std::net::SocketAddrV6 as Schema>::Example) -> Self::Example,
        >,
    >;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        let is_human_readable = schema_builder.is_human_readable();
        let description =
            "An IP socket address consisting of an IP address and a 16-bit port number";
        let examples = || {
            Ok(
                <std::net::SocketAddrV4 as SchemaExamples>::examples(is_human_readable)?
                    .map(std::net::SocketAddr::V4 as _)
                    .chain(
                        <std::net::SocketAddrV6 as SchemaExamples>::examples(is_human_readable)?
                            .map(std::net::SocketAddr::V6 as _),
                    ),
            )
        };
        if is_human_readable {
            let mut enum_schema_builder =
                schema_builder.describe_one_of(2, Some(description), examples, false)?;
            enum_schema_builder.collect_subschema(
                None,
                false,
                <std::net::SocketAddrV4 as Schema>::describe,
            )?;
            enum_schema_builder.collect_subschema(
                None,
                false,
                <std::net::SocketAddrV6 as Schema>::describe,
            )?;
            enum_schema_builder.end()
        } else {
            let mut enum_schema_builder = schema_builder.describe_enum(
                Some(SchemaId::new("SocketAddr", *SocketAddrCallsite)),
                2,
                true,
                VariantTag::default(),
                Some(description),
                examples,
                false,
            )?;
            enum_schema_builder.collect_newtype_variant(
                0,
                SchemaId::new("V4", *SocketAddrV4VariantCallsite),
                None,
                false,
                <std::net::SocketAddrV4 as Schema>::describe,
            )?;
            enum_schema_builder.collect_newtype_variant(
                1,
                SchemaId::new("V6", *SocketAddrV6VariantCallsite),
                None,
                false,
                <std::net::SocketAddrV6 as Schema>::describe,
            )?;
            enum_schema_builder.end()
        }
    }
}

impl Schema for std::net::SocketAddrV4 {
    type Example = Self;
    type Examples = std::iter::Chain<
        std::iter::Map<<std::net::Ipv4Addr as Schema>::Examples, fn(std::net::Ipv4Addr) -> Self>,
        std::iter::Map<<std::net::Ipv4Addr as Schema>::Examples, fn(std::net::Ipv4Addr) -> Self>,
    >;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        let description = "An IPv4 socket address consisting of an IPv4 address and a 16-bit port number, as stated in [IETF RFC 793](https://tools.ietf.org/html/rfc793)";
        let is_human_readable = schema_builder.is_human_readable();
        let examples = || {
            Ok(
                <std::net::Ipv4Addr as SchemaExamples>::examples(is_human_readable)?
                    .map((|ip| Self::new(ip, 80)) as _)
                    .chain(
                        <std::net::Ipv4Addr as SchemaExamples>::examples(is_human_readable)?
                            .map((|ip| Self::new(ip, 1234)) as _),
                    ),
            )
        };

        if is_human_readable {
            const MIN_LEN: usize = "0.0.0.0:0".len();
            const MAX_LEN: usize = "255.255.255.255:65535".len();
            const REGEX: &str = const_format::formatcp!("^{IPV4_SOCKET_ADDR_REGEX}$");

            schema_builder.describe_str(
                Some(MIN_LEN),
                Some(MAX_LEN),
                Some(REGEX),
                None,
                None,
                Some(description),
                examples,
                false,
            )
        } else {
            let mut tuple_schema_builder =
                schema_builder.describe_tuple(2, Some(description), examples, false)?;

            tuple_schema_builder.collect_element(
                None,
                false,
                <std::net::Ipv4Addr as Schema>::describe,
            )?;
            tuple_schema_builder.collect_element(None, false, <u16 as Schema>::describe)?;

            tuple_schema_builder.end()
        }
    }
}

impl Schema for std::net::SocketAddrV6 {
    type Example = Self;
    type Examples = std::iter::Chain<
        std::iter::Map<
            <std::net::Ipv6Addr as Schema>::Examples,
            fn(std::net::Ipv6Addr) -> Self::Example,
        >,
        std::iter::Map<
            <std::net::Ipv6Addr as Schema>::Examples,
            fn(std::net::Ipv6Addr) -> Self::Example,
        >,
    >;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        let description = "An IPv6 socket address consisting of an IPv6 socket addresses and a 16-bit port number according to [IETF RFC 2553, Section 3.3](https://tools.ietf.org/html/rfc2553#section-3.3)";
        let is_human_readable = schema_builder.is_human_readable();
        let examples = || {
            Ok(
                <std::net::Ipv6Addr as SchemaExamples>::examples(is_human_readable)?
                    .map((|ip| Self::new(ip, 80, 0, 0)) as _)
                    .chain(
                        <std::net::Ipv6Addr as SchemaExamples>::examples(is_human_readable)?
                            .map((|ip| Self::new(ip, 1234, 0, 0)) as _),
                    ),
            )
        };

        if is_human_readable {
            const MIN_LEN: usize = "[::]:0".len();
            const MAX_LEN: usize =
                "[ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff%4294967295]:65000".len();
            const REGEX: &str = const_format::formatcp!("^{IPV6_SOCKET_ADDR_REGEX}$");

            schema_builder.describe_str(
                Some(MIN_LEN),
                Some(MAX_LEN),
                Some(REGEX),
                None,
                None,
                Some(description),
                examples,
                false,
            )
        } else {
            let mut tuple_schema_builder =
                schema_builder.describe_tuple(2, Some(description), examples, false)?;

            tuple_schema_builder.collect_element(
                None,
                false,
                <std::net::Ipv6Addr as Schema>::describe,
            )?;
            tuple_schema_builder.collect_element(None, false, <u16 as Schema>::describe)?;

            tuple_schema_builder.end()
        }
    }
}

mod test {
    #[test]
    fn test_ip_addr_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<std::net::IpAddr>(
            Specification::OpenAPI3_1,
            schema_collection.clone(),
        )
        .unwrap();

        let schemas_object = Rc::try_unwrap(schema_collection)
            .map_err(|_| "Should be the only Rc strong reference")
            .unwrap()
            .into_inner()
            .to_schemas_object();

        pretty_assertions::assert_eq!(
            serde_json::to_value(schema).unwrap(),
            serde_json::json!(
                {
                    "description": "An IP address",
                    "examples": [
                        "1.2.3.4",
                        "101.102.103.104",
                        "2001:db8:3333:4444:5555:6666:7777:8888",
                        "2001:db8:3333:4444:cccc:dddd:eeee:ffff",
                        "::",
                        "2001:db8::",
                        "::1234:5678",
                        "2001:db8::1234:5678",
                        "2001:db8:1::ab9:c0a8:102",
                        "2001:db8:3333:4444:5555:6666:102:304",
                        "::b16:212c",
                        "2001:db8::7b7b:7b7b",
                        "::1234:5678:5b7b:438",
                        "::1234:5678:102:304",
                        "2001:db8::1234:5678:506:708",
                    ],
                    "oneOf": [
                        {
                            "description": "An IPv4 address according to [IETF RFC 791](https://tools.ietf.org/html/rfc791)",
                            "examples": [
                                "1.2.3.4",
                                "101.102.103.104",
                            ],
                            "format": "ipv4",
                            "maxLength": 15,
                            "minLength": 7,
                            "pattern": "^(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)$",
                            "type": "string",
                        },
                        {
                            "description": "An IPv6 address according to [IETF RFC 4291](https://tools.ietf.org/html/rfc4291)",
                            "examples": [
                                "2001:db8:3333:4444:5555:6666:7777:8888",
                                "2001:db8:3333:4444:cccc:dddd:eeee:ffff",
                                "::",
                                "2001:db8::",
                                "::1234:5678",
                                "2001:db8::1234:5678",
                                "2001:db8:1::ab9:c0a8:102",
                                "2001:db8:3333:4444:5555:6666:102:304",
                                "::b16:212c",
                                "2001:db8::7b7b:7b7b",
                                "::1234:5678:5b7b:438",
                                "::1234:5678:102:304",
                                "2001:db8::1234:5678:506:708",
                            ],
                            "format": "ipv6",
                            "maxLength": 39,
                            "minLength": 2,
                            "pattern": "^(?:(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,7}:|(?:[0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,5}(?::[0-9a-fA-F]{1,4}){1,2}|(?:[0-9a-fA-F]{1,4}:){1,4}(?::[0-9a-fA-F]{1,4}){1,3}|(?:[0-9a-fA-F]{1,4}:){1,3}(?::[0-9a-fA-F]{1,4}){1,4}|(?:[0-9a-fA-F]{1,4}:){1,2}(?::[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:(?:(?::[0-9a-fA-F]{1,4}){1,6})|:(?:(?::[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(?::[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(?:ffff(?::0{1,4}){0,1}:){0,1}(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)|(?:[0-9a-fA-F]{1,4}:){1,4}:(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d))$",
                            "type": "string",
                        },
                    ],
                }
            )
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    fn test_ip_v4_addr_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<std::net::Ipv4Addr>(
            Specification::OpenAPI3_1,
            schema_collection.clone(),
        )
        .unwrap();

        let schemas_object = Rc::try_unwrap(schema_collection)
            .map_err(|_| "Should be the only Rc strong reference")
            .unwrap()
            .into_inner()
            .to_schemas_object();

        pretty_assertions::assert_eq!(
            serde_json::to_value(schema).unwrap(),
            serde_json::json!(
                {
                    "type": "string",
                    "description": "An IPv4 address according to [IETF RFC 791](https://tools.ietf.org/html/rfc791)",
                    "format": "ipv4",
                    "maxLength": 15,
                    "minLength": 7,
                    "pattern": "^(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)$",
                    "examples": [
                        "1.2.3.4",
                        "101.102.103.104",
                    ],
                }
            )
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    fn test_ip_v6_addr_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<std::net::Ipv6Addr>(
            Specification::OpenAPI3_1,
            schema_collection.clone(),
        )
        .unwrap();

        let schemas_object = Rc::try_unwrap(schema_collection)
            .map_err(|_| "Should be the only Rc strong reference")
            .unwrap()
            .into_inner()
            .to_schemas_object();

        pretty_assertions::assert_eq!(
            serde_json::to_value(schema).unwrap(),
            serde_json::json!(
                {
                    "type": "string",
                    "description": "An IPv6 address according to [IETF RFC 4291](https://tools.ietf.org/html/rfc4291)",
                    "examples": [
                        "2001:db8:3333:4444:5555:6666:7777:8888",
                        "2001:db8:3333:4444:cccc:dddd:eeee:ffff",
                        "::",
                        "2001:db8::",
                        "::1234:5678",
                        "2001:db8::1234:5678",
                        "2001:db8:1::ab9:c0a8:102",
                        "2001:db8:3333:4444:5555:6666:102:304",
                        "::b16:212c",
                        "2001:db8::7b7b:7b7b",
                        "::1234:5678:5b7b:438",
                        "::1234:5678:102:304",
                        "2001:db8::1234:5678:506:708",
                    ],
                    "format": "ipv6",
                    "maxLength": 39,
                    "minLength": 2,
                    "pattern": "^(?:(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,7}:|(?:[0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,5}(?::[0-9a-fA-F]{1,4}){1,2}|(?:[0-9a-fA-F]{1,4}:){1,4}(?::[0-9a-fA-F]{1,4}){1,3}|(?:[0-9a-fA-F]{1,4}:){1,3}(?::[0-9a-fA-F]{1,4}){1,4}|(?:[0-9a-fA-F]{1,4}:){1,2}(?::[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:(?:(?::[0-9a-fA-F]{1,4}){1,6})|:(?:(?::[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(?::[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(?:ffff(?::0{1,4}){0,1}:){0,1}(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)|(?:[0-9a-fA-F]{1,4}:){1,4}:(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d))$",
                }
            )
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_socket_addr_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<std::net::SocketAddr>(
            Specification::OpenAPI3_1,
            schema_collection.clone(),
        )
        .unwrap();

        let schemas_object = Rc::try_unwrap(schema_collection)
            .map_err(|_| "Should be the only Rc strong reference")
            .unwrap()
            .into_inner()
            .to_schemas_object();

        pretty_assertions::assert_eq!(
            serde_json::to_value(schema).unwrap(),
            serde_json::json!(
                {
                    "description": "An IP socket address consisting of an IP address and a 16-bit port number",
                    "examples": [
                        "1.2.3.4:80",
                        "101.102.103.104:80",
                        "1.2.3.4:1234",
                        "101.102.103.104:1234",
                        "[2001:db8:3333:4444:5555:6666:7777:8888]:80",
                        "[2001:db8:3333:4444:cccc:dddd:eeee:ffff]:80",
                        "[::]:80",
                        "[2001:db8::]:80",
                        "[::1234:5678]:80",
                        "[2001:db8::1234:5678]:80",
                        "[2001:db8:1::ab9:c0a8:102]:80",
                        "[2001:db8:3333:4444:5555:6666:102:304]:80",
                        "[::b16:212c]:80",
                        "[2001:db8::7b7b:7b7b]:80",
                        "[::1234:5678:5b7b:438]:80",
                        "[::1234:5678:102:304]:80",
                        "[2001:db8::1234:5678:506:708]:80",
                        "[2001:db8:3333:4444:5555:6666:7777:8888]:1234",
                        "[2001:db8:3333:4444:cccc:dddd:eeee:ffff]:1234",
                        "[::]:1234",
                        "[2001:db8::]:1234",
                        "[::1234:5678]:1234",
                        "[2001:db8::1234:5678]:1234",
                        "[2001:db8:1::ab9:c0a8:102]:1234",
                        "[2001:db8:3333:4444:5555:6666:102:304]:1234",
                        "[::b16:212c]:1234",
                        "[2001:db8::7b7b:7b7b]:1234",
                        "[::1234:5678:5b7b:438]:1234",
                        "[::1234:5678:102:304]:1234",
                        "[2001:db8::1234:5678:506:708]:1234",
                    ],
                    "oneOf": [
                        {
                            "description": "An IPv4 socket address consisting of an IPv4 address and a 16-bit port number, as stated in [IETF RFC 793](https://tools.ietf.org/html/rfc793)",
                            "examples": [
                                "1.2.3.4:80",
                                "101.102.103.104:80",
                                "1.2.3.4:1234",
                                "101.102.103.104:1234",
                            ],
                            "maxLength": 21,
                            "minLength": 9,
                            "pattern": "^(?:(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d):(?:(?:6553[0-5])|(?:655[0-2]\\d)|(?:65[0-4]\\d{2})|(?:6[0-4]\\d{3})|(?:[0-5]\\d{4})|(?:\\d{1,4})))$",
                            "type": "string",
                        },
                        {
                            "description": "An IPv6 socket address consisting of an IPv6 socket addresses and a 16-bit port number according to [IETF RFC 2553, Section 3.3](https://tools.ietf.org/html/rfc2553#section-3.3)",
                            "examples": [
                                "[2001:db8:3333:4444:5555:6666:7777:8888]:80",
                                "[2001:db8:3333:4444:cccc:dddd:eeee:ffff]:80",
                                "[::]:80",
                                "[2001:db8::]:80",
                                "[::1234:5678]:80",
                                "[2001:db8::1234:5678]:80",
                                "[2001:db8:1::ab9:c0a8:102]:80",
                                "[2001:db8:3333:4444:5555:6666:102:304]:80",
                                "[::b16:212c]:80",
                                "[2001:db8::7b7b:7b7b]:80",
                                "[::1234:5678:5b7b:438]:80",
                                "[::1234:5678:102:304]:80",
                                "[2001:db8::1234:5678:506:708]:80",
                                "[2001:db8:3333:4444:5555:6666:7777:8888]:1234",
                                "[2001:db8:3333:4444:cccc:dddd:eeee:ffff]:1234",
                                "[::]:1234",
                                "[2001:db8::]:1234",
                                "[::1234:5678]:1234",
                                "[2001:db8::1234:5678]:1234",
                                "[2001:db8:1::ab9:c0a8:102]:1234",
                                "[2001:db8:3333:4444:5555:6666:102:304]:1234",
                                "[::b16:212c]:1234",
                                "[2001:db8::7b7b:7b7b]:1234",
                                "[::1234:5678:5b7b:438]:1234",
                                "[::1234:5678:102:304]:1234",
                                "[2001:db8::1234:5678:506:708]:1234",
                            ],
                            "maxLength": 58,
                            "minLength": 6,
                            "pattern": "^(?:\\[(?:(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,7}:|(?:[0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,5}(?::[0-9a-fA-F]{1,4}){1,2}|(?:[0-9a-fA-F]{1,4}:){1,4}(?::[0-9a-fA-F]{1,4}){1,3}|(?:[0-9a-fA-F]{1,4}:){1,3}(?::[0-9a-fA-F]{1,4}){1,4}|(?:[0-9a-fA-F]{1,4}:){1,2}(?::[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:(?:(?::[0-9a-fA-F]{1,4}){1,6})|:(?:(?::[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(?::[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(?:ffff(?::0{1,4}){0,1}:){0,1}(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)|(?:[0-9a-fA-F]{1,4}:){1,4}:(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d))(?:%(?:(?:429496729[0-5])|(?:42949672[0-8]\\d)|(?:4294967[0-1]\\d{2})|(?:429496[0-6]\\d{3})|(?:42949[0-5]\\d{4})|(?:4294[0-8]\\d{5})|(?:429[0-3]\\d{6})|(?:42[0-8]\\d{7})|(?:4[0-1]\\d{8})|(?:[0-3]\\d{9})|(?:\\d{1,9})))?\\]:(?:(?:6553[0-5])|(?:655[0-2]\\d)|(?:65[0-4]\\d{2})|(?:6[0-4]\\d{3})|(?:[0-5]\\d{4})|(?:\\d{1,4})))$",
                            "type": "string",
                        },
                    ],
                }
            )
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    fn test_socket_addr_v4_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<std::net::SocketAddrV4>(
            Specification::OpenAPI3_1,
            schema_collection.clone(),
        )
        .unwrap();

        let schemas_object = Rc::try_unwrap(schema_collection)
            .map_err(|_| "Should be the only Rc strong reference")
            .unwrap()
            .into_inner()
            .to_schemas_object();

        pretty_assertions::assert_eq!(
            serde_json::to_value(schema).unwrap(),
            serde_json::json!(
                {
                    "type": "string",
                    "description": "An IPv4 socket address consisting of an IPv4 address and a 16-bit port number, as stated in [IETF RFC 793](https://tools.ietf.org/html/rfc793)",
                    "maxLength": 21,
                    "minLength": 9,
                    "pattern": "^(?:(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d):(?:(?:6553[0-5])|(?:655[0-2]\\d)|(?:65[0-4]\\d{2})|(?:6[0-4]\\d{3})|(?:[0-5]\\d{4})|(?:\\d{1,4})))$",
                    "examples": [
                        "1.2.3.4:80",
                        "101.102.103.104:80",
                        "1.2.3.4:1234",
                        "101.102.103.104:1234",
                    ],
                }
            )
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    fn test_socket_addr_v6_schema() {
        use crate::openapi::json::{SchemaCollection, Specification, build_schema_with_collection};
        use std::{cell::RefCell, rc::Rc};

        let schema_collection = Rc::new(RefCell::new(SchemaCollection::new()));

        #[allow(deprecated)]
        let schema = build_schema_with_collection::<std::net::SocketAddrV6>(
            Specification::OpenAPI3_1,
            schema_collection.clone(),
        )
        .unwrap();

        let schemas_object = Rc::try_unwrap(schema_collection)
            .map_err(|_| "Should be the only Rc strong reference")
            .unwrap()
            .into_inner()
            .to_schemas_object();

        pretty_assertions::assert_eq!(
            serde_json::to_value(schema).unwrap(),
            serde_json::json!(
                {
                    "type": "string",
                        "description": "An IPv6 socket address consisting of an IPv6 socket addresses and a 16-bit port number according to [IETF RFC 2553, Section 3.3](https://tools.ietf.org/html/rfc2553#section-3.3)",
                        "examples": [
                            "[2001:db8:3333:4444:5555:6666:7777:8888]:80",
                            "[2001:db8:3333:4444:cccc:dddd:eeee:ffff]:80",
                            "[::]:80",
                            "[2001:db8::]:80",
                            "[::1234:5678]:80",
                            "[2001:db8::1234:5678]:80",
                            "[2001:db8:1::ab9:c0a8:102]:80",
                            "[2001:db8:3333:4444:5555:6666:102:304]:80",
                            "[::b16:212c]:80",
                            "[2001:db8::7b7b:7b7b]:80",
                            "[::1234:5678:5b7b:438]:80",
                            "[::1234:5678:102:304]:80",
                            "[2001:db8::1234:5678:506:708]:80",
                            "[2001:db8:3333:4444:5555:6666:7777:8888]:1234",
                            "[2001:db8:3333:4444:cccc:dddd:eeee:ffff]:1234",
                            "[::]:1234",
                            "[2001:db8::]:1234",
                            "[::1234:5678]:1234",
                            "[2001:db8::1234:5678]:1234",
                            "[2001:db8:1::ab9:c0a8:102]:1234",
                            "[2001:db8:3333:4444:5555:6666:102:304]:1234",
                            "[::b16:212c]:1234",
                            "[2001:db8::7b7b:7b7b]:1234",
                            "[::1234:5678:5b7b:438]:1234",
                            "[::1234:5678:102:304]:1234",
                            "[2001:db8::1234:5678:506:708]:1234",
                        ],
                        "maxLength": 58,
                        "minLength": 6,
                        "pattern": "^(?:\\[(?:(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,7}:|(?:[0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,5}(?::[0-9a-fA-F]{1,4}){1,2}|(?:[0-9a-fA-F]{1,4}:){1,4}(?::[0-9a-fA-F]{1,4}){1,3}|(?:[0-9a-fA-F]{1,4}:){1,3}(?::[0-9a-fA-F]{1,4}){1,4}|(?:[0-9a-fA-F]{1,4}:){1,2}(?::[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:(?:(?::[0-9a-fA-F]{1,4}){1,6})|:(?:(?::[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(?::[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(?:ffff(?::0{1,4}){0,1}:){0,1}(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)|(?:[0-9a-fA-F]{1,4}:){1,4}:(?:(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d)\\.){3}(?:25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]\\d|\\d))(?:%(?:(?:429496729[0-5])|(?:42949672[0-8]\\d)|(?:4294967[0-1]\\d{2})|(?:429496[0-6]\\d{3})|(?:42949[0-5]\\d{4})|(?:4294[0-8]\\d{5})|(?:429[0-3]\\d{6})|(?:42[0-8]\\d{7})|(?:4[0-1]\\d{8})|(?:[0-3]\\d{9})|(?:\\d{1,9})))?\\]:(?:(?:6553[0-5])|(?:655[0-2]\\d)|(?:65[0-4]\\d{2})|(?:6[0-4]\\d{3})|(?:[0-5]\\d{4})|(?:\\d{1,4})))$",
                }
            )
        );
        pretty_assertions::assert_eq!(
            serde_json::to_value(schemas_object).unwrap(),
            serde_json::json!({})
        );
    }
}
