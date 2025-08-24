/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

//! Generic `OpenAPI` schema builder framework.
//!
//! This module provides a set of traits and types for describing Rust data structures
//! as `OpenAPI` schemas. The main entry point is [`SchemaBuilder`], which exposes methods
//! for describing primitive types, compound types, and complex structures such as
//! structs, enums, tuples, and maps. The framework is designed to be flexible and
//! extensible, supporting custom schema generation and advanced features like field
//! modifiers, combinators, and tagging strategies.
//!
//! # Overview
//!
//! - **A type implementing [`SchemaBuilder`] is a schema generator** that can describe
//!   any Rust type as an `OpenAPI` schema.
//! - **Traits like [`StructSchemaBuilder`], [`EnumSchemaBuilder`], [`TupleSchemaBuilder`]**
//!   represent the building blocks for describing compound types.
//!
//! ## Provided `Schema` Implementations for Standard Library Types
//!
//! The framework includes ready-to-use `Schema` implementations for many Rust standard library types:
//!
//! - **Primitive types:** `bool`, `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, `u128`, `f32`, `f64`, `char`, `str`, `String`
//! - **Option and Result:** `Option<T>`, `Result<T, E>`
//! - **Tuples and arrays:** Tuples up to length 16, arrays `[T; N]` for N = 0..32, slices `[T]`
//! - **Collections:** `Vec<T>`, `VecDeque<T>`, `LinkedList<T>`, `BinaryHeap<T>`, `BTreeSet<T>`, `HashSet<T>`, `BTreeMap<K, V>`, `HashMap<K, V>`
//! - **Sync primitives:** `Atomic*` types, `Mutex<T>`, `RwLock<T>`, `Cell<T>`, `RefCell<T>`
//! - **Pointer types:** `Box<T>`, `Rc<T>`, `Arc<T>`, `Weak<T>`, references, `Cow<'a, T>`
//! - **FFI types:** `CString`, `CStr`, `OsString`, `OsStr`
//! - **Path types:** `Path`, `PathBuf`
//! - **Numbers:** `NonZero*`, `Wrapping<T>`, `Saturating<T>`
//! - **Time types:** `Duration`, `SystemTime`
//! - **Net types:** `IpAddr`, `Ipv4Addr`, `Ipv6Addr`, `SocketAddr`, `SocketAddrV4`, `SocketAddrV6`
//! - **Ranges and bounds:** `RangeFrom<T>`, `RangeTo<T>`, `RangeInclusive<T>`, `Bound<T>`
//!
//! These implementations ensure that most common Rust types can be described as `OpenAPI` schemas out of the box.
//!
//! # Usage
//!
//! Implementations of these traits are typically used by derive macros or manual
//! implementations to generate `OpenAPI` schemas for Rust types. The builder pattern
//! allows for incremental construction of schemas, supporting features such as
//! optional fields, default values, deprecation, and documentation.
//!
//! # Implementing the `Schema` Trait
//!
//! The [`Schema`] trait can be implemented manually for custom types, or automatically
//! via the provided `#[api_schema]` attribute macro. Manual implementation allows full
//! control over schema generation, while the attribute macro generates an implementation
//! that closely matches the type's [`serde::Serialize`] and [`serde::Deserialize`] behavior.
//!
//! ## Manual Implementation
//!
//! To manually implement [`Schema`], define the associated types and the `describe` method:
//!
//! # Example
//! ```rust
//! use nexustack::openapi::SchemaBuilder;
//! use nexustack::openapi::Schema;
//! use nexustack::openapi::StructSchemaBuilder;
//! use nexustack::openapi::FieldMod;
//! use nexustack::openapi::IntoSchemaBuilder;
//! use nexustack::openapi::SchemaId;
//! use nexustack::callsite;
//!
//! #[derive(serde::Serialize)]
//! struct MyType {
//!     a: u16,
//!     b: u64,
//! }
//!
//! impl Schema for MyType {
//!     type Example = MyType;
//!     type Examples = <[Self::Example; 1] as IntoIterator>::IntoIter;
//!
//!     #[inline]
//!     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
//!     where
//!         B: SchemaBuilder<Self::Examples>,
//!     {
//!         let mut struct_schema_builder = schema_builder.describe_struct(
//!             Some(SchemaId::new("MyType", callsite!())),
//!             2usize,
//!             Some("My custom struct description"),
//!             || Ok([
//!                 MyType {
//!                     a: 1u16,
//!                     b: 3u64,
//!                 }
//!             ]),
//!             false
//!         )?;
//!         
//!         struct_schema_builder.collect_field(
//!             "a",
//!             FieldMod::ReadWrite,
//!             Some("Field a"),
//!             false,
//!             <u16 as Schema>::describe
//!         )?;
//!         
//!         struct_schema_builder.collect_field_optional(
//!             "b",
//!             FieldMod::ReadWrite,
//!             Some(0u64),
//!             Some("Field b"),
//!             false,
//!             <u64 as Schema>::describe
//!         )?;
//!
//!         struct_schema_builder.end()
//!     }
//! }
//! ```
//!
//! ## Attribute Macro Implementation
//!
//! The `#[api_schema]` attribute macro can be applied to structs and enums. When used,
//! it generates a [`Schema`] implementation that matches the type's [`serde::Serialize`]
//! and [`serde::Deserialize`] representation. If the attribute is present, the type
//! will also automatically derive [`serde::Serialize`] and/or [`serde::Deserialize`]
//! (if not already implemented).
//!
//! ### Example: Struct
//!
//! ```rust
//! use nexustack::openapi::api_schema;
//!
//! /// Custom struct definition
//! #[api_schema]
//! #[derive(Debug)]
//! pub struct Point {
//!     /// The x field
//!     x: i32,
//!     /// The y field
//!     y: i32,
//! }
//! ```
//!
//! The generated schema will describe an object with two integer properties, matching
//! the serde serialization format.
//!
//! ### Example: Enum
//!
//! ```rust
//! use nexustack::openapi::api_schema;
//!
//! /// Custom enum definition
//! #[api_schema(untagged)]
//! pub enum Message {
//!     /// String option
//!     Text(
//!         /// Value of the string option
//!         String
//!     ),
//!     /// Number option
//!     Number(
//!         /// Value of the number option
//!         i32
//!     ),
//! }
//! ```
//!
//! The schema will use an "untagged" representation, matching serde's `#[serde(untagged)]`.
//!
//! ### Example: Customization
//!
//! The attribute supports options similar to serde, such as `rename`, `skip`, `default`,
//! and tagging strategies:
//!
//! ```rust
//! use nexustack::openapi::api_schema;
//!
//! /// MyStruct description
//! #[api_schema(rename = "MyStruct")]
//! pub struct S {
//!     /// Hidden field
//!     #[api_property(skip)]
//!     hidden: String,
//!
//!     /// Optional field
//!     #[api_property(default, skip_serializing_if = "Option::is_none")]
//!     value: Option<i32>,
//! }
//! ```
//!
//! # API Parity with Serde
//!
//! The schema builder API is designed to closely match the API of [`serde::Serializer`]
//! and [`serde::Deserialize`]. The same tagging strategies (externally tagged, internally
//! tagged, adjacently tagged, untagged) are supported for enums. Field attributes such
//! as `rename`, `skip`, `default`, and `skip_serializing_if` are recognized and reflected
//! in the generated schema. This ensures that the `OpenAPI` schema accurately represents
//! the wire format of the type as serialized/deserialized by serde.
//!
//!
//! # Traits
//!
//! - [`SchemaBuilder`]: Entry point for describing schemas.
//! - [`StructSchemaBuilder`]: Describes struct fields.
//! - [`EnumSchemaBuilder`]: Describes enum variants.
//! - [`TupleSchemaBuilder`]: Describes tuple elements.
//! - [`MapSchemaBuilder`]: Describes map keys and values.
//! - [`CombinatorSchemaBuilder`]: Describes combinator schemas (oneOf, anyOf, allOf).
//!
//! # Field Modifiers
//!
//! The [`FieldMod`] enum allows marking fields as read-only, write-only, or read-write,
//! which is useful for API documentation and code generation.
//!
//! # Tagging Strategies
//!
//! The [`VariantTag`] enum supports different enum tagging strategies, such as
//! externally tagged, internally tagged, adjacently tagged, or untagged.
//!
//! # Extensibility
//!
//! The framework is designed to be extensible, allowing custom schema builders
//! and integration with other `OpenAPI` tooling.
//!
//! # See Also
//!
//! - [`serde::Serializer`]: For serialization of Rust types.
//! - [`openapi::schema::Schema`]: The actual schema representation.
//!

// Used by generated code and doc tests. Not public API.
#[doc(hidden)]
#[path = "private/mod.rs"]
pub mod __private;

#[cfg(feature = "derive")]
pub use nexustack_macros::api_schema;

mod either;
mod error;
mod example;
mod impls;
mod impossible;
mod nop;
mod post_process;
mod schema;
mod schema_builder;

pub mod json;

pub use error::Error;
pub use example::SchemaExamples;
pub use impossible::Impossible;
pub use nop::Nop;
pub use schema::Schema;
pub use schema_builder::{
    Combinator, CombinatorSchemaBuilder, EnumSchemaBuilder, FieldMod, IntoSchemaBuilder,
    MapSchemaBuilder, SchemaBuilder, SchemaId, StructSchemaBuilder, StructVariantSchemaBuilder,
    TupleSchemaBuilder, TupleStructSchemaBuilder, TupleVariantSchemaBuilder, VariantTag,
};
