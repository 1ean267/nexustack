/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::openapi::{HttpContentType, HttpResponse, HttpResponseBuilder};
use bytes::{Buf, Bytes, BytesMut, buf::Chain};
use std::{borrow::Cow, convert::Infallible};

impl HttpResponse for () {
    #[inline]
    fn describe<B>(mut response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::HttpResponseBuilder,
    {
        response_builder.describe_empty_response(200, None, false)?;
        response_builder.end()
    }
}

impl HttpResponse for Infallible {
    #[inline]
    fn describe<B>(response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::HttpResponseBuilder,
    {
        response_builder.end()
    }
}

struct WrappedHttpResponseBuilder<B> {
    inner: B,
}

impl<B> HttpResponseBuilder for &mut WrappedHttpResponseBuilder<B>
where
    B: HttpResponseBuilder,
{
    type Ok = ();
    type Error = B::Error;

    type ContentTypeBuilder<'a>
        = B::ContentTypeBuilder<'a>
    where
        Self: 'a;

    fn describe_response<'a>(
        &'a mut self,
        status_code: u16,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::ContentTypeBuilder<'a>, Self::Error> {
        self.inner
            .describe_response(status_code, description, deprecated)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<T, E> HttpResponse for Result<T, E>
where
    T: HttpResponse,
    E: HttpResponse,
{
    #[inline]
    fn describe<B>(response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::HttpResponseBuilder,
    {
        let mut wrapped = WrappedHttpResponseBuilder {
            inner: response_builder,
        };

        <T as HttpResponse>::describe(&mut wrapped)?;
        <E as HttpResponse>::describe(&mut wrapped)?;

        wrapped.inner.end()
    }
}

impl HttpResponse for &'static str {
    #[inline]
    fn describe<B>(response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpResponseBuilder,
    {
        <Cow<'static, str> as HttpResponse>::describe(response_builder)
    }
}

impl HttpResponse for String {
    #[inline]
    fn describe<B>(response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpResponseBuilder,
    {
        <Cow<'static, str> as HttpResponse>::describe(response_builder)
    }
}

impl HttpResponse for Box<str> {
    #[inline]
    fn describe<B>(response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpResponseBuilder,
    {
        <Cow<'static, str> as HttpResponse>::describe(response_builder)
    }
}

impl HttpResponse for Cow<'static, str> {
    #[inline]
    fn describe<B>(mut response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpResponseBuilder,
    {
        response_builder.collect_response(
            200,
            None,
            false,
            <Cow<'static, str> as HttpContentType>::describe,
        )?;
        response_builder.end()
    }
}

impl HttpResponse for BytesMut {
    #[inline]
    fn describe<B>(response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpResponseBuilder,
    {
        <Bytes as HttpResponse>::describe(response_builder)
    }
}

impl HttpResponse for Bytes {
    #[inline]
    fn describe<B>(mut response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpResponseBuilder,
    {
        response_builder.collect_response(200, None, false, <Self as HttpContentType>::describe)?;
        response_builder.end()
    }
}

impl<T, U> HttpResponse for Chain<T, U>
where
    T: Buf + Unpin + Send + 'static,
    U: Buf + Unpin + Send + 'static,
{
    #[inline]
    fn describe<B>(mut response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpResponseBuilder,
    {
        response_builder.collect_response(200, None, false, <Self as HttpContentType>::describe)?;
        response_builder.end()
    }
}

impl HttpResponse for &'static [u8] {
    #[inline]
    fn describe<B>(response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpResponseBuilder,
    {
        <Cow<'static, [u8]> as HttpResponse>::describe(response_builder)
    }
}

impl<const N: usize> HttpResponse for &'static [u8; N] {
    #[inline]
    fn describe<B>(response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpResponseBuilder,
    {
        <Cow<'static, [u8]> as HttpResponse>::describe(response_builder)
    }
}

impl<const N: usize> HttpResponse for [u8; N] {
    #[inline]
    fn describe<B>(response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpResponseBuilder,
    {
        <Cow<'static, [u8]> as HttpResponse>::describe(response_builder)
    }
}

impl HttpResponse for Vec<u8> {
    #[inline]
    fn describe<B>(response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpResponseBuilder,
    {
        <Cow<'static, [u8]> as HttpResponse>::describe(response_builder)
    }
}

impl HttpResponse for Box<[u8]> {
    #[inline]
    fn describe<B>(response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpResponseBuilder,
    {
        <Cow<'static, [u8]> as HttpResponse>::describe(response_builder)
    }
}

impl HttpResponse for Cow<'static, [u8]> {
    #[inline]
    fn describe<B>(mut response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpResponseBuilder,
    {
        response_builder.collect_response(
            200,
            None,
            false,
            <Cow<'static, [u8]> as HttpContentType>::describe,
        )?;
        response_builder.end()
    }
}

impl<R> HttpResponse for (R,)
where
    R: HttpResponse,
{
    #[inline]
    fn describe<B>(response_builder: B) -> Result<B::Ok, B::Error>
    where
        B: HttpResponseBuilder,
    {
        <R as HttpResponse>::describe(response_builder)
    }
}

// TODO: Implement for all axum rejections: https://docs.rs/axum/latest/axum/response/trait.IntoResponse.html#implementors
