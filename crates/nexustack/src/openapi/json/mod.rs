/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

//! # `OpenAPI` JSON Schema Generation
//!
//! This module provides comprehensive facilities for generating [OpenAPI](https://spec.openapis.org/) JSON Schema objects from Rust types.
//!
//! ## Overview
//!
//! The core purpose of this module is to translate Rust type information, as described by the [`Schema`] trait, into OpenAPI-compatible JSON Schema representations. It supports both `OpenAPI` 3.0 and 3.1 specifications, handling nuances and differences between them.
//!
//! ## Features
//!
//! - **Schema Generation**: Functions like [`build_schema`] and [`build_schema_with_collection`] allow you to generate schemas for any type implementing [`Schema`].
//! - **Schema Collection**: Support for schema deduplication and referencing via [`SchemaCollection`], enabling reuse and reference of complex types.
//! - **Flexible Builders**: Implements a rich set of builder patterns for structs, tuples, enums, maps, combinators, and more, allowing fine-grained control over schema generation.
//! - **Field Modifiers**: Expressive support for field modifiers such as `read_only`, `write_only`, `deprecated`, and default values.
//! - **Nullability & Examples**: Handles nullable types and example values, adapting to `OpenAPI` version differences.
//! - **Pattern Properties**: Advanced support for map key patterns, including integer and string keys, with regular expression generation.
//! - **Post-Processing**: Allows post-processing and transformation of schemas for custom requirements.
//!
//! ## Usage
//!
//! Typical usage involves calling [`build_schema`] or [`build_schema_with_collection`] with a Rust type and desired specification. The module is designed to be extensible and composable, supporting advanced `OpenAPI` schema features.

use crate::openapi::{
    impossible::Impossible,
    json::schema_collection::SchemaCollectionResolutionError,
    nop::Nop,
    post_process::{PostProcessSchemaBuilder, Transform},
    schema::Schema,
    schema_builder::{
        Combinator, CombinatorSchemaBuilder, EnumSchemaBuilder, FieldMod, IntoSchemaBuilder,
        MapSchemaBuilder, SchemaBuilder, SchemaId, StructSchemaBuilder, StructVariantSchemaBuilder,
        TupleSchemaBuilder, TupleStructSchemaBuilder, TupleVariantSchemaBuilder, VariantTag,
    },
};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{BTreeSet, HashMap},
    fmt::Write,
    rc::Rc,
};

mod error;
mod schema_collection;
mod specification;

use error::Error;
pub use schema_collection::SchemaCollection;
pub use specification::*;

/// Build an OpenAPI-compatible JSON Schema for a Rust type implementing [`Schema`].
///
/// # Arguments
/// * `specification` - The `OpenAPI` specification version to target (e.g., [`Specification::OpenAPI3_0`] or [`Specification::OpenAPI3_1`]).
///   This affects schema features such as nullability, examples, and type representation.
///
/// # Returns
/// * `Result<SchemaOrReferenceObject, Error>` - On success, returns the generated schema object or reference. On failure, returns an error describing the problem.
///
///  # Errors
///
/// Returns an error if schema construction fails, for example due to:
/// - Invalid type information or unsupported types.
/// - Serialization errors when generating example values.
/// - Builder-specific errors encountered during schema description.
/// - Multiple conflicting schema definitions for the same name
///
/// # Example
/// ```rust
/// /// use nexustack::openapi::json::SchemaCollection;
/// use nexustack::openapi::json::Specification;
/// use nexustack::openapi::api_schema;
/// use nexustack::openapi::json::build_schema;
///
/// /// Custom struct definition
/// #[api_schema]
/// struct MyType {
///     /// Field i
///     i: i32,
///     /// Field f
///     f: f32
/// }
///
/// let schema = build_schema::<MyType>(Specification::OpenAPI3_1);
/// ```
pub fn build_schema<T: Schema>(
    specification: Specification,
) -> Result<SchemaOrReferenceObject, Error> {
    let schema_builder = JsonSchemaBuilder::new(specification, None);
    T::describe(schema_builder)
}

/// Build an OpenAPI-compatible JSON Schema for a Rust type, using a shared [`SchemaCollection`] for deduplication and referencing.
///
/// # Arguments
/// * `specification` - The `OpenAPI` specification version to target (e.g., [`Specification::OpenAPI3_0`] or [`Specification::OpenAPI3_1`]).
///   This affects schema features such as nullability, examples, and type representation.
/// * `schema_collection` - A reference-counted, mutable [`SchemaCollection`] used to store and deduplicate schemas.
///   This enables referencing complex types and avoids redundant schema definitions.
///
/// # Returns
/// * `Result<SchemaOrReferenceObject, Error>` - On success, returns the generated schema object or reference. On failure, returns an error describing the problem.
///
/// # Errors
///
/// Returns an error if schema construction fails, for example due to:
/// - Invalid type information or unsupported types.
/// - Serialization errors when generating example values.
/// - Builder-specific errors encountered during schema description.
/// - Multiple conflicting schema definitions for the same name
///
/// # Example
///
/// ```rust
/// use nexustack::openapi::json::SchemaCollection;
/// use nexustack::openapi::json::Specification;
/// use nexustack::openapi::api_schema;
/// use nexustack::openapi::json::build_schema_with_collection;
/// use std::cell::RefCell;
/// use std::rc::Rc;
///
/// /// Custom struct definition
/// #[api_schema]
/// struct MyType {
///     /// Field i
///     i: i32,
///     /// Field f
///     f: f32
/// }
///
/// let collection = Rc::new(RefCell::new(SchemaCollection::default()));
/// let schema = build_schema_with_collection::<MyType>(Specification::OpenAPI3_1, collection.clone());
/// ```
///
pub fn build_schema_with_collection<T: Schema>(
    specification: Specification,
    schema_collection: Rc<RefCell<SchemaCollection>>,
) -> Result<SchemaOrReferenceObject, Error> {
    let schema_builder = JsonSchemaBuilder::new(specification, Some(schema_collection));
    T::describe(schema_builder)
}

macro_rules! set {
    () => {
        std::collections::BTreeSet::new()
    };
    ($($x:expr),+ $(,)?) => ({
        std::collections::BTreeSet::from([$($x,)+])
    });
}

macro_rules! map {
    () => {
        std::collections::HashMap::new()
    };
    ($($k:expr => $v:expr),+ $(,)?) => {
        std::collections::HashMap::from([
            $(($k,$v),)+
        ])
    };
}

macro_rules! one_of {
    () => ({
        let mut one_of = SchemaObject::default();
        one_of.one_of = Some(vec![]);
        one_of
    });
    ($($x:expr),+ $(,)?) => ({
        let mut one_of = SchemaObject::default();
        one_of.one_of = Some(vec![$($x.into()),+]);
        one_of
    });
}

macro_rules! all_of {
    () => ({
        let mut all_of = SchemaObject::default();
        all_of.all_of = Some(vec![]);
        all_of
    });
    ($($x:expr),+ $(,)?) => ({
        let mut all_of = SchemaObject::default();
        all_of.all_of = Some(vec![$($x.into()),+]);
        all_of
    });
}

macro_rules! schema {
    () => {
        SchemaObject {
            ..Default::default()
        }
    };
    ($($k:ident:$v:expr),+ $(,)?) => {
        SchemaObject {
            $($k:Some($v),)+
            ..Default::default()
        }
    };
}

fn null_schema(specification: Specification) -> SchemaObject {
    match specification {
        Specification::OpenAPI3_0 => {
            schema! {
                r#type: "string".into(),
                nullable: true,
                r#enum: vec![JsonValue::Null],
                example: JsonValue::Null,
            }
        }
        Specification::OpenAPI3_1 => {
            schema! {
                r#type: "null".into(),
                examples: specification::Examples::Vec(vec![JsonValue::Null]),
            }
        }
    }
}

//
// Struct
//

struct StructFieldTransform<'s> {
    schema_builder: &'s mut StructJsonSchemaBuilder,
    key: &'static str,
    modifier: FieldMod,
    description: Option<&'static str>,
    deprecated: bool,
    is_optional: bool,
    default: Option<JsonValue>,
}

impl<'s> StructFieldTransform<'s> {
    const fn new(
        schema_builder: &'s mut StructJsonSchemaBuilder,
        key: &'static str,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
        is_optional: bool,
        default: Option<JsonValue>,
    ) -> Self {
        Self {
            schema_builder,
            key,
            modifier,
            description,
            deprecated,
            is_optional,
            default,
        }
    }
}

impl Transform<SchemaOrReferenceObject> for StructFieldTransform<'_> {
    type Output = ();
    type Error = Error;

    fn transform(self, i: SchemaOrReferenceObject) -> Result<Self::Output, Self::Error> {
        let mut schema = i;

        if let SchemaOrReferenceObject::Schema(schema_object) = &mut schema {
            // TODO: This overrides the schema definition
            if let Some(description) = self.description {
                schema_object.description = Some(description.into());
            }

            // TODO: This overrides the schema definition
            if self.deprecated {
                schema_object.deprecated = Some(true);
            }

            // TODO: This overrides the schema definition
            if let Some(default) = self.default {
                schema_object.default = Some(default);
            }

            match self.modifier {
                FieldMod::Read => schema_object.read_only = Some(true),
                FieldMod::Write => schema_object.write_only = Some(true),
                FieldMod::ReadWrite => {}
            }
        } else {
            let mut additional_schema = self.default.map(|default| schema! { default: default });

            match self.modifier {
                FieldMod::Read => {
                    if let Some(additional_schema) = &mut additional_schema {
                        additional_schema.read_only = Some(true);
                    } else {
                        additional_schema = Some(schema! { read_only: true });
                    }
                }
                FieldMod::Write => {
                    if let Some(additional_schema) = &mut additional_schema {
                        additional_schema.write_only = Some(true);
                    } else {
                        additional_schema = Some(schema! { write_only: true });
                    }
                }
                FieldMod::ReadWrite => {}
            }

            if let Some(additional_schema) = additional_schema {
                schema = all_of!(schema, additional_schema).into();
            }
        }

        let properties = self
            .schema_builder
            .result_schema
            .properties
            .get_or_insert_with(|| HashMap::with_capacity(self.schema_builder.len));

        if !self.is_optional {
            self.schema_builder
                .result_schema
                .required
                .get_or_insert_with(BTreeSet::new)
                .insert(self.key.into());
        }

        let previous = properties.insert(self.key.into(), schema.into());

        if previous.is_some() {
            // TODO: Custom error type and more information
            return Err(Error::custom(format!(
                "duplicate entry for field {key}",
                key = self.key
            )));
        }

        Ok(())
    }
}

struct StructJsonSchemaBuilder {
    specification: Specification,
    schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
    id: Option<SchemaId>,
    result_schema: SchemaObject,
    len: usize,
}

impl StructJsonSchemaBuilder {
    #[allow(clippy::too_many_arguments)]
    fn new(
        specification: Specification,
        schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: Option<Vec<JsonValue>>,
        deprecated: bool,
        nullable: bool,
        len: usize,
    ) -> Self {
        let mut result = SchemaObject {
            description: description.map(Into::into),
            ..SchemaObject::default()
        };

        if deprecated {
            result.deprecated = Some(true);
        }

        match specification {
            Specification::OpenAPI3_0 => {
                result.example = examples.and_then(|examples| examples.into_iter().next());
                result.r#type = Some("object".into());

                if nullable {
                    result.nullable = Some(true);
                }
            }
            Specification::OpenAPI3_1 => {
                result.examples = examples.map(specification::Examples::Vec);

                if nullable {
                    result.r#type = Some(vec!["object".into(), "null".into()].into());
                } else {
                    result.r#type = Some("object".into());
                }
            }
        }

        Self {
            specification,
            schema_collection,
            id,
            result_schema: result,
            len,
        }
    }
}

impl StructSchemaBuilder for StructJsonSchemaBuilder {
    type MapKey = Option<Cow<'static, str>>;
    type Ok = SchemaOrReferenceObject;
    type Error = Error;

