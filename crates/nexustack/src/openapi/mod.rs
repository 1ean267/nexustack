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
