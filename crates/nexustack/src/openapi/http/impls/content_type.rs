/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use bytes::{Buf, Bytes, BytesMut, buf::Chain};

use crate::openapi::{HttpContentType, HttpContentTypeBuilder, Schema};
use std::borrow::Cow;

impl HttpContentType for &'static str {
    #[inline]
    fn describe<B>(content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpContentTypeBuilder,
    {
        <Cow<'static, str> as HttpContentType>::describe(content_type_builder)
    }
}

impl HttpContentType for String {
    #[inline]
    fn describe<B>(content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpContentTypeBuilder,
    {
        <Cow<'static, str> as HttpContentType>::describe(content_type_builder)
    }
}

impl HttpContentType for Box<str> {
    #[inline]
    fn describe<B>(content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpContentTypeBuilder,
    {
        <Cow<'static, str> as HttpContentType>::describe(content_type_builder)
    }
}

impl HttpContentType for Cow<'static, str> {
    #[inline]
    fn describe<B>(mut content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpContentTypeBuilder,
    {
        content_type_builder.collect_content_type(
            "text/plain; charset=utf-8",
            None,
            false,
            <Cow<'static, str> as Schema>::describe,
        )?;
        content_type_builder.end()
    }
}

impl HttpContentType for BytesMut {
    #[inline]
    fn describe<B>(content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpContentTypeBuilder,
    {
        <Bytes as HttpContentType>::describe(content_type_builder)
    }
}

impl HttpContentType for Bytes {
    #[inline]
    fn describe<B>(mut content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpContentTypeBuilder,
    {
        content_type_builder.collect_content_type(
            "application/octet-stream",
            None,
            false,
            <Self as Schema>::describe,
        )?;
        content_type_builder.end()
    }
}

impl<T, U> HttpContentType for Chain<T, U>
where
    T: Buf + Unpin + Send + 'static,
    U: Buf + Unpin + Send + 'static,
{
    #[inline]
    fn describe<B>(mut content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpContentTypeBuilder,
    {
        content_type_builder.collect_content_type(
            "application/octet-stream",
            None,
            false,
            <Self as Schema>::describe,
        )?;
        content_type_builder.end()
    }
}

impl HttpContentType for &'static [u8] {
    #[inline]
    fn describe<B>(content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpContentTypeBuilder,
    {
        <Cow<'static, [u8]> as HttpContentType>::describe(content_type_builder)
    }
}

impl<const N: usize> HttpContentType for &'static [u8; N] {
    #[inline]
    fn describe<B>(content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpContentTypeBuilder,
    {
        <Cow<'static, [u8]> as HttpContentType>::describe(content_type_builder)
    }
}

impl<const N: usize> HttpContentType for [u8; N] {
    #[inline]
    fn describe<B>(content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpContentTypeBuilder,
    {
        <Cow<'static, [u8]> as HttpContentType>::describe(content_type_builder)
    }
}

impl HttpContentType for Vec<u8> {
    #[inline]
    fn describe<B>(content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpContentTypeBuilder,
    {
        <Cow<'static, [u8]> as HttpContentType>::describe(content_type_builder)
    }
}

impl HttpContentType for Box<[u8]> {
    #[inline]
    fn describe<B>(content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpContentTypeBuilder,
    {
        <Cow<'static, [u8]> as HttpContentType>::describe(content_type_builder)
    }
}

impl HttpContentType for Cow<'static, [u8]> {
    #[inline]
    fn describe<B>(mut content_type_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpContentTypeBuilder,
    {
        content_type_builder.collect_content_type(
            "application/octet-stream",
            None,
            false,
            <Cow<'static, [u8]> as Schema>::describe,
        )?;
        content_type_builder.end()
    }
}
