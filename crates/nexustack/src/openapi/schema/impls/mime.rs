/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::openapi::Schema;
use mime::Mime;

impl Schema for Mime {
    type Example = &'static str;
    type Examples = <[Self::Example; 4] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            Some(r"^((\w+\/((\w[-+.\w]*)|\*))|(\*\/\*))(;\s*\w+\s*=[^;]+)*$"),
            None,
            None,
            Some("An RFC 6838 internet media type with optional parameters."),
            || {
                Ok([
                    "application/zip",
                    "video/*",
                    "*/*",
                    "application/soap+xml; charset=utf-8",
                ])
            },
            false,
        )
    }
}