    type FieldSchemaBuilder<'a>
        = PostProcessSchemaBuilder<StructFieldTransform<'a>, JsonSchemaBuilder>
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        let specification = self.specification;
        let schema_collection = self.schema_collection.clone();
        Ok(PostProcessSchemaBuilder::new(
            StructFieldTransform::new(self, key, modifier, description, deprecated, false, None),
            JsonSchemaBuilder::new(specification, schema_collection),
        ))
    }

    fn describe_field_optional<'a, F: Serialize>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        default: Option<F>,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        let specification = self.specification;
        let schema_collection = self.schema_collection.clone();
        let default = default
            .map(|default| serde_json::to_value(default))
            .transpose()
            .map_err(Error::custom)?;
        Ok(PostProcessSchemaBuilder::new(
            StructFieldTransform::new(self, key, modifier, description, deprecated, true, default),
            JsonSchemaBuilder::new(specification, schema_collection),
        ))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if let Some(schema_collection) = self.schema_collection {
            let mut schema_collection = schema_collection.borrow_mut();
            if let Some(schema_id) = self.id {
                let schema_ref = schema_collection.set(&schema_id, self.result_schema.into());
                return Ok(ReferenceObject {
                    r#ref: schema_ref,
                    summary: None,
                    description: None,
                }
                .into());
            }
        }

        Ok(self.result_schema.into())
    }
}

//
// Tuple
//

struct TupleElementTransform<'s> {
    schema_builder: &'s mut TupleJsonSchemaBuilder,
    description: Option<&'static str>,
    deprecated: bool,
}

impl<'s> TupleElementTransform<'s> {
    const fn new(
        schema_builder: &'s mut TupleJsonSchemaBuilder,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Self {
        Self {
            schema_builder,
            description,
            deprecated,
        }
    }
}

impl Transform<SchemaOrReferenceObject> for TupleElementTransform<'_> {
    type Output = ();
    type Error = Error;

    fn transform(self, i: SchemaOrReferenceObject) -> Result<Self::Output, Self::Error> {
        let mut schema = i;

        match &mut schema {
            SchemaOrReferenceObject::Schema(schema_object) => {
                // TODO: This overrides the schema definition
                if let Some(description) = self.description {
                    schema_object.description = Some(description.into());
                }

                // TODO: This overrides the schema definition
                if self.deprecated {
                    schema_object.deprecated = Some(true);
                }
            }
            SchemaOrReferenceObject::Reference(reference_object) => {
                if let Some(description) = self.description {
                    reference_object.description = Some(description.into());
                }

                // TODO: How can we handle this?
                // if self.deprecated {
                //     reference_object.deprecated = Some(true);
                // }
            }
        }

        self.schema_builder.subschemas.push(schema);

        Ok(())
    }
}

struct TupleJsonSchemaBuilder {
    specification: Specification,
    schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
    id: Option<SchemaId>,
    subschemas: Vec<SchemaOrReferenceObject>,
    result_schema: SchemaObject,
}

impl TupleJsonSchemaBuilder {
    #[allow(clippy::too_many_arguments)]
    fn new(
        specification: Specification,
        schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: Option<Vec<JsonValue>>,
        deprecated: bool,
        nullable: bool,
        len: usize,
    ) -> Self {
        let mut result = SchemaObject {
            description: description.map(Into::into),
            ..SchemaObject::default()
        };

        if deprecated {
            result.deprecated = Some(true);
        }

        match specification {
            Specification::OpenAPI3_0 => {
                result.example = examples.and_then(|examples| examples.into_iter().next());
                result.r#type = Some("array".into());

                if nullable {
                    result.nullable = Some(true);
                }
            }
            Specification::OpenAPI3_1 => {
                result.examples = examples.map(specification::Examples::Vec);

                if nullable {
                    result.r#type = Some(vec!["array".into(), "null".into()].into());
                } else {
                    result.r#type = Some("array".into());
                }
            }
        }

        Self {
            specification,
            schema_collection,
            id,
            subschemas: Vec::with_capacity(len),
            result_schema: result,
        }
    }
}

impl TupleSchemaBuilder for TupleJsonSchemaBuilder {
    type MapKey = Option<Cow<'static, str>>;
    type Ok = SchemaOrReferenceObject;
    type Error = Error;

    type ElementSchemaBuilder<'a>
        = PostProcessSchemaBuilder<TupleElementTransform<'a>, JsonSchemaBuilder>
    where
        Self: 'a;

    fn describe_element<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::ElementSchemaBuilder<'a>, Self::Error> {
        let specification = self.specification;
        let schema_collection = self.schema_collection.clone();
        Ok(PostProcessSchemaBuilder::new(
            TupleElementTransform::new(self, description, deprecated),
            JsonSchemaBuilder::new(specification, schema_collection),
        ))
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.result_schema.min_items = Some(self.subschemas.len().into());
        self.result_schema.max_items = Some(self.subschemas.len().into());

        match self.specification {
            Specification::OpenAPI3_0 => {
                // TODO: Combine items if possible
                self.result_schema.items = Some(
                    schema! {
                        one_of: self.subschemas
                                    .into_iter()
                                    .map(Into::into)
                                    .collect(),
                    }
                    .into(),
                );
            }
            Specification::OpenAPI3_1 => {
                self.result_schema.prefix_items =
                    Some(self.subschemas.into_iter().map(Into::into).collect());
            }
        }

        if let Some(schema_collection) = self.schema_collection {
            let mut schema_collection = schema_collection.borrow_mut();
            if let Some(schema_id) = self.id {
                let schema_ref = schema_collection.set(&schema_id, self.result_schema.into());
                return Ok(ReferenceObject {
                    r#ref: schema_ref,
                    summary: None,
                    description: None,
                }
                .into());
            }
        }

        Ok(self.result_schema.into())
    }
}

//
// Tuple struct
//

struct TupleStructJsonSchemaBuilder {
    inner: TupleJsonSchemaBuilder,
}

impl TupleStructJsonSchemaBuilder {
    #[allow(clippy::too_many_arguments)]
    fn new(
        specification: Specification,
        schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: Option<Vec<JsonValue>>,
        deprecated: bool,
        nullable: bool,
        len: usize,
    ) -> Self {
        Self {
            inner: TupleJsonSchemaBuilder::new(
                specification,
                schema_collection,
                id,
                description,
                examples,
                deprecated,
                nullable,
                len,
            ),
        }
    }
}

impl TupleStructSchemaBuilder for TupleStructJsonSchemaBuilder {
    type MapKey = Option<Cow<'static, str>>;
    type Ok = SchemaOrReferenceObject;
    type Error = Error;

    type FieldSchemaBuilder<'a>
        = PostProcessSchemaBuilder<TupleElementTransform<'a>, JsonSchemaBuilder>
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        self.inner.describe_element(description, deprecated)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

//
// Combinator
//

struct CombinatorSubschemaTransform<'s> {
    schema_builder: &'s mut CombinatorJsonSchemaBuilder,
    description: Option<&'static str>,
    deprecated: bool,
}

impl<'s> CombinatorSubschemaTransform<'s> {
    const fn new(
        schema_builder: &'s mut CombinatorJsonSchemaBuilder,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Self {
        Self {
            schema_builder,
            description,
            deprecated,
        }
    }
}

impl Transform<SchemaOrReferenceObject> for CombinatorSubschemaTransform<'_> {
    type Output = ();
    type Error = Error;

    fn transform(self, i: SchemaOrReferenceObject) -> Result<Self::Output, Self::Error> {
        let mut schema = i;

        match &mut schema {
            SchemaOrReferenceObject::Schema(schema_object) => {
                // TODO: This overrides the schema definition
                if let Some(description) = self.description {
                    schema_object.description = Some(description.into());
                }

                // TODO: This overrides the schema definition
                if self.deprecated {
                    schema_object.deprecated = Some(true);
                }
            }
            SchemaOrReferenceObject::Reference(reference_object) => {
                if let Some(description) = self.description {
                    reference_object.description = Some(description.into());
                }

                // TODO: How can we handle this?
                // if self.deprecated {
                //     reference_object.deprecated = Some(true);
                // }
            }
        }

        self.schema_builder.subschemas.push(schema);

        Ok(())
    }
}

struct CombinatorJsonSchemaBuilder {
    specification: Specification,
    schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
    description: Option<&'static str>,
    examples: Option<Vec<JsonValue>>,
    deprecated: bool,
    nullable: bool,
    combinator: Combinator,
    subschemas: Vec<SchemaOrReferenceObject>,
}

impl CombinatorJsonSchemaBuilder {
    #[allow(clippy::too_many_arguments)]
    fn new(
        specification: Specification,
        schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
        description: Option<&'static str>,
        examples: Option<Vec<JsonValue>>,
        deprecated: bool,
        nullable: bool,
        combinator: Combinator,
        len: usize,
    ) -> Self {
        let capacity = if nullable && combinator == Combinator::OneOf {
            len + 1
        } else {
            len
        };

        Self {
            specification,
            schema_collection,
            description,
            examples,
            deprecated,
            nullable,
            combinator,
            subschemas: Vec::with_capacity(capacity),
        }
    }
}

impl CombinatorSchemaBuilder for CombinatorJsonSchemaBuilder {
    type MapKey = Option<Cow<'static, str>>;
    type Ok = SchemaOrReferenceObject;
    type Error = Error;

    type SubSchemaBuilder<'a>
        = PostProcessSchemaBuilder<CombinatorSubschemaTransform<'a>, JsonSchemaBuilder>
    where
        Self: 'a;

    fn describe_subschema<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::SubSchemaBuilder<'a>, Self::Error> {
        let specification = self.specification;
        let schema_collection = self.schema_collection.clone();
        Ok(PostProcessSchemaBuilder::new(
            CombinatorSubschemaTransform::new(self, description, deprecated),
            JsonSchemaBuilder::new(specification, schema_collection),
        ))
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        if self.nullable && self.combinator == Combinator::OneOf {
            self.subschemas.push(null_schema(self.specification).into());
        }

        let subschemas = self
            .subschemas
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();

        let mut result_schema = match self.combinator {
            Combinator::OneOf => schema! { one_of: subschemas },
            Combinator::AllOf => schema! { all_of: subschemas },
            Combinator::AnyOf => schema! { any_of: subschemas },
        };

        result_schema = if self.nullable && self.combinator != Combinator::OneOf {
            one_of!(result_schema, null_schema(self.specification))
        } else {
            result_schema
        };

        match self.specification {
            Specification::OpenAPI3_0 => {
                result_schema.example = self
                    .examples
                    .and_then(|examples| examples.into_iter().next());
            }
            Specification::OpenAPI3_1 => {
                result_schema.examples = self.examples.map(specification::Examples::Vec);
            }
        }

        result_schema.description = self.description.map(Into::into);

        if self.deprecated {
            result_schema.deprecated = Some(true);
        }

        Ok(result_schema.into())
    }
}

//
// Map
//

macro_rules! describe_signed_integer_key {
    ($only:ident) => {{
        if let Some(only) = $only {
            let mut f = String::new();
            f.push('^');
            f.push('(');

            for (index, variant) in only.iter().enumerate() {
                if index > 0 {
                    f.push('|');
                }

                write!(f, "({variant}([eE][+-]?0+)?)").map_err(Error::custom)?;
            }

            f.push(')');
            f.push('$');

            return Ok(Some(Cow::Owned(f)));
        }

        Ok(Some(Cow::Borrowed(r"^(-?(0|[1-9]\d*)([eE][+-]?0+)?)$")))
    }};
}

macro_rules! describe_unsigned_integer_key {
    ($only:ident) => {{
        if let Some(only) = $only {
            let mut f = String::new();
            f.push('^');
            f.push('(');

            for (index, variant) in only.iter().enumerate() {
                if index > 0 {
                    f.push('|');
                }

                write!(f, "({variant}([eE][+-]?0+)?)").map_err(Error::custom)?;
            }

            f.push(')');
            f.push('$');

            return Ok(Some(Cow::Owned(f)));
        }

        Ok(Some(Cow::Borrowed(r"^((0|[1-9]\d*)([eE][+-]?0+)?)$")))
    }};
}

struct MapKeyPatternBuilder;

fn key_must_be_a_string() -> Error {
    Error::custom("key must be a string")
}

impl IntoSchemaBuilder for MapKeyPatternBuilder {
    type MapKey = Option<Cow<'static, str>>;
    type Ok = Option<Cow<'static, str>>;
    type Error = Error;
    type SchemaBuilder<E: Iterator<Item: Serialize + 'static>> = Self;

    fn into_schema_builder<E: Iterator<Item: Serialize + 'static>>(self) -> Self::SchemaBuilder<E> {
        self
    }
}

