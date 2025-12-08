/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use std::convert::Infallible;

///
/// A trait for converting a type into an HTTP response with additional context.
///
/// This trait extends the functionality of `axum::IntoResponse` by allowing the inclusion
/// of additional context when generating the response.
///
/// # Associated Types
///
/// - `Context`: Represents the additional context required to generate the response.
///   This context must implement `FromRequestParts<S>`.
///
/// # Paramaters
///
/// - `context`: The additional context of type `Self::Context` required to generate the response.
///
/// # Example
///
/// ```rust
/// use axum::{response::Response, body::Body};
/// use crate::http::response::IntoResponseWithContext;
///
/// struct MyContext;
///
/// impl<S> IntoResponseWithContext<S> for String {
///     type Context = MyContext;
///
///     fn into_response(self, _context: Self::Context) -> Response<Body> {
///         Response::new(Body::from(self))
///     }
/// }
///
/// let context = MyContext;
/// let response = "Hello, world!".to_string().into_response(context);
/// ```
///
pub trait IntoResponseWithContext<S>: Sized {
    /// The associated context type required to generate the response.
    type Context: axum::extract::FromRequestParts<S> + Send;

    /// Converts the type into an HTTP response using the provided context.
    ///
    /// # Paramaters
    ///
    /// - `context` - The additional context of type `Self::Context` required to generate the response.
    fn into_response(self, context: Self::Context) -> axum::response::Response;
}

// impl<S: Send + Sync, T: IntoResponse> IntoResponseWithContext<S> for T {
//     type Context = ();

//     fn into_response(self, _context: Self::Context) -> Response<Body> {
//         self.into_response()
//     }
// }

impl<R, E, S> IntoResponseWithContext<S> for Result<R, E>
where
    R: IntoResponseWithContext<S>,
    E: IntoResponseWithContext<S>,
    S: Send + Sync,
{
    type Context = (R::Context, E::Context);

    fn into_response(self, context: Self::Context) -> axum::response::Response {
        match self {
            Ok(value) => value.into_response(context.0),
            Err(err) => err.into_response(context.1),
        }
    }
}

// TODO: Provide all other impls as is done for IntoResponse

impl<S> IntoResponseWithContext<S> for Infallible
where
    S: Send + Sync,
{
    type Context = ();

    fn into_response(self, _context: Self::Context) -> axum::response::Response {
        match self {}
    }
}
