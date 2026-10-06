/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

// Used by generated code and doc tests. Not public API.
#[doc(hidden)]
#[path = "private/mod.rs"]
pub mod __private;

mod error;
mod schema;
mod spec;
mod string_serializer;
mod version;

#[cfg(feature = "derive")]
pub use nexustack_macros::api_schema;

pub use error::Error;
pub use version::SpecificationVersion;

#[cfg(feature = "http")]
pub mod http;

pub use schema::{
    Schema,
    builder::{
        Combinator, CombinatorSchemaBuilder, EnumSchemaBuilder, FieldMod, IntoSchemaBuilder,
        MapSchemaBuilder, SchemaBuilder, SchemaId, StructSchemaBuilder, StructVariantSchemaBuilder,
        TupleSchemaBuilder, TupleStructSchemaBuilder, TupleVariantSchemaBuilder, VariantTag,
    },
    error::SchemaGenerationError,
    example::SchemaExamples,
    impossible::Impossible,
    nop::Nop,
    optional::Optional,
};