impl<E: Iterator<Item: Serialize + 'static>> SchemaBuilder<E> for MapKeyPatternBuilder {
    type MapKey = Option<Cow<'static, str>>;
    type Ok = Option<Cow<'static, str>>;
    type Error = Error;

    type TupleSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type TupleStructSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type StructSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type CombinatorSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type EnumSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type MapSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type OptionSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type NewtypeStructSchemaBuilder = Self;
    type SeqSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;
    type NotSchemaBuilder = Impossible<Self::MapKey, Self::Ok, Self::Error>;

    fn describe_option<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::OptionSchemaBuilder, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn describe_bool<I: IntoIterator<IntoIter = E>>(
        self,
        only: Option<bool>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        if let Some(only) = only {
            if only {
                return Ok(Some(Cow::Borrowed("^(true)$")));
            }

            return Ok(Some(Cow::Borrowed("^(false)$")));
        }

        Ok(Some(Cow::Borrowed("^(true|false)$")))
    }

    fn describe_i8<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<i8>,
        _max: std::ops::Bound<i8>,
        _multiple_of: Option<i8>,
        _format: Option<&'static str>,
        only: Option<&'static [i8]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_signed_integer_key!(only)
    }

    fn describe_i16<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<i16>,
        _max: std::ops::Bound<i16>,
        _multiple_of: Option<i16>,
        _format: Option<&'static str>,
        only: Option<&'static [i16]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_signed_integer_key!(only)
    }

    fn describe_i32<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<i32>,
        _max: std::ops::Bound<i32>,
        _multiple_of: Option<i32>,
        _format: Option<&'static str>,
        only: Option<&'static [i32]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_signed_integer_key!(only)
    }

    fn describe_i64<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<i64>,
        _max: std::ops::Bound<i64>,
        _multiple_of: Option<i64>,
        _format: Option<&'static str>,
        only: Option<&'static [i64]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_signed_integer_key!(only)
    }

    fn describe_u8<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<u8>,
        _max: std::ops::Bound<u8>,
        _multiple_of: Option<u8>,
        _format: Option<&'static str>,
        only: Option<&'static [u8]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_unsigned_integer_key!(only)
    }

    fn describe_u16<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<u16>,
        _max: std::ops::Bound<u16>,
        _multiple_of: Option<u16>,
        _format: Option<&'static str>,
        only: Option<&'static [u16]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_unsigned_integer_key!(only)
    }

    fn describe_u32<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<u32>,
        _max: std::ops::Bound<u32>,
        _multiple_of: Option<u32>,
        _format: Option<&'static str>,
        only: Option<&'static [u32]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_unsigned_integer_key!(only)
    }

    fn describe_u64<I: IntoIterator<IntoIter = E>>(
        self,
        _min: std::ops::Bound<u64>,
        _max: std::ops::Bound<u64>,
        _multiple_of: Option<u64>,
        _format: Option<&'static str>,
        only: Option<&'static [u64]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_unsigned_integer_key!(only)
    }

    fn describe_f32<I: IntoIterator<IntoIter = E>>(
        self,
        _allow_nan: bool,
        _allow_inf: bool,
        _min: std::ops::Bound<f32>,
        _max: std::ops::Bound<f32>,
        _format: Option<&'static str>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Some(Cow::Borrowed(
            r"^(-?(0|[1-9]\d*)(\.\d+)?([eE][+-]?\d+)?)$",
        )))
    }

    fn describe_f64<I: IntoIterator<IntoIter = E>>(
        self,
        _allow_nan: bool,
        _allow_inf: bool,
        _min: std::ops::Bound<f64>,
        _max: std::ops::Bound<f64>,
        _format: Option<&'static str>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Some(Cow::Borrowed(
            r"^(-?(0|[1-9]\d*)(\.\d+)?([eE][+-]?\d+)?)$",
        )))
    }

    fn describe_char<I: IntoIterator<IntoIter = E>>(
        self,
        pattern: Option<&'static str>,
        _format: Option<&'static str>,
        only: Option<&'static [char]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        if let Some(only) = only {
            let mut f = String::new();
            f.push('^');
            f.push('(');

            for (index, variant) in only.iter().enumerate() {
                if index > 0 {
                    f.push('|');
                }

                f.push(*variant);
            }

            f.push(')');
            f.push('$');

            return Ok(Some(Cow::Owned(f)));
        }

        if let Some(pattern) = pattern {
            return Ok(Some(Cow::Borrowed(pattern)));
        }

        Ok(Some(Cow::Borrowed("^(.{1})$")))
    }

    fn describe_str<I: IntoIterator<IntoIter = E>>(
        self,
        min_len: Option<usize>,
        max_len: Option<usize>,
        pattern: Option<&'static str>,
        _format: Option<&'static str>,
        only: Option<&'static [&'static str]>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        if let Some(only) = only {
            let mut f = String::new();
            f.push('^');
            f.push('(');

            for (index, variant) in only.iter().enumerate() {
                if index > 0 {
                    f.push('|');
                }

                f.push_str(variant);
            }

            f.push(')');
            f.push('$');

            return Ok(Some(Cow::Owned(f)));
        }

        if let Some(pattern) = pattern {
            return Ok(Some(Cow::Borrowed(pattern)));
        }

        if let Some(min_len) = min_len {
            if let Some(max_len) = max_len {
                if min_len == max_len {
                    return Ok(Some(Cow::Owned(format!("^(.{{{min_len}}})$"))));
                }
                return Ok(Some(Cow::Owned(format!("^(.{{{min_len},{max_len}}})$"))));
            }

            return Ok(Some(Cow::Owned(format!("^(.{{{min_len},}})$"))));
        }

        if let Some(max_len) = max_len {
            return Ok(Some(Cow::Owned(format!("^(.{{,{max_len}}})$"))));
        }

        Ok(None)
    }

    fn describe_bytes<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn describe_unit<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn describe_unit_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn describe_newtype_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::NewtypeStructSchemaBuilder, Self::Error> {
        Ok(self)
    }

    fn describe_seq<I: IntoIterator<IntoIter = E>>(
        self,
        _min_len: Option<usize>,
        _max_len: Option<usize>,
        _unique: bool,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::SeqSchemaBuilder, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn describe_tuple<I: IntoIterator<IntoIter = E>>(
        self,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::TupleSchemaBuilder, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn describe_tuple_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::TupleStructSchemaBuilder, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn describe_map<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::MapSchemaBuilder, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn describe_struct<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::StructSchemaBuilder, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn describe_enum<I: IntoIterator<IntoIter = E>>(
        self,
        _id: Option<SchemaId>,
        _len: usize,
        _exhaustive: bool,
        _tag: VariantTag,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::EnumSchemaBuilder, Self::Error> {
        Err(key_must_be_a_string()) // TODO: Unit variant is ok!
    }

    fn describe_not<I: IntoIterator<IntoIter = E>>(
        self,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::NotSchemaBuilder, Self::Error> {
        Err(key_must_be_a_string()) // TODO: Can we implement this?
    }

    fn describe_combinator<I: IntoIterator<IntoIter = E>>(
        self,
        _combinator: Combinator,
        _len: usize,
        _description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        _deprecated: bool,
    ) -> Result<Self::CombinatorSchemaBuilder, Self::Error> {
        Err(key_must_be_a_string()) // TODO: Can we implement this?
    }
}

struct NamedMapElementTransform<'s> {
    schema_builder: &'s mut MapJsonSchemaBuilder,
    key: Cow<'static, str>,
    modifier: FieldMod,
    description: Option<&'static str>,
    deprecated: bool,
    is_optional: bool,
    default: Option<JsonValue>,
}

impl Transform<SchemaOrReferenceObject> for NamedMapElementTransform<'_> {
    type Output = ();
    type Error = Error;

    fn transform(self, i: SchemaOrReferenceObject) -> Result<Self::Output, Self::Error> {
        let mut schema = i;

        if let SchemaOrReferenceObject::Schema(schema_object) = &mut schema {
            // TODO: This overrides the schema definition
            if let Some(description) = self.description {
                schema_object.description = Some(description.into());
            }

            // TODO: This overrides the schema definition
            if self.deprecated {
                schema_object.deprecated = Some(true);
            }

            // TODO: This overrides the schema definition
            if let Some(default) = self.default {
                schema_object.default = Some(default);
            }

            match self.modifier {
                FieldMod::Read => schema_object.read_only = Some(true),
                FieldMod::Write => schema_object.write_only = Some(true),
                FieldMod::ReadWrite => {}
            }
        } else {
            let mut additional_schema = self.default.map(|default| schema! { default: default });

            match self.modifier {
                FieldMod::Read => {
                    if let Some(additional_schema) = &mut additional_schema {
                        additional_schema.read_only = Some(true);
                    } else {
                        additional_schema = Some(schema! { read_only: true });
                    }
                }
                FieldMod::Write => {
                    if let Some(additional_schema) = &mut additional_schema {
                        additional_schema.write_only = Some(true);
                    } else {
                        additional_schema = Some(schema! { write_only: true });
                    }
                }
                FieldMod::ReadWrite => {}
            }

            if let Some(additional_schema) = additional_schema {
                schema = all_of!(schema, additional_schema).into();
            }
        }

        let properties = self
            .schema_builder
            .result_schema
            .properties
            .get_or_insert_with(|| {
                self.schema_builder
                    .len
                    .map_or_else(HashMap::new, HashMap::with_capacity)
            });

        if !self.is_optional {
            let key = self.key.clone();
            self.schema_builder
                .result_schema
                .required
                .get_or_insert_with(BTreeSet::new)
                .insert(key);
        }

        let previous = properties.insert(self.key.clone(), schema.into());

        if previous.is_some() {
            return Err(Error::custom(format!(
                "duplicate entry for field {key}",
                key = self.key
            )));
        }

        Ok(())
    }
}

struct AdditionalMapElementTransform<'s> {
    schema_builder: &'s mut MapJsonSchemaBuilder,
    key_pattern: Option<Cow<'static, str>>,
    description: Option<&'static str>,
    deprecated: bool,
}

impl Transform<SchemaOrReferenceObject> for AdditionalMapElementTransform<'_> {
    type Output = ();
    type Error = Error;

    fn transform(self, i: SchemaOrReferenceObject) -> Result<Self::Output, Self::Error> {
        let mut property_schema = i;

        match &mut property_schema {
            SchemaOrReferenceObject::Schema(schema_object) => {
                // TODO: This overrides the schema definition
                if let Some(description) = self.description {
                    schema_object.description = Some(description.into());
                }

                // TODO: This overrides the schema definition
                if self.deprecated {
                    schema_object.deprecated = Some(true);
                }
            }
            SchemaOrReferenceObject::Reference(reference_object) => {
                if let Some(description) = self.description {
                    reference_object.description = Some(description.into());
                }

                // TODO: How can we handle this?
                // if self.deprecated {
                //     reference_object.deprecated = Some(true);
                // }
            }
        }

        if let Some(key_pattern) = self.key_pattern
            && self.schema_builder.specification == Specification::OpenAPI3_1
        {
            let pattern_properties = self.schema_builder.result_schema.pattern_properties.take();

            if let Some(mut pattern_properties) = pattern_properties {
                pattern_properties
                    .entry(key_pattern)
                    .and_modify(|entry| match entry {
                        BoxSchemaOrReferenceObject::Schema(schema_object) => {
                            if let Some(one_of) = &mut schema_object.one_of {
                                one_of.push(property_schema.clone().into());
                            } else {
                                *entry =
                                    one_of!(schema_object.clone(), property_schema.clone()).into();
                            }
                        }
                        BoxSchemaOrReferenceObject::Reference(reference_object) => {
                            *entry =
                                one_of!(reference_object.clone(), property_schema.clone()).into();
                        }
                    })
                    .or_insert_with(|| property_schema.into());
                self.schema_builder.result_schema.pattern_properties = Some(pattern_properties);
            } else {
                let mut pattern_properties = HashMap::new();
                pattern_properties.insert(key_pattern, property_schema.into());
                self.schema_builder.result_schema.pattern_properties = Some(pattern_properties);
            }

            return Ok(());
        }

        let additional_properties = self
            .schema_builder
            .result_schema
            .additional_properties
            .take();

        if let Some(additional_properties) = additional_properties {
            match additional_properties {
                AdditionalProperties::Schema(schema_object) => {
                    if let Some(mut one_of) = schema_object.one_of {
                        one_of.push(property_schema.into());

                        self.schema_builder.result_schema.additional_properties =
                            Some(schema! { one_of: one_of }.into());
                    } else {
                        self.schema_builder.result_schema.additional_properties =
                            Some(one_of!(schema_object, property_schema).into());
                    }
                }
                AdditionalProperties::Boolean(true) => {}
                AdditionalProperties::Boolean(false) => {
                    self.schema_builder.result_schema.additional_properties =
                        Some(property_schema.into());
                }
                AdditionalProperties::Reference(reference_object) => {
                    self.schema_builder.result_schema.additional_properties =
                        Some(one_of!(reference_object, property_schema).into());
                }
            }
        } else {
            self.schema_builder.result_schema.additional_properties = Some(property_schema.into());
        }

        Ok(())
    }
}

