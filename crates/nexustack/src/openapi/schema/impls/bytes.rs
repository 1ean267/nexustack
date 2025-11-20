/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::openapi::Schema;
use bytes::{Buf, Bytes, BytesMut, buf::Chain};

impl Schema for Bytes {
    type Example = <[u8] as Schema>::Example;
    type Examples = <[u8] as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::schema::builder::SchemaBuilder<Self::Examples>,
    {
        <[u8] as Schema>::describe(schema_builder)
    }
}

impl Schema for BytesMut {
    type Example = <[u8] as Schema>::Example;
    type Examples = <[u8] as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::schema::builder::SchemaBuilder<Self::Examples>,
    {
        <[u8] as Schema>::describe(schema_builder)
    }
}

impl<T, U> Schema for Chain<T, U>
where
    T: Buf + Unpin + Send + 'static,
    U: Buf + Unpin + Send + 'static,
{
    type Example = <[u8] as Schema>::Example;
    type Examples = <[u8] as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::schema::builder::SchemaBuilder<Self::Examples>,
    {
        <[u8] as Schema>::describe(schema_builder)
    }
}
