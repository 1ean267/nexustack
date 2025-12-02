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
#[cfg(feature = "http")]
mod http;

mod error;
mod schema;
mod spec;
mod version;

#[cfg(feature = "derive")]
pub use nexustack_macros::api_schema;

pub use error::Error;
pub use version::SpecificationVersion;

#[path = ""]
pub mod generator {
    pub use crate::openapi::schema::generator::{
        SchemaCollection, build_schema, build_schema_with_collection,
    };
}

// TODO: Replace with pub mod http; when stable (need to change macros)
#[cfg(feature = "http")]
pub use http::{
    HttpDocument, HttpDocumentBuilder, HttpServer, HttpServerVariable, Tag,
    content_type::{HttpContentType, HttpContentTypeBuilder},
    operation::{
        HttpOperation, HttpOperationBuilder, HttpOperationId, HttpSecurityRequirementBuilder,
    },
    response::{HttpResponse, HttpResponseBuilder},
};

pub use schema::{
    Schema,
    builder::{
        Combinator, CombinatorSchemaBuilder, EnumSchemaBuilder, FieldMod, IntoSchemaBuilder,
        MapSchemaBuilder, SchemaBuilder, SchemaId, StructSchemaBuilder, StructVariantSchemaBuilder,
        TupleSchemaBuilder, TupleStructSchemaBuilder, TupleVariantSchemaBuilder, VariantTag,
    },
    example::SchemaExamples,
    impossible::Impossible,
    nop::Nop,
    optional::Optional,
};