enum MapElementTransform<'s> {
    Named(NamedMapElementTransform<'s>),
    Additional(AdditionalMapElementTransform<'s>),
}

impl Transform<SchemaOrReferenceObject> for MapElementTransform<'_> {
    type Output = ();
    type Error = Error;

    fn transform(self, i: SchemaOrReferenceObject) -> Result<Self::Output, Self::Error> {
        match self {
            Self::Named(named_map_element_transform) => named_map_element_transform.transform(i),
            Self::Additional(additional_map_element_transform) => {
                additional_map_element_transform.transform(i)
            }
        }
    }
}

fn serialize_key<K: Schema + serde::Serialize>(key: K) -> Result<Cow<'static, str>, Error> {
    match serde_json::to_value(key).map_err(Error::custom)? {
        JsonValue::Bool(val) => Ok(Cow::Borrowed(if val { "true" } else { "false" })),
        JsonValue::Number(val) => Ok(Cow::Owned(val.to_string())),
        JsonValue::String(val) => Ok(Cow::Owned(val)),
        _ => Err(key_must_be_a_string()),
    }
}
struct MapJsonSchemaBuilder {
    specification: Specification,
    schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
    id: Option<SchemaId>,
    result_schema: SchemaObject,
    len: Option<usize>,
}

impl MapJsonSchemaBuilder {
    #[allow(clippy::too_many_arguments)]
    fn new(
        specification: Specification,
        schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: Option<Vec<JsonValue>>,
        deprecated: bool,
        nullable: bool,
        len: Option<usize>,
    ) -> Self {
        let mut result_schema = SchemaObject {
            description: description.map(Into::into),
            ..SchemaObject::default()
        };

        if deprecated {
            result_schema.deprecated = Some(true);
        }

        match specification {
            Specification::OpenAPI3_0 => {
                result_schema.example = examples.and_then(|examples| examples.into_iter().next());

                result_schema.r#type = Some("object".into());

                if nullable {
                    result_schema.nullable = Some(true);
                }
            }
            Specification::OpenAPI3_1 => {
                result_schema.examples = examples.map(specification::Examples::Vec);

                if nullable {
                    result_schema.r#type = Some(vec!["object".into(), "null".into()].into());
                } else {
                    result_schema.r#type = Some("object".into());
                }
            }
        }

        Self {
            specification,
            schema_collection,
            id,
            result_schema,
            len,
        }
    }
}

impl MapSchemaBuilder for MapJsonSchemaBuilder {
    type MapKey = Option<Cow<'static, str>>;
    type Ok = SchemaOrReferenceObject;
    type Error = Error;

    type MapKeySchemaBuilder = MapKeyPatternBuilder;
    type MapValueSchemaBuilder<'a>
        = PostProcessSchemaBuilder<MapElementTransform<'a>, JsonSchemaBuilder>
    where
        Self: 'a;

    fn describe_element<'a, K: Schema + serde::Serialize>(
        &'a mut self,
        key: K,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'a>, Self::Error> {
        let specification = self.specification;
        let schema_collection = self.schema_collection.clone();

        Ok(PostProcessSchemaBuilder::new(
            MapElementTransform::Named(NamedMapElementTransform {
                schema_builder: self,
                key: serialize_key(key)?,
                modifier,
                description,
                deprecated,
                is_optional: false,
                default: None,
            }),
            JsonSchemaBuilder::new(specification, schema_collection),
        ))
    }

    fn describe_element_optional<'a, K: Schema + serde::Serialize, F: Serialize>(
        &'a mut self,
        key: K,
        modifier: FieldMod,
        default: Option<F>,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'a>, Self::Error> {
        let specification = self.specification;
        let schema_collection = self.schema_collection.clone();
        let default = default
            .map(|default| serde_json::to_value(default))
            .transpose()
            .map_err(Error::custom)?;
        Ok(PostProcessSchemaBuilder::new(
            MapElementTransform::Named(NamedMapElementTransform {
                schema_builder: self,
                key: serialize_key(key)?,
                modifier,
                description,
                deprecated,
                is_optional: true,
                default,
            }),
            JsonSchemaBuilder::new(specification, schema_collection),
        ))
    }

    fn describe_additional_elements<'a, K, I: Iterator<Item: Serialize + 'static>>(
        &'a mut self,
        describe_key: K,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'a>, Self::Error>
    where
        K: FnOnce(
            <Self::MapKeySchemaBuilder as IntoSchemaBuilder>::SchemaBuilder<I>,
        )
            -> Result<<Self::MapKeySchemaBuilder as IntoSchemaBuilder>::Ok, Self::Error>,
    {
        let specification = self.specification;
        let schema_collection = self.schema_collection.clone();
        let key_pattern: Option<Cow<'static, str>> = describe_key(MapKeyPatternBuilder)?;

        Ok(PostProcessSchemaBuilder::new(
            MapElementTransform::Additional(AdditionalMapElementTransform {
                schema_builder: self,
                key_pattern,
                description,
                deprecated,
            }),
            JsonSchemaBuilder::new(specification, schema_collection),
        ))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if let Some(schema_collection) = self.schema_collection {
            let mut schema_collection = schema_collection.borrow_mut();
            if let Some(schema_id) = self.id {
                let schema_ref = schema_collection.set(&schema_id, self.result_schema.into());
                return Ok(ReferenceObject {
                    r#ref: schema_ref,
                    summary: None,
                    description: None,
                }
                .into());
            }
        }

        Ok(self.result_schema.into())
    }
}

//
// Enum
//

struct StructVariantJsonSchemaBuilder<'s> {
    enum_builder: &'s mut EnumJsonSchemaBuilder,
    name: &'static str,
    description: Option<&'static str>,
    deprecated: bool,
    inner: StructJsonSchemaBuilder,
}

impl<'s> StructVariantJsonSchemaBuilder<'s> {
    fn new(
        enum_builder: &'s mut EnumJsonSchemaBuilder,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        len: usize,
    ) -> Self {
        let tag = enum_builder.tag;
        let specification = enum_builder.specification;
        let schema_collection = enum_builder.schema_collection.clone();
        let capacity = match tag {
            VariantTag::InternallyTagged { .. } => len + 1,
            _ => len,
        };

        Self {
            enum_builder,
            name,
            description,
            deprecated,
            inner: StructJsonSchemaBuilder::new(
                specification,
                schema_collection,
                None,
                match tag {
                    VariantTag::Untagged | VariantTag::InternallyTagged { .. } => description,
                    _ => None,
                },
                None,
                match tag {
                    VariantTag::Untagged | VariantTag::InternallyTagged { .. } => deprecated,
                    _ => false,
                },
                false,
                capacity,
            ),
        }
    }
}

impl StructVariantSchemaBuilder for StructVariantJsonSchemaBuilder<'_> {
    type MapKey = Option<Cow<'static, str>>;
    type Error = Error;

    type FieldSchemaBuilder<'a>
        = PostProcessSchemaBuilder<StructFieldTransform<'a>, JsonSchemaBuilder>
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        self.inner
            .describe_field(key, modifier, description, deprecated)
    }

    fn describe_field_optional<'a, F: Serialize>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        default: Option<F>,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        self.inner
            .describe_field_optional(key, modifier, default, description, deprecated)
    }

    fn end(self) -> Result<(), Self::Error> {
        let subschema = self.inner.end()?;

        match self.enum_builder.tag {
            VariantTag::Untagged => {
                self.enum_builder.subschemas.push(subschema);
            }
            VariantTag::ExternallyTagged => {
                self.enum_builder.subschemas.push(
                    {
                        let mut subschema = schema! {
                            r#type: "object".into(),
                            properties: map! {
                                self.name.into() => subschema.into()
                            },
                            required: set![self.name.into()],
                        };

                        subschema.description = self.description.map(Into::into);

                        if self.deprecated {
                            subschema.deprecated = Some(true);
                        }

                        subschema
                    }
                    .into(),
                );
            }
            VariantTag::InternallyTagged { tag } => {
                let mut subschema = subschema;

                if let SchemaOrReferenceObject::Schema(schema_object) = &mut subschema {
                    schema_object.properties.get_or_insert_default().insert(
                        tag.into(),
                        schema! {
                            r#type: "string".into(),
                            r#enum: vec![self.name.into()],
                        }
                        .into(),
                    );

                    schema_object
                        .required
                        .get_or_insert_default()
                        .insert(tag.into());
                } else {
                    subschema = all_of!(
                        subschema,
                        schema! { r#type: "object".into(), properties: map![tag.into() => schema! {
                            r#type: "string".into(),
                            r#enum: vec![self.name.into()],
                        }.into()],  required: set! [tag.into()] }
                    )
                    .into();
                }

                self.enum_builder.subschemas.push(subschema);
            }
            VariantTag::AdjacentlyTagged { tag, content } => {
                self.enum_builder.subschemas.push(
                    {
                        let mut subschema = schema! {
                            r#type: "object".into(),
                            properties: map! {
                                tag.into() => schema! {
                                    r#type: "string".into(),
                                    r#enum: vec![self.name.into()],
                                }.into(),
                                content.into() => subschema.into()
                            },
                            required: set![tag.into(), content.into()],
                        };

                        subschema.description = self.description.map(Into::into);

                        if self.deprecated {
                            subschema.deprecated = Some(true);
                        }

                        subschema
                    }
                    .into(),
                );
            }
        }

        Ok(())
    }
}

struct TupleVariantJsonSchemaBuilder<'s> {
    enum_builder: &'s mut EnumJsonSchemaBuilder,
    name: &'static str,
    description: Option<&'static str>,
    deprecated: bool,
    inner: TupleJsonSchemaBuilder,
}

impl<'s> TupleVariantJsonSchemaBuilder<'s> {
    fn new(
        enum_builder: &'s mut EnumJsonSchemaBuilder,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        len: usize,
    ) -> Self {
        let tag = enum_builder.tag;
        let specification = enum_builder.specification;
        let schema_collection = enum_builder.schema_collection.clone();

        Self {
            enum_builder,
            name,
            description,
            deprecated,
            inner: TupleJsonSchemaBuilder::new(
                specification,
                schema_collection,
                None,
                if tag == VariantTag::Untagged {
                    description
                } else {
                    None
                },
                None,
                if tag == VariantTag::Untagged {
                    deprecated
                } else {
                    false
                },
                false,
                len,
            ),
        }
    }
}

impl TupleVariantSchemaBuilder for TupleVariantJsonSchemaBuilder<'_> {
    type MapKey = Option<Cow<'static, str>>;
    type Error = Error;

    type FieldSchemaBuilder<'a>
        = PostProcessSchemaBuilder<TupleElementTransform<'a>, JsonSchemaBuilder>
    where
        Self: 'a;

    fn describe_field<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error> {
        self.inner.describe_element(description, deprecated)
    }

    fn end(self) -> Result<(), Self::Error> {
        let subschema = self.inner.end()?;

        match self.enum_builder.tag {
            VariantTag::Untagged => {
                self.enum_builder.subschemas.push(subschema);
            }
            VariantTag::ExternallyTagged => {
                self.enum_builder.subschemas.push(
                    {
                        let mut subschema = schema! {
                            r#type: "object".into(),
                            properties: map! {
                                self.name.into() => subschema.into()
                            },
                            required: set![self.name.into()],
                        };

                        subschema.description = self.description.map(Into::into);

                        if self.deprecated {
                            subschema.deprecated = Some(true);
                        }

                        subschema
                    }
                    .into(),
                );
            }
            VariantTag::InternallyTagged { tag: _tag } => {
                unreachable!();
            }
            VariantTag::AdjacentlyTagged { tag, content } => {
                self.enum_builder.subschemas.push(
                    {
                        let mut subschema = schema! {
                            r#type: "object".into(),
                            properties: map! {
                                tag.into() => schema! {
                                    r#type: "string".into(),
                                    r#enum: vec![self.name.into()],
                                }.into(),
                                content.into() => subschema.into()
                            },
                            required: set![tag.into(), content.into()],
                        };

                        subschema.description = self.description.map(Into::into);

                        if self.deprecated {
                            subschema.deprecated = Some(true);
                        }

                        subschema
                    }
                    .into(),
                );
            }
        }

        Ok(())
    }
}

