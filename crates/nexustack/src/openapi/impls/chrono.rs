/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::openapi::Schema;
use chrono::{DateTime, TimeZone};

impl<Tz: TimeZone> Schema for DateTime<Tz> {
    type Example = &'static str;
    type Examples = <[&'static str; 10] as IntoIterator>::IntoIter;

    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            Some(19),
            None,
            Some(r"^((?:(\d{4}-\d{2}-\d{2})T(\d{2}:\d{2}:\d{2}(?:\.\d+)?))(Z|[\+-]\d{2}:\d{2})?)$"),
            Some("date-time"),
            None,
            Some("A timestamp in RFC 3339 format."),
            || {
                Ok([
                    "2025-10-21T15:30:45Z",
                    "2025-10-21T15:30:45+02:00",
                    "2025-10-21T15:30:45-05:00",
                    "2025-10-21T15:30:45.123Z",
                    "2025-10-21T15:30:45.123456+02:00",
                    "2025-10-21T15:30:45.123456789-05:00",
                    "2025-10-21T15:30:45.0Z",
                    "2025-10-21T15:30:45.000+00:00",
                    "2025-10-21T15:30:45.123+14:00",
                    "2025-10-21T15:30:45.123-12:00",
                ])
            },
            false,
        )
    }
}
