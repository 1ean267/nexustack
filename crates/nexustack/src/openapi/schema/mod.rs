/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

pub(crate) mod builder;
mod either;
pub(crate) mod example;
pub(crate) mod generator;
mod impls;
pub(crate) mod impossible;
pub(crate) mod nop;
pub(crate) mod optional;
pub(crate) mod post_process;

use builder::SchemaBuilder;
use serde::Serialize;

/// Trait for describing the `OpenAPI` schema of a Rust type.
///
/// This trait is implemented for many standard library types and can be derived for custom types.
/// It provides a way to generate OpenAPI-compatible schema definitions and example values for types,
/// which can be used for documentation, validation, and code generation.
///
/// # Associated Types
///
/// - `Example` - The type of a single example value for this schema. Must implement [`Serialize`] and `'static`.
/// - `Examples` - An iterator over example values of type [`Self::Example`].
///
/// # Required Method
///
/// - `describe` - Given a [`SchemaBuilder`], produces a schema description and example values for the type.
///
/// # Usage
///
/// You can implement `Schema` for your own types, or use the provided implementations for standard types.
/// The trait is designed to be flexible and extensible, supporting complex types such as tuples, enums,
/// collections, and more.
///
/// ## Example
///
/// ```rust
/// use nexustack::openapi::{FieldMod, Schema, SchemaBuilder, StructSchemaBuilder};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct MyStruct {
///     id: u32,
///     name: String,
/// }
///
/// impl Schema for MyStruct {
///     type Example = Self;
///     type Examples = std::iter::Once<Self>;
///
///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
///     where
///         B: SchemaBuilder<Self::Examples>,
///     {
///         let mut builder = schema_builder.describe_struct(
///             None,
///             2,
///             Some("A custom struct example"),
///             || Ok(std::iter::once(MyStruct { id: 1, name: "example".to_string() })),
///             false,
///         )?;
///         builder.collect_field("id", FieldMod::ReadWrite, None, false, <u32 as Schema>::describe)?;
///         builder.collect_field("name", FieldMod::ReadWrite, None, false, <String as Schema>::describe)?;
///         builder.end()
///     }
/// }
/// ```
///
/// # Implementing for Custom Types
///
/// When implementing `Schema` for your own types, you should:
/// - Choose appropriate example values for documentation and testing.
/// - Use the provided schema builder methods to describe the structure of your type.
/// - Ensure that your `Example` type is serializable and `'static`.
///
/// # See Also
///
/// - [`SchemaBuilder`]: Used to construct schema definitions.
/// - [`Serialize`]: Required for example values.
///
pub trait Schema {
    /// The type of a single example value for this schema.
    type Example: Serialize + 'static;

    /// An iterator over example values for this schema.
    type Examples: Iterator<Item = Self::Example>;

    /// Describe the schema for this type using the provided schema builder.
    ///
    /// # Paramaters
    ///
    /// - `schema_builder` - A builder that constructs the schema and collects example values.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>;
}