struct NewTypeVariantTransform<'s> {
    schema_builder: &'s mut EnumJsonSchemaBuilder,
    name: &'static str,
    description: Option<&'static str>,
    deprecated: bool,
}

impl Transform<SchemaOrReferenceObject> for NewTypeVariantTransform<'_> {
    type Output = ();
    type Error = Error;

    #[allow(clippy::too_many_lines)] // TODO: Refactor this
    fn transform(self, i: SchemaOrReferenceObject) -> Result<Self::Output, Self::Error> {
        let mut subschema = i;

        match self.schema_builder.tag {
            VariantTag::Untagged => {
                match &mut subschema {
                    SchemaOrReferenceObject::Schema(schema_object) => {
                        // TODO: This overrides the schema definition
                        if let Some(description) = self.description {
                            schema_object.description = Some(description.into());
                        }

                        // TODO: This overrides the schema definition
                        if self.deprecated {
                            schema_object.deprecated = Some(true);
                        }
                    }
                    SchemaOrReferenceObject::Reference(reference_object) => {
                        if let Some(description) = self.description {
                            reference_object.description = Some(description.into());
                        }

                        // TODO: How can we handle this?
                        // if self.deprecated {
                        //     reference_object.deprecated = Some(true);
                        // }
                    }
                }

                self.schema_builder.subschemas.push(subschema);
            }
            VariantTag::ExternallyTagged => {
                self.schema_builder.subschemas.push(
                    {
                        let mut subschema = schema! {
                            r#type: "object".into(),
                            properties: map! {
                                self.name.into() => subschema.into()
                            },
                            required: set![self.name.into()],
                        };

                        subschema.description = self.description.map(Into::into);

                        if self.deprecated {
                            subschema.deprecated = Some(true);
                        }

                        subschema
                    }
                    .into(),
                );
            }
            VariantTag::InternallyTagged { tag } => {
                // TODO: How to validate this for reference-objects?
                if let SchemaOrReferenceObject::Schema(schema_object) = &subschema
                    && !schema_object
                        .r#type
                        .as_ref()
                        .is_some_and(|r#type| match r#type {
                            OneOrMany::One(item) => *item == "object",
                            OneOrMany::Many(items) => items.iter().any(|item| *item == "object"),
                        })
                {
                    return Err(Error::custom(
                        "internally tagged enum must contain struct or map in newtype variant",
                    ));
                }

                if let SchemaOrReferenceObject::Schema(schema_object) = &mut subschema {
                    schema_object.properties.get_or_insert_default().insert(
                        tag.into(),
                        schema! {
                            r#type: "string".into(),
                            r#enum: vec![self.name.into()],
                        }
                        .into(),
                    );

                    schema_object
                        .required
                        .get_or_insert_default()
                        .insert(tag.into());

                    if let Some(example) = &mut schema_object.example {
                        if let serde_json::Value::Object(obj) = example {
                            obj.insert(tag.into(), serde_json::Value::String(self.name.into()));
                        } else {
                            // Expected an object
                            // As a workaround, just remove the example, as we cannot patch it
                            // TODO: Can we solve this?
                            schema_object.example = None;
                        }
                    }

                    if let Some(examples) = &mut schema_object.examples {
                        match examples {
                            Examples::Vec(values) => {
                                values.retain_mut(|example| {
                                    if let serde_json::Value::Object(obj) = example {
                                        obj.insert(
                                            tag.into(),
                                            serde_json::Value::String(self.name.into()),
                                        );

                                        true
                                    } else {
                                        // Expected an object
                                        // As a workaround, just remove the example, as we cannot patch it
                                        // TODO: Can we solve this?

                                        false
                                    }
                                });
                            }
                            Examples::Map(examples) => {
                                examples.retain(|_, example| {
                                    if let serde_json::Value::Object(obj) = example {
                                        obj.insert(
                                            tag.into(),
                                            serde_json::Value::String(self.name.into()),
                                        );
                                        true
                                    } else {
                                        // Expected an object
                                        // As a workaround, just remove the example, as we cannot patch it
                                        // TODO: Can we solve this?
                                        false
                                    }
                                });
                            }
                        }
                    }

                    // TODO: This overrides the schema definition
                    if let Some(description) = self.description {
                        schema_object.description = Some(description.into());
                    }

                    if self.deprecated {
                        schema_object.deprecated = Some(true);
                    }
                } else {
                    let mut combined_schemas = all_of!(
                        subschema,
                        schema! {
                            r#type: "object".into(),
                            properties: map![
                                tag.into() => schema! {
                                    r#type: "string".into(),
                                    r#enum: vec![self.name.into()],
                                }.into()],
                            required: set! [tag.into()]
                        }
                    );

                    combined_schemas.description = self.description.map(Into::into);

                    if self.deprecated {
                        combined_schemas.deprecated = Some(true);
                    }

                    subschema = combined_schemas.into();
                }

                self.schema_builder.subschemas.push(subschema);
            }
            VariantTag::AdjacentlyTagged { tag, content } => {
                self.schema_builder.subschemas.push(
                    {
                        let mut subschema = schema! {
                            r#type: "object".into(),
                            properties: map! {
                                tag.into() => schema! {
                                    r#type: "string".into(),
                                    r#enum: vec![self.name.into()],
                                }.into(),
                                content.into() => subschema.into()
                            },
                            required: set![tag.into(), content.into()],
                        };

                        subschema.description = self.description.map(Into::into);

                        if self.deprecated {
                            subschema.deprecated = Some(true);
                        }

                        subschema
                    }
                    .into(),
                );
            }
        }

        Ok(())
    }
}

struct EnumJsonSchemaBuilder {
    specification: Specification,
    schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
    id: Option<SchemaId>,
    description: Option<&'static str>,
    examples: Option<Vec<JsonValue>>,
    deprecated: bool,
    tag: VariantTag,
    subschemas: Vec<SchemaOrReferenceObject>,
    variant_names: Vec<&'static str>,
    exhaustive: bool,
}

impl EnumJsonSchemaBuilder {
    #[allow(clippy::too_many_arguments)]
    fn new(
        specification: Specification,
        schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: Option<Vec<JsonValue>>,
        deprecated: bool,
        nullable: bool, // TODO: What to do with this??
        tag: VariantTag,
        len: usize,
        exhaustive: bool,
    ) -> Self {
        let mut capacity = len;

        if nullable {
            capacity += 1;
        }

        if !exhaustive {
            capacity += 1;
        }

        Self {
            specification,
            schema_collection,
            id,
            description,
            examples,
            deprecated,
            tag,
            subschemas: Vec::with_capacity(capacity),
            variant_names: Vec::with_capacity(capacity),
            exhaustive,
        }
    }
}

impl EnumSchemaBuilder for EnumJsonSchemaBuilder {
    type MapKey = Option<Cow<'static, str>>;
    type Ok = SchemaOrReferenceObject;
    type Error = Error;

    type TupleVariantSchemaBuilder<'a>
        = TupleVariantJsonSchemaBuilder<'a>
    where
        Self: 'a;

    type StructVariantSchemaBuilder<'a>
        = StructVariantJsonSchemaBuilder<'a>
    where
        Self: 'a;

    type NewTypeVariantSchemaBuilder<'a>
        = PostProcessSchemaBuilder<NewTypeVariantTransform<'a>, JsonSchemaBuilder>
    where
        Self: 'a;

    fn describe_unit_variant(
        &mut self,
        _index: u32,
        id: SchemaId,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<(), Self::Error> {
        self.variant_names.push(id.name());
        let mut subschema = match self.tag {
            VariantTag::Untagged => {
                // The (non-present) content is serialized as null by serde_json
                null_schema(self.specification)
            }
            VariantTag::ExternallyTagged => {
                schema! {
                    r#type: "string".into(),
                    r#enum: vec![id.name().into()],
                }
            }
            VariantTag::InternallyTagged { tag } | VariantTag::AdjacentlyTagged { tag, .. } => {
                schema! {
                    r#type: "object".into(),
                    properties: map! {
                        tag.into() => schema! {
                            r#type: "string".into(),
                            r#enum: vec![id.name().into()],
                        }.into()
                    },
                    required: set![tag.into()]
                }
            }
        };

        subschema.description = description.map(Into::into);

        if deprecated {
            subschema.deprecated = Some(true);
        }

        self.subschemas.push(subschema.into());

        Ok(())
    }

    fn describe_newtype_variant<'a>(
        &'a mut self,
        _index: u32,
        id: SchemaId,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::NewTypeVariantSchemaBuilder<'a>, Self::Error> {
        self.variant_names.push(id.name());
        let specification = self.specification;
        let schema_collection = self.schema_collection.clone();
        Ok(PostProcessSchemaBuilder::new(
            NewTypeVariantTransform {
                schema_builder: self,
                name: id.name(),
                description,
                deprecated,
            },
            JsonSchemaBuilder::new(specification, schema_collection),
        ))
    }

    fn describe_tuple_variant<'a>(
        &'a mut self,
        _index: u32,
        id: SchemaId,
        len: usize,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::TupleVariantSchemaBuilder<'a>, Self::Error> {
        self.variant_names.push(id.name());
        Ok(Self::TupleVariantSchemaBuilder::new(
            self,
            id.name(),
            description,
            deprecated,
            len,
        ))
    }

    fn describe_struct_variant<'a>(
        &'a mut self,
        _index: u32,
        id: SchemaId,
        len: usize,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::StructVariantSchemaBuilder<'a>, Self::Error> {
        self.variant_names.push(id.name());
        Ok(Self::StructVariantSchemaBuilder::new(
            self,
            id.name(),
            description,
            deprecated,
            len,
        ))
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        // TODO: Discriminator, nullable

        if !self.exhaustive {
            match self.tag {
                VariantTag::Untagged => self.subschemas.push(schema!().into()),
                VariantTag::ExternallyTagged => match self.specification {
                    Specification::OpenAPI3_0 => {
                        self.subschemas.push(
                            schema! {
                                r#type: "object".into(),
                                additional_properties: AdditionalProperties::Boolean(true)
                            }
                            .into(),
                        );
                    }
                    Specification::OpenAPI3_1 => {
                        let variants_not_match_pattern =
                            build_variants_not_match_pattern(&self.variant_names);
                        self.subschemas.push(
                            schema! {
                                r#type: "object".into(),
                                pattern_properties: map! {
                                    variants_not_match_pattern.into() => schema!().into()
                                }
                            }
                            .into(),
                        );
                    }
                },
                VariantTag::InternallyTagged { tag } => {
                    let variants_not_match_pattern =
                        build_variants_not_match_pattern(&self.variant_names);
                    self.subschemas.push(
                        schema! {
                            r#type: "object".into(),
                            required: set! (tag.into()),
                            properties: map!{
                                tag.into() => schema! {
                                    r#type: "string".into(),
                                    pattern: variants_not_match_pattern.into()
                                }.into()
                            }
                        }
                        .into(),
                    );
                }
                VariantTag::AdjacentlyTagged { tag, content } => {
                    let variants_not_match_pattern =
                        build_variants_not_match_pattern(&self.variant_names);
                    self.subschemas.push(
                        schema! {
                            r#type: "object".into(),
                            required: set! (tag.into(), content.into()),
                            properties: map!{
                                tag.into() => schema! {
                                    r#type: "string".into(),
                                    pattern: variants_not_match_pattern.into()
                                }.into(),
                                content.into() => schema!().into()
                            }
                        }
                        .into(),
                    );
                }
            }
        }

        let mut result_schema = schema! {
            any_of: self.subschemas.into_iter().map(Into::into).collect::<Vec<_>>()
        };

        result_schema.description = self.description.map(Into::into);
        result_schema.deprecated = if self.deprecated { Some(true) } else { None };

        match self.specification {
            Specification::OpenAPI3_0 => {
                result_schema.example = self
                    .examples
                    .and_then(|examples| examples.into_iter().next());
            }
            Specification::OpenAPI3_1 => {
                result_schema.examples = self.examples.map(specification::Examples::Vec);
            }
        }

        if let Some(schema_collection) = self.schema_collection {
            let mut schema_collection = schema_collection.borrow_mut();
            if let Some(schema_id) = self.id {
                let schema_ref = schema_collection.set(&schema_id, result_schema.into());
                return Ok(ReferenceObject {
                    r#ref: schema_ref,
                    summary: None,
                    description: None,
                }
                .into());
            }
        }

        Ok(result_schema.into())
    }
}

