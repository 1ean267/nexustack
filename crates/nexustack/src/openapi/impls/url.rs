/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use std::net::{Ipv4Addr, Ipv6Addr};

use crate::{
    callsite,
    openapi::{EnumSchemaBuilder as _, Schema, SchemaExamples, SchemaId, VariantTag},
};
use url::{Host, Url};

impl Schema for Url {
    type Example = &'static str;
    type Examples = <[Self::Example; 10] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            Some(2),
            None,
            Some(r"^(([^:\/?#\r\n\s]+):(\/\/([^\/?#\r\n\s]*))?([^?#\r\n\s]*)(\?([^#\r\n\s]*))?(#([^\r\n\s]*))?)$"),
            Some("uri"),
            None,
            Some("A uniform resource locator (URL)."),
            || {
                Ok([
                    "http://example.com",
                    "https://subdomain.example.com",
                    "http://example.com:8080",
                    "https://example.com/path/to/resource",
                    "https://example.com/search?q=query&lang=en",
                    "https://example.com/page#section",
                    "https://user:password@example.com",
                    "http://192.168.1.1",
                    "http://[2001:db8::ff00:42:8329]",
                    "ftp://ftp.example.com/resource",
                ])
            },
            false,
        )
    }
}

impl<S> Schema for Host<S>
where
    S: Schema,
{
    type Example = Host<<S as Schema>::Example>;
    type Examples = std::iter::Chain<
        std::iter::Chain<
            std::iter::Map<<S as Schema>::Examples, fn(<S as Schema>::Example) -> Self::Example>,
            std::iter::Map<
                <Ipv4Addr as Schema>::Examples,
                fn(<Ipv4Addr as Schema>::Example) -> Self::Example,
            >,
        >,
        std::iter::Map<
            <Ipv6Addr as Schema>::Examples,
            fn(<Ipv6Addr as Schema>::Example) -> Self::Example,
        >,
    >;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        let is_human_readable = schema_builder.is_human_readable();
        let mut enum_builder = schema_builder.describe_enum(
            Some(SchemaId::new("Host", callsite!())),
            3,
            false,
            VariantTag::ExternallyTagged,
            Some("The host name of an URL."),
            || {
                Ok(Iterator::chain(
                    Iterator::chain(
                        Iterator::map(
                            <S as SchemaExamples>::examples(is_human_readable)?,
                            Host::Domain as _,
                        ),
                        Iterator::map(
                            <Ipv4Addr as SchemaExamples>::examples(is_human_readable)?,
                            Host::Ipv4 as _,
                        ),
                    ),
                    Iterator::map(
                        <Ipv6Addr as SchemaExamples>::examples(is_human_readable)?,
                        Host::Ipv6 as _,
                    ),
                ))
            },
            false,
        )?;

        enum_builder.collect_newtype_variant(
            0,
            SchemaId::new("Domain", callsite!()),
            Some("A DNS domain name."),
            false,
            <S as Schema>::describe,
        )?;

        enum_builder.collect_newtype_variant(
            0,
            SchemaId::new("Ipv4", callsite!()),
            Some("An IPv4 address."),
            false,
            <Ipv4Addr as Schema>::describe,
        )?;

        enum_builder.collect_newtype_variant(
            0,
            SchemaId::new("Ipv6", callsite!()),
            Some("An IPv6 address."),
            false,
            <Ipv6Addr as Schema>::describe,
        )?;

        enum_builder.end()
    }
}
