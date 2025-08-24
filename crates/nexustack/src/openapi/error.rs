/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/// Trait used by [`Schema`] implementations to generically construct errors belonging to the [`SchemaBuilder`] against which they are currently running.
///
/// # Example
///
/// ```rust
/// #[derive(Debug, PartialEq)]
/// struct Error(String);
///
/// impl nexustack::openapi::Error for Error {
///     fn custom<T>(msg: T) -> Self
///         where
///             T: std::fmt::Display {
///         Self(msg.to_string())
///     }
/// }
///
/// impl std::fmt::Display for Error {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
///         f.write_str(&self.0)
///     }
/// }
///
/// impl std::error::Error for Error { }
/// ```
///
/// [`Schema`]: crate::openapi::schema::Schema
/// [`SchemaBuilder`]: crate::openapi::schema_builder::SchemaBuilder
pub trait Error: Sized + std::error::Error {
    /// Used when a [`Schema`] implementation encounters any error while describing a type.
    ///
    /// The message should not be capitalized and should not end with a period.
    ///
    /// # Arguments
    /// * `msg` - The error message to be included in the custom error. Must implement [`std::fmt::Display`].
    ///
    /// [`Schema`]: crate::openapi::schema::Schema
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display;
}