fn build_variants_not_match_pattern(variants: &[&str]) -> String {
    let mut builder = String::new();

    if variants.is_empty() {
        builder.push('^');
        builder.push('.');
        builder.push('*');
        builder.push('$');
    } else if variants.len() == 1 {
        build_variant_not_match_pattern(variants[0], &mut builder);
    } else {
        for variant in variants {
            builder.push('(');
            builder.push('?');
            builder.push('=');
            build_variant_not_match_pattern(variant, &mut builder);
            builder.push(')');
        }
        builder.push('^');
        builder.push('.');
        builder.push('*');
        builder.push('$');
    }

    builder
}

fn build_variant_not_match_pattern(variant: &str, builder: &mut String) {
    // TODO: If we can construct an iterator that iterates
    // all but the last element, we are not required to perform
    // this O(N) operation
    let variant_char_len = variant.chars().count();

    for (char_idx, char) in variant.chars().enumerate() {
        if char_idx > 0 {
            builder.push('|');
        }

        builder.push('^');
        for previous_char in variant.chars().take(char_idx) {
            builder.push(previous_char);
        }
        builder.push('[');
        builder.push('^');
        builder.push(char);
        builder.push('\\');
        builder.push('n');
        builder.push(']');
        builder.push('.');
        builder.push('*');
        builder.push('$');
        builder.push('|');

        builder.push('^');
        for previous_char in variant.chars().take(char_idx) {
            builder.push(previous_char);
        }
        builder.push(char);
        if char_idx >= (variant_char_len - 1) {
            builder.push('.');
            builder.push('+');
        }

        builder.push('$');
    }
}

//
// Schema
//

macro_rules! describe_integer {
    (
        $type:ty,
        $self:ident,
        $min:ident,
        $max:ident,
        $multiple_of:ident,
        $format:ident,
        $only:ident,
        $description:ident,
        $examples:expr,
        $deprecated:ident,
    ) => {{
        let mut result = schema! {};

        result.description = $self.description.or($description).map(Into::into);

        if $deprecated || $self.deprecated {
            result.deprecated = Some(true);
        }

        match $self.specification {
            Specification::OpenAPI3_0 => {
                result.example = $examples.into_iter().next();

                result.r#type = Some("integer".into());

                match $min {
                    std::ops::Bound::Unbounded => {
                        result.minimum = Some(serde_json::Number::from(<$type>::MIN));
                    }
                    std::ops::Bound::Included(min) => {
                        result.minimum = Some(serde_json::Number::from(min));
                    }
                    std::ops::Bound::Excluded(min) => {
                        result.minimum = Some(serde_json::Number::from(min));
                        result.exclusive_minimum = Some(EitherUntagged::Left(true));
                    }
                };

                match $max {
                    std::ops::Bound::Unbounded => {
                        result.maximum = Some(serde_json::Number::from(<$type>::MAX));
                    }
                    std::ops::Bound::Included(max) => {
                        result.maximum = Some(serde_json::Number::from(max));
                    }
                    std::ops::Bound::Excluded(max) => {
                        result.maximum = Some(serde_json::Number::from(max));
                        result.exclusive_maximum = Some(EitherUntagged::Left(true));
                    }
                };

                if $self.nullable {
                    result.nullable = Some(true);

                    if let Some(only) = $only {
                        let mut r#enum = only
                            .iter()
                            .map(|value| serde_json::Value::from(*value))
                            .collect::<Vec<_>>();

                        r#enum.push(JsonValue::Null);

                        result.r#enum = Some(r#enum);
                    }
                } else {
                    result.r#enum = $only.map(|only| {
                        only.iter()
                            .map(|value| serde_json::Value::from(*value))
                            .collect()
                    });
                }
            }
            Specification::OpenAPI3_1 => {
                result.examples = Some(specification::Examples::Vec($examples));

                if $self.nullable {
                    result.r#type = Some(vec!["integer".into(), "null".into()].into());
                } else {
                    result.r#type = Some("integer".into());
                }

                // TODO: If self.nullable, does this have to be part of the enum spec in OAS3.1??
                result.r#enum = $only.map(|only| {
                    only.iter()
                        .map(|value| serde_json::Value::from(*value))
                        .collect()
                });

                match $min {
                    std::ops::Bound::Unbounded => {
                        result.minimum = Some(serde_json::Number::from(<$type>::MIN));
                    }
                    std::ops::Bound::Included(min) => {
                        result.minimum = Some(serde_json::Number::from(min));
                    }
                    std::ops::Bound::Excluded(min) => {
                        result.exclusive_minimum =
                            Some(EitherUntagged::Right(serde_json::Number::from(min)));
                    }
                };

                match $max {
                    std::ops::Bound::Unbounded => {
                        result.maximum = Some(serde_json::Number::from(<$type>::MAX));
                    }
                    std::ops::Bound::Included(max) => {
                        result.maximum = Some(serde_json::Number::from(max));
                    }
                    std::ops::Bound::Excluded(max) => {
                        result.exclusive_maximum =
                            Some(EitherUntagged::Right(serde_json::Number::from(max)));
                    }
                };
            }
        }

        result.multiple_of = $multiple_of.map(|multiple_of| serde_json::Number::from(multiple_of));
        result.format = $format.map(Into::into);

        Ok(result.into())
    }};
}

macro_rules! describe_float {
    (
        $type:ty,
        $self:ident,
        $min:ident,
        $max:ident,
        $format:ident,
        $description:ident,
        $examples:expr,
        $deprecated:ident
    ) => {{
        let mut result = schema! {};

        result.description = $self.description.or($description).map(Into::into);

        if $deprecated || $self.deprecated {
            result.deprecated = Some(true);
        }

        match $self.specification {
            Specification::OpenAPI3_0 => {
                result.example = $examples.into_iter().next();
                result.r#type = Some("number".into());

                match $min {
                    std::ops::Bound::Unbounded => {}
                    std::ops::Bound::Included(min) => {
                        // TODO: Does the string interpolation work!
                        let min = serde_json::Number::from_f64(min.into()).ok_or_else(|| {
                            Error::custom(format!(
                                "expected Json Number compatible $type, got {}",
                                min
                            ))
                        })?;

                        result.minimum = Some(serde_json::Number::from(min));
                    }
                    std::ops::Bound::Excluded(min) => {
                        let min = serde_json::Number::from_f64(min.into()).ok_or_else(|| {
                            Error::custom(format!(
                                "expected Json Number compatible $type, got {}",
                                min
                            ))
                        })?;

                        result.minimum = Some(serde_json::Number::from(min));
                        result.exclusive_minimum = Some(EitherUntagged::Left(true));
                    }
                };

                match $max {
                    std::ops::Bound::Unbounded => {}
                    std::ops::Bound::Included(max) => {
                        let max = serde_json::Number::from_f64(max.into()).ok_or_else(|| {
                            Error::custom(format!(
                                "expected Json Number compatible $type, got {}",
                                max
                            ))
                        })?;

                        result.maximum = Some(serde_json::Number::from(max));
                    }
                    std::ops::Bound::Excluded(max) => {
                        let max = serde_json::Number::from_f64(max.into()).ok_or_else(|| {
                            Error::custom(format!(
                                "expected Json Number compatible $type, got {}",
                                max
                            ))
                        })?;

                        result.maximum = Some(serde_json::Number::from(max));
                        result.exclusive_maximum = Some(EitherUntagged::Left(true));
                    }
                };

                if $self.nullable {
                    result.nullable = Some(true);
                }
            }
            Specification::OpenAPI3_1 => {
                result.examples = Some(specification::Examples::Vec($examples));

                if $self.nullable {
                    result.r#type = Some(vec!["number".into(), "null".into()].into());
                } else {
                    result.r#type = Some("number".into());
                }

                match $min {
                    std::ops::Bound::Unbounded => {}
                    std::ops::Bound::Included(min) => {
                        let min = serde_json::Number::from_f64(min.into()).ok_or_else(|| {
                            Error::custom(format!(
                                "expected Json Number compatible $type, got {}",
                                min
                            ))
                        })?;

                        result.minimum = Some(serde_json::Number::from(min));
                    }
                    std::ops::Bound::Excluded(min) => {
                        let min = serde_json::Number::from_f64(min.into()).ok_or_else(|| {
                            Error::custom(format!(
                                "expected Json Number compatible $type, got {}",
                                min
                            ))
                        })?;

                        result.exclusive_minimum =
                            Some(EitherUntagged::Right(serde_json::Number::from(min)));
                    }
                };

                match $max {
                    std::ops::Bound::Unbounded => {}
                    std::ops::Bound::Included(max) => {
                        let max = serde_json::Number::from_f64(max.into()).ok_or_else(|| {
                            Error::custom(format!(
                                "expected Json Number compatible $type, got {}",
                                max
                            ))
                        })?;

                        result.maximum = Some(serde_json::Number::from(max));
                    }
                    std::ops::Bound::Excluded(max) => {
                        let max = serde_json::Number::from_f64(max.into()).ok_or_else(|| {
                            Error::custom(format!(
                                "expected Json Number compatible $type, got {}",
                                max
                            ))
                        })?;

                        result.exclusive_maximum =
                            Some(EitherUntagged::Right(serde_json::Number::from(max)));
                    }
                };
            }
        }

        result.format = $format.map(Into::into);

        Ok(result.into())
    }};
}

struct NotSchemaTransform {
    specification: Specification,
    description: Option<&'static str>,
    examples: Option<Vec<JsonValue>>,
    deprecated: bool,
    nullable: bool,
}

impl NotSchemaTransform {
    const fn new(
        specification: Specification,
        description: Option<&'static str>,
        examples: Option<Vec<JsonValue>>,
        deprecated: bool,
        nullable: bool,
    ) -> Self {
        Self {
            specification,
            description,
            examples,
            deprecated,
            nullable,
        }
    }
}

impl Transform<SchemaOrReferenceObject> for NotSchemaTransform {
    type Output = SchemaOrReferenceObject;
    type Error = Error;

    fn transform(self, i: SchemaOrReferenceObject) -> Result<Self::Output, Self::Error> {
        let mut result = schema! {
            not: i.into()
        };

        match self.specification {
            Specification::OpenAPI3_0 => {
                result.example = self
                    .examples
                    .and_then(|examples| examples.into_iter().next());
            }
            Specification::OpenAPI3_1 => {
                result.examples = self.examples.map(specification::Examples::Vec);
            }
        }

        result = if self.nullable {
            one_of!(result, null_schema(self.specification))
        } else {
            result
        };

        result.description = self.description.map(Into::into);

        if self.deprecated {
            result.deprecated = Some(true);
        }

        Ok(result.into())
    }
}

struct SeqSchemaTransform {
    specification: Specification,
    description: Option<&'static str>,
    examples: Option<Vec<JsonValue>>,
    deprecated: bool,
    min_len: Option<usize>,
    max_len: Option<usize>,
    unique: bool,
    nullable: bool,
}

impl SeqSchemaTransform {
    #[allow(clippy::too_many_arguments)]
    const fn new(
        specification: Specification,
        description: Option<&'static str>,
        examples: Option<Vec<JsonValue>>,
        deprecated: bool,
        min_len: Option<usize>,
        max_len: Option<usize>,
        unique: bool,
        nullable: bool,
    ) -> Self {
        Self {
            specification,
            description,
            examples,
            deprecated,
            min_len,
            max_len,
            unique,
            nullable,
        }
    }
}

impl Transform<SchemaOrReferenceObject> for SeqSchemaTransform {
    type Output = SchemaOrReferenceObject;
    type Error = Error;

    fn transform(self, i: SchemaOrReferenceObject) -> Result<Self::Output, Self::Error> {
        let mut result = SchemaObject::default();

        match self.specification {
            Specification::OpenAPI3_0 => {
                result.example = self
                    .examples
                    .and_then(|examples| examples.into_iter().next());

                result.r#type = Some("array".into());

                if self.nullable {
                    result.nullable = Some(true);
                }
            }
            Specification::OpenAPI3_1 => {
                result.examples = self.examples.map(specification::Examples::Vec);

                if self.nullable {
                    result.r#type = Some(vec!["array".into(), "null".into()].into());
                } else {
                    result.r#type = Some("array".into());
                }
            }
        }

        result.min_items = self.min_len.map(serde_json::Number::from);
        result.max_items = self.max_len.map(serde_json::Number::from);
        result.unique_items = if self.unique { Some(true) } else { None };
        result.items = Some(i.into());

        result.description = self.description.map(Into::into);

        if self.deprecated {
            result.deprecated = Some(true);
        }

        Ok(result.into())
    }
}

struct SchemaCollectionTransform {
    schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
    id: Option<SchemaId>,
}

impl SchemaCollectionTransform {
    const fn new(
        schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
        id: Option<SchemaId>,
    ) -> Self {
        Self {
            schema_collection,
            id,
        }
    }
}

impl Transform<SchemaOrReferenceObject> for SchemaCollectionTransform {
    type Output = SchemaOrReferenceObject;
    type Error = Error;

    fn transform(self, i: SchemaOrReferenceObject) -> Result<Self::Output, Self::Error> {
        if let Some(schema_collection) = self.schema_collection {
            let mut schema_collection = schema_collection.borrow_mut();
            if let Some(schema_id) = self.id {
                let schema_ref = schema_collection.set(&schema_id, i);
                return Ok(ReferenceObject {
                    r#ref: schema_ref,
                    summary: None,
                    description: None,
                }
                .into());
            }
        }

        Ok(i)
    }
}

struct JsonSchemaBuilder {
    specification: Specification,
    schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
    description: Option<&'static str>,
    examples: Option<Vec<JsonValue>>,
    deprecated: bool,
    nullable: bool,
}

impl JsonSchemaBuilder {
    const fn new(
        specification: Specification,
        schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
    ) -> Self {
        Self {
            specification,
            schema_collection,
            description: None,
            examples: None,
            deprecated: false,
            nullable: false,
        }
    }
}

impl IntoSchemaBuilder for JsonSchemaBuilder {
    type MapKey = Option<Cow<'static, str>>;
    type Ok = SchemaOrReferenceObject;
    type Error = Error;

    type SchemaBuilder<E: Iterator<Item: Serialize + 'static>> = Self;

    fn into_schema_builder<E: Iterator<Item: Serialize + 'static>>(self) -> Self::SchemaBuilder<E> {
        self
    }
}

impl<E: Iterator<Item: Serialize + 'static>> SchemaBuilder<E> for JsonSchemaBuilder {
    type MapKey = Option<Cow<'static, str>>;
    type Ok = SchemaOrReferenceObject;
    type Error = Error;

    type TupleSchemaBuilder = TupleJsonSchemaBuilder;
    type TupleStructSchemaBuilder =
        either::Either<TupleStructJsonSchemaBuilder, Nop<Self::MapKey, Self::Ok, Self::Error>>;
    type StructSchemaBuilder =
        either::Either<StructJsonSchemaBuilder, Nop<Self::MapKey, Self::Ok, Self::Error>>;
    type CombinatorSchemaBuilder = CombinatorJsonSchemaBuilder;
    type EnumSchemaBuilder =
        either::Either<EnumJsonSchemaBuilder, Nop<Self::MapKey, Self::Ok, Self::Error>>;
    type MapSchemaBuilder =
        either::Either<MapJsonSchemaBuilder, Nop<Self::MapKey, Self::Ok, Self::Error>>;

    type OptionSchemaBuilder = Self;
    type NewtypeStructSchemaBuilder = either::Either<
        PostProcessSchemaBuilder<SchemaCollectionTransform, Self>,
        Nop<Self::MapKey, Self::Ok, Self::Error>,
    >;
    type SeqSchemaBuilder = PostProcessSchemaBuilder<SeqSchemaTransform, Self>;
    type NotSchemaBuilder = PostProcessSchemaBuilder<NotSchemaTransform, Self>;

    fn describe_option<I: IntoIterator<IntoIter = E>>(
        mut self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::OptionSchemaBuilder, Self::Error> {
        let examples = examples()?
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::custom)?;
        // Keep the most outer description and examples
        self.description = self.description.or(description);
        self.examples = self.examples.or(Some(examples));
        self.deprecated |= deprecated;
        self.nullable = true;
        Ok(self)
    }

    fn describe_bool<I: IntoIterator<IntoIter = E>>(
        self,
        only: Option<bool>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        let mut result = schema! {};

        let examples = self.examples.unwrap_or(
            examples()?
                .into_iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()
                .map_err(Error::custom)?,
        );

        result.description = self.description.or(description).map(Into::into);

        if deprecated || self.deprecated {
            result.deprecated = Some(true);
        }

        match self.specification {
            Specification::OpenAPI3_0 => {
                result.example = examples.into_iter().next();

                result.r#type = Some("boolean".into());

                if self.nullable {
                    result.nullable = Some(true);

                    if let Some(only) = only {
                        result.r#enum = Some(vec![serde_json::Value::Bool(only), JsonValue::Null]);
                    }
                } else {
                    result.r#enum = only.map(|only| vec![serde_json::Value::Bool(only)]);
                }
            }
            Specification::OpenAPI3_1 => {
                result.examples = Some(specification::Examples::Vec(examples));

                if self.nullable {
                    result.r#type = Some(vec!["boolean".into(), "null".into()].into());
                } else {
                    result.r#type = Some("boolean".into());
                }

                // TODO: If self.nullable, does this have to be part of the enum spec in OAS3.1??
                result.r#enum = only.map(|only| vec![serde_json::Value::Bool(only)]);
            }
        }

        Ok(result.into())
    }

    fn describe_i8<I: IntoIterator<IntoIter = E>>(
        self,
        min: std::ops::Bound<i8>,
        max: std::ops::Bound<i8>,
        multiple_of: Option<i8>,
        format: Option<&'static str>,
        only: Option<&'static [i8]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_integer!(
            i8,
            self,
            min,
            max,
            multiple_of,
            format,
            only,
            description,
            self.examples.unwrap_or(
                examples()?
                    .into_iter()
                    .map(serde_json::to_value)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::custom)?
            ),
            deprecated,
        )
    }

    fn describe_i16<I: IntoIterator<IntoIter = E>>(
        self,
        min: std::ops::Bound<i16>,
        max: std::ops::Bound<i16>,
        multiple_of: Option<i16>, // TODO: Only positive values
        format: Option<&'static str>,
        only: Option<&'static [i16]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_integer!(
            i16,
            self,
            min,
            max,
            multiple_of,
            format,
            only,
            description,
            self.examples.unwrap_or(
                examples()?
                    .into_iter()
                    .map(serde_json::to_value)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::custom)?
            ),
            deprecated,
        )
    }

    fn describe_i32<I: IntoIterator<IntoIter = E>>(
        self,
        min: std::ops::Bound<i32>,
        max: std::ops::Bound<i32>,
        multiple_of: Option<i32>, // TODO: Only positive values
        format: Option<&'static str>,
        only: Option<&'static [i32]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_integer!(
            i32,
            self,
            min,
            max,
            multiple_of,
            format,
            only,
            description,
            self.examples.unwrap_or(
                examples()?
                    .into_iter()
                    .map(serde_json::to_value)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::custom)?
            ),
            deprecated,
        )
    }

    fn describe_i64<I: IntoIterator<IntoIter = E>>(
        self,
        min: std::ops::Bound<i64>,
        max: std::ops::Bound<i64>,
        multiple_of: Option<i64>, // TODO: Only positive values
        format: Option<&'static str>,
        only: Option<&'static [i64]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_integer!(
            i64,
            self,
            min,
            max,
            multiple_of,
            format,
            only,
            description,
            self.examples.unwrap_or(
                examples()?
                    .into_iter()
                    .map(serde_json::to_value)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::custom)?
            ),
            deprecated,
        )
    }

    fn describe_u8<I: IntoIterator<IntoIter = E>>(
        self,
        min: std::ops::Bound<u8>,
        max: std::ops::Bound<u8>,
        multiple_of: Option<u8>,
        format: Option<&'static str>,
        only: Option<&'static [u8]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_integer!(
            u8,
            self,
            min,
            max,
            multiple_of,
            format,
            only,
            description,
            self.examples.unwrap_or(
                examples()?
                    .into_iter()
                    .map(serde_json::to_value)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::custom)?
            ),
            deprecated,
        )
    }

    fn describe_u16<I: IntoIterator<IntoIter = E>>(
        self,
        min: std::ops::Bound<u16>,
        max: std::ops::Bound<u16>,
        multiple_of: Option<u16>,
        format: Option<&'static str>,
        only: Option<&'static [u16]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_integer!(
            u16,
            self,
            min,
            max,
            multiple_of,
            format,
            only,
            description,
            self.examples.unwrap_or(
                examples()?
                    .into_iter()
                    .map(serde_json::to_value)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::custom)?
            ),
            deprecated,
        )
    }

    fn describe_u32<I: IntoIterator<IntoIter = E>>(
        self,
        min: std::ops::Bound<u32>,
        max: std::ops::Bound<u32>,
        multiple_of: Option<u32>,
        format: Option<&'static str>,
        only: Option<&'static [u32]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_integer!(
            u32,
            self,
            min,
            max,
            multiple_of,
            format,
            only,
            description,
            self.examples.unwrap_or(
                examples()?
                    .into_iter()
                    .map(serde_json::to_value)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::custom)?
            ),
            deprecated,
        )
    }

    fn describe_u64<I: IntoIterator<IntoIter = E>>(
        self,
        min: std::ops::Bound<u64>,
        max: std::ops::Bound<u64>,
        multiple_of: Option<u64>,
        format: Option<&'static str>,
        only: Option<&'static [u64]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_integer!(
            u64,
            self,
            min,
            max,
            multiple_of,
            format,
            only,
            description,
            self.examples.unwrap_or(
                examples()?
                    .into_iter()
                    .map(serde_json::to_value)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::custom)?
            ),
            deprecated,
        )
    }

    fn describe_f32<I: IntoIterator<IntoIter = E>>(
        self,
        _allow_nan: bool, // TODO
        _allow_inf: bool, // TODO
        min: std::ops::Bound<f32>,
        max: std::ops::Bound<f32>,
        format: Option<&'static str>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_float!(
            f32,
            self,
            min,
            max,
            format,
            description,
            self.examples.unwrap_or(
                examples()?
                    .into_iter()
                    .map(serde_json::to_value)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::custom)?,
            ),
            deprecated
        )
    }

    fn describe_f64<I: IntoIterator<IntoIter = E>>(
        self,
        _allow_nan: bool, // TODO
        _allow_inf: bool, // TODO
        min: std::ops::Bound<f64>,
        max: std::ops::Bound<f64>,
        format: Option<&'static str>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        describe_float!(
            f64,
            self,
            min,
            max,
            format,
            description,
            self.examples.unwrap_or(
                examples()?
                    .into_iter()
                    .map(serde_json::to_value)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::custom)?,
            ),
            deprecated
        )
    }

    fn describe_char<I: IntoIterator<IntoIter = E>>(
        self,
        pattern: Option<&'static str>,
        format: Option<&'static str>,
        only: Option<&'static [char]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        let mut result = schema! {};

        let examples = self.examples.unwrap_or(
            examples()?
                .into_iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()
                .map_err(Error::custom)?,
        );

        result.description = self.description.or(description).map(Into::into);

        if deprecated || self.deprecated {
            result.deprecated = Some(true);
        }

        match self.specification {
            Specification::OpenAPI3_0 => {
                result.example = examples.into_iter().next();
                result.r#type = Some("string".into());

                if self.nullable {
                    result.nullable = Some(true);

                    if let Some(only) = only {
                        let mut r#enum = only
                            .iter()
                            .map(|value| serde_json::Value::from(&*value.encode_utf8(&mut [0; 4])))
                            .collect::<Vec<_>>();

                        r#enum.push(JsonValue::Null);

                        result.r#enum = Some(r#enum);
                    }
                } else {
                    result.r#enum = only.map(|only| {
                        only.iter()
                            .map(|value| serde_json::Value::from(&*value.encode_utf8(&mut [0; 4])))
                            .collect()
                    });
                }
            }
            Specification::OpenAPI3_1 => {
                result.examples = Some(specification::Examples::Vec(examples));

                if self.nullable {
                    result.r#type = Some(vec!["string".into(), "null".into()].into());
                } else {
                    result.r#type = Some("string".into());
                }

                // TODO: If self.nullable, does this have to be part of the enum spec in OAS3.1??
                result.r#enum = only.map(|only| {
                    only.iter()
                        .map(|value| serde_json::Value::from(&*value.encode_utf8(&mut [0; 4])))
                        .collect()
                });
            }
        }

        result.min_length = Some(serde_json::Number::from(1));
        result.max_length = Some(serde_json::Number::from(1));
        result.pattern = pattern.map(Into::into);
        result.format = format.map(Into::into);

        Ok(result.into())
    }

    fn describe_str<I: IntoIterator<IntoIter = E>>(
        self,
        min_len: Option<usize>,
        max_len: Option<usize>,
        pattern: Option<&'static str>,
        format: Option<&'static str>,
        only: Option<&'static [&'static str]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        let mut result = schema! {
            r#type: "string".into()
        };

        result.description = self.description.or(description).map(Into::into);

        if let Some(min_len) = min_len {
            result.min_length = Some(serde_json::Number::from(min_len));
        }

        if let Some(max_len) = max_len {
            result.max_length = Some(serde_json::Number::from(max_len));
        }

        result.pattern = pattern.map(Into::into);
        result.format = format.map(Into::into);
        result.r#enum = only.map(|only| {
            only.iter()
                .map(|entry| serde_json::Value::String((*entry).to_string()))
                .collect::<Vec<_>>()
        });

        result.deprecated = if deprecated { Some(true) } else { None };

        let examples = examples()?
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::custom)?;

        match self.specification {
            Specification::OpenAPI3_0 => {
                result.example = examples.into_iter().next();
            }
            Specification::OpenAPI3_1 => {
                result.examples = Some(specification::Examples::Vec(examples));
            }
        }

        Ok(result.into())
    }

    fn describe_bytes<I: IntoIterator<IntoIter = E>>(
        self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        let inner_schema_builder =
            self.describe_seq(None, None, false, description, examples, deprecated)?;

        <u8 as Schema>::describe(inner_schema_builder)
    }

    fn describe_unit<I: IntoIterator<IntoIter = E>>(
        self,
        description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        let mut result = null_schema(self.specification);

        result.description = self.description.or(description).map(Into::into);

        if deprecated || self.deprecated {
            result.deprecated = Some(true);
        }

        Ok(result.into())
    }

    fn describe_unit_struct<I: IntoIterator<IntoIter = E>>(
        self,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        _examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        if let Some(schema_collection) = &self.schema_collection {
            let schema_collection = schema_collection.borrow();
            if let Some(schema_id) = &id {
                match schema_collection.resolve_ref(schema_id) {
                    Ok(schema_ref) => {
                        return Ok(ReferenceObject {
                            r#ref: schema_ref,
                            summary: None,
                            description: None,
                        }
                        .into());
                    }
                    Err(SchemaCollectionResolutionError::ConflictingDefinition {
                        conflicting_callsite,
                        schema_id,
                    }) => {
                        return Err(Error::conflicting_definition(
                            schema_id,
                            conflicting_callsite,
                        ));
                    }
                    _ => {}
                }
            }
        }

        let mut result_schema = null_schema(self.specification);

        result_schema.description = self.description.or(description).map(Into::into);

        if deprecated || self.deprecated {
            result_schema.deprecated = Some(true);
        }

        if let Some(schema_collection) = self.schema_collection {
            let mut schema_collection = schema_collection.borrow_mut();
            if let Some(schema_id) = id {
                let schema_ref = schema_collection.set(&schema_id, result_schema.into());
                return Ok(ReferenceObject {
                    r#ref: schema_ref,
                    summary: None,
                    description: None,
                }
                .into());
            }
        }

        Ok(result_schema.into())
    }

    fn describe_newtype_struct<I: IntoIterator<IntoIter = E>>(
        mut self,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::NewtypeStructSchemaBuilder, Self::Error> {
        if let Some(schema_collection) = &self.schema_collection {
            let schema_collection = schema_collection.borrow();
            if let Some(schema_id) = &id {
                match schema_collection.resolve_ref(schema_id) {
                    Ok(schema_ref) => {
                        return Ok(either::Either::Right(Nop::new(
                            ReferenceObject {
                                r#ref: schema_ref,
                                summary: None,
                                description: None,
                            }
                            .into(),
                            true,
                        )));
                    }
                    Err(SchemaCollectionResolutionError::ConflictingDefinition {
                        conflicting_callsite,
                        schema_id,
                    }) => {
                        return Err(Error::conflicting_definition(
                            schema_id,
                            conflicting_callsite,
                        ));
                    }
                    _ => {}
                }
            }
        }

        let examples = examples()?
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::custom)?;
        // Keep the most outer description and examples
        self.description = self.description.or(description);
        self.examples = self.examples.or(Some(examples));
        self.deprecated |= deprecated;

        Ok(either::Either::Left(PostProcessSchemaBuilder::new(
            SchemaCollectionTransform::new(self.schema_collection.clone(), id),
            self,
        )))
    }

    fn describe_seq<I: IntoIterator<IntoIter = E>>(
        self,
        min_len: Option<usize>,
        max_len: Option<usize>,
        unique: bool,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::SeqSchemaBuilder, Self::Error> {
        Ok(PostProcessSchemaBuilder::new(
            SeqSchemaTransform::new(
                self.specification,
                self.description.or(description),
                self.examples.or(Some(
                    examples()?
                        .into_iter()
                        .map(serde_json::to_value)
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(Error::custom)?,
                )),
                deprecated || self.deprecated,
                min_len,
                max_len,
                unique,
                self.nullable,
            ),
            Self::new(self.specification, self.schema_collection),
        ))
    }

    fn describe_tuple<I: IntoIterator<IntoIter = E>>(
        self,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::TupleSchemaBuilder, Self::Error> {
        Ok(TupleJsonSchemaBuilder::new(
            self.specification,
            self.schema_collection,
            None,
            self.description.or(description),
            self.examples.or(Some(
                examples()?
                    .into_iter()
                    .map(serde_json::to_value)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::custom)?,
            )),
            deprecated || self.deprecated,
            self.nullable,
            len,
        ))
    }

    fn describe_tuple_struct<I: IntoIterator<IntoIter = E>>(
        self,
        id: Option<SchemaId>,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::TupleStructSchemaBuilder, Self::Error> {
        if let Some(schema_collection) = &self.schema_collection {
            let schema_collection = schema_collection.borrow();
            if let Some(schema_id) = &id {
                match schema_collection.resolve_ref(schema_id) {
                    Ok(schema_ref) => {
                        return Ok(either::Either::Right(Nop::new(
                            ReferenceObject {
                                r#ref: schema_ref,
                                summary: None,
                                description: None,
                            }
                            .into(),
                            true,
                        )));
                    }
                    Err(SchemaCollectionResolutionError::ConflictingDefinition {
                        conflicting_callsite,
                        schema_id,
                    }) => {
                        return Err(Error::conflicting_definition(
                            schema_id,
                            conflicting_callsite,
                        ));
                    }
                    _ => {}
                }
            }
        }

        Ok(either::Either::Left(TupleStructJsonSchemaBuilder::new(
            self.specification,
            self.schema_collection,
            id,
            self.description.or(description),
            self.examples.or(Some(
                examples()?
                    .into_iter()
                    .map(serde_json::to_value)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::custom)?,
            )),
            deprecated || self.deprecated,
            self.nullable,
            len,
        )))
    }

    fn describe_map<I: IntoIterator<IntoIter = E>>(
        self,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::MapSchemaBuilder, Self::Error> {
        if let Some(schema_collection) = &self.schema_collection {
            let schema_collection = schema_collection.borrow();
            if let Some(schema_id) = &id {
                match schema_collection.resolve_ref(schema_id) {
                    Ok(schema_ref) => {
                        return Ok(either::Either::Right(Nop::new(
                            ReferenceObject {
                                r#ref: schema_ref,
                                summary: None,
                                description: None,
                            }
                            .into(),
                            true,
                        )));
                    }
                    Err(SchemaCollectionResolutionError::ConflictingDefinition {
                        conflicting_callsite,
                        schema_id,
                    }) => {
                        return Err(Error::conflicting_definition(
                            schema_id,
                            conflicting_callsite,
                        ));
                    }
                    _ => {}
                }
            }
        }

        Ok(either::Either::Left(MapJsonSchemaBuilder::new(
            self.specification,
            self.schema_collection,
            id,
            description,
            self.examples.or(Some(
                examples()?
                    .into_iter()
                    .map(serde_json::to_value)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::custom)?,
            )),
            deprecated,
            self.nullable,
            None,
        )))
    }

    fn describe_struct<I: IntoIterator<IntoIter = E>>(
        self,
        id: Option<SchemaId>,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::StructSchemaBuilder, Self::Error> {
        if let Some(schema_collection) = &self.schema_collection {
            let schema_collection = schema_collection.borrow();
            if let Some(schema_id) = &id {
                match schema_collection.resolve_ref(schema_id) {
                    Ok(schema_ref) => {
                        return Ok(either::Either::Right(Nop::new(
                            ReferenceObject {
                                r#ref: schema_ref,
                                summary: None,
                                description: None,
                            }
                            .into(),
                            true,
                        )));
                    }
                    Err(SchemaCollectionResolutionError::ConflictingDefinition {
                        conflicting_callsite,
                        schema_id,
                    }) => {
                        return Err(Error::conflicting_definition(
                            schema_id,
                            conflicting_callsite,
                        ));
                    }
                    _ => {}
                }
            }
        }

        Ok(either::Either::Left(StructJsonSchemaBuilder::new(
            self.specification,
            self.schema_collection,
            id,
            self.description.or(description),
            self.examples.or(Some(
                examples()?
                    .into_iter()
                    .map(serde_json::to_value)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::custom)?,
            )),
            deprecated || self.deprecated,
            self.nullable,
            len,
        )))
    }

    fn describe_enum<I: IntoIterator<IntoIter = E>>(
        self,
        id: Option<SchemaId>,
        len: usize,
        exhaustive: bool,
        tag: VariantTag,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::EnumSchemaBuilder, Self::Error> {
        if let Some(schema_collection) = &self.schema_collection {
            let schema_collection = schema_collection.borrow();
            if let Some(schema_id) = &id {
                match schema_collection.resolve_ref(schema_id) {
                    Ok(schema_ref) => {
                        return Ok(either::Either::Right(Nop::new(
                            ReferenceObject {
                                r#ref: schema_ref,
                                summary: None,
                                description: None,
                            }
                            .into(),
                            true,
                        )));
                    }
                    Err(SchemaCollectionResolutionError::ConflictingDefinition {
                        conflicting_callsite,
                        schema_id,
                    }) => {
                        return Err(Error::conflicting_definition(
                            schema_id,
                            conflicting_callsite,
                        ));
                    }
                    _ => {}
                }
            }
        }

        Ok(either::Either::Left(EnumJsonSchemaBuilder::new(
            self.specification,
            self.schema_collection,
            id,
            self.description.or(description),
            self.examples.or(Some(
                examples()?
                    .into_iter()
                    .map(serde_json::to_value)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::custom)?,
            )),
            deprecated || self.deprecated,
            self.nullable,
            tag,
            len,
            exhaustive,
        )))
    }

    fn describe_not<I: IntoIterator<IntoIter = E>>(
        self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::NotSchemaBuilder, Self::Error> {
        Ok(PostProcessSchemaBuilder::new(
            NotSchemaTransform::new(
                self.specification,
                self.description.or(description),
                self.examples.or(Some(
                    examples()?
                        .into_iter()
                        .map(serde_json::to_value)
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(Error::custom)?,
                )),
                deprecated || self.deprecated,
                self.nullable,
            ),
            Self::new(self.specification, self.schema_collection),
        ))
    }

    fn describe_combinator<I: IntoIterator<IntoIter = E>>(
        self,
        combinator: Combinator,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::CombinatorSchemaBuilder, Self::Error> {
        Ok(CombinatorJsonSchemaBuilder::new(
            self.specification,
            self.schema_collection,
            self.description.or(description),
            self.examples.or(Some(
                examples()?
                    .into_iter()
                    .map(serde_json::to_value)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::custom)?,
            )),
            deprecated || self.deprecated,
            self.nullable,
            combinator,
            len,
        ))
    }
}
