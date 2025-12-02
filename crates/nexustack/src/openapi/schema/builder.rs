/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use std::fmt::Display;

use crate::{
    Callsite,
    openapi::{error::Error, schema::Schema},
};
use serde::Serialize;

//
// Struct
//

/// Indicates the access modifier for a struct field in a schema.
///
/// This enum is used to specify whether a field is readable, writable, or both.
/// It is used by schema builders to annotate fields for documentation and code generation.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldMod {
    /// The field is both readable and writable.
    #[default]
    ReadWrite,
    /// The field is only readable.
    Read,
    /// The field is only writable.
    Write,
}

/// Builder for describing the schema of a struct type.
///
/// This trait provides methods for describing and collecting fields of a struct,
/// including optional fields, skipping fields, and finalizing the schema.
///
/// For a usage example see the [`SchemaBuilder::describe_struct`] function.
pub trait StructSchemaBuilder {
    /// The type used for keys in the schema.
    type MapKey;

    /// The output type produced when the schema is finalized.
    type Ok;

    /// The error type for schema building.
    type Error: Error;

    /// Builder for describing a single field.
    type FieldSchemaBuilder<'a>: IntoSchemaBuilder<MapKey = Self::MapKey, Ok = (), Error = Self::Error>
    where
        Self: 'a;

    /// Describe a field in the struct schema.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_struct`] function.
    ///
    /// # Paramaters
    /// - `key` - The name of the field.
    /// - `modifier` - The field access modifier.
    /// - `description` - Optional description for the field.
    /// - `deprecated` - Whether the field is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_field<'a>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error>;

    /// Collect and describe a field using a closure.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_struct`] function.
    ///
    /// # Paramaters
    /// - `key` - The name of the field.
    /// - `modifier` - The field access modifier.
    /// - `description` - Optional description for the field.
    /// - `deprecated` - Whether the field is deprecated.
    /// - `describe` - Closure to describe the field schema.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn collect_field<'a, D, E: Iterator<Item: Serialize + 'static>>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(
            <Self::FieldSchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<E>,
        ) -> Result<(), Self::Error>,
    {
        describe(
            StructSchemaBuilder::describe_field(self, key, modifier, description, deprecated)?
                .into_schema_builder(),
        )
    }

    /// Describe an optional field in the struct schema.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_struct`] function.
    ///
    /// # Paramaters
    /// - `key` - The name of the field.
    /// - `modifier` - The field access modifier.
    /// - `default` - Optional default value for the field.
    /// - `description` - Optional description for the field.
    /// - `deprecated` - Whether the field is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_field_optional<'a, F: Serialize>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        default: Option<F>,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error>;

    /// Collect and describe an optional field using a closure.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_struct`] function.
    ///
    /// # Paramaters
    /// - `key` - The name of the field.
    /// - `modifier` - The field access modifier.
    /// - `default` - Optional default value for the field.
    /// - `description` - Optional description for the field.
    /// - `deprecated` - Whether the field is deprecated.
    /// - `describe` - Closure to describe the field schema.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn collect_field_optional<'a, D, E: Iterator<Item: Serialize + 'static>, F: Serialize>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        default: Option<F>,
        description: Option<&'static str>,
        deprecated: bool,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(
            <Self::FieldSchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<E>,
        ) -> Result<(), Self::Error>,
    {
        describe(
            StructSchemaBuilder::describe_field_optional(
                self,
                key,
                modifier,
                default,
                description,
                deprecated,
            )?
            .into_schema_builder(),
        )
    }

    /// Skip a field in the struct schema.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_struct`] function.
    ///
    /// # Paramaters
    /// - `key` - The name of the field to skip.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
        let _ = key;
        Ok(())
    }

    /// Finalize the struct schema and return the result.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_struct`] function.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

//
// Tuple
//

/// Builder for describing the schema of a tuple type.
///
/// For a usage example see the [`SchemaBuilder::describe_tuple`] function.
pub trait TupleSchemaBuilder {
    /// The type used for keys in the schema.
    type MapKey;

    /// The output type produced when the schema is finalized.
    type Ok;

    /// The error type for schema building.
    type Error: Error;

    /// Builder for describing a single tuple element.
    type ElementSchemaBuilder<'a>: IntoSchemaBuilder<MapKey = Self::MapKey, Ok = (), Error = Self::Error>
    where
        Self: 'a;

    /// Describe an element in the tuple schema.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_tuple`] function.
    ///
    /// # Paramaters
    /// - `description` - Optional description for the element.
    /// - `deprecated` - Whether the element is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_element<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::ElementSchemaBuilder<'a>, Self::Error>;

    /// Collect and describe an element using a closure.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_tuple`] function.
    ///
    /// # Paramaters
    /// - `description` - Optional description for the element.
    /// - `deprecated` - Whether the element is deprecated.
    /// - `describe` - Closure to describe the element schema.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn collect_element<'a, D, E: Iterator<Item: Serialize + 'static>>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(
            <Self::ElementSchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<E>,
        ) -> Result<(), Self::Error>,
    {
        describe(
            TupleSchemaBuilder::describe_element(self, description, deprecated)?
                .into_schema_builder(),
        )
    }

    /// Finalize the tuple schema and return the result.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_tuple`] function.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

//
// Tuple struct
//

/// Builder for describing the schema of a tuple struct type.
///
/// For a usage example see the [`SchemaBuilder::describe_tuple_struct`] function.
pub trait TupleStructSchemaBuilder {
    /// The type used for keys in the schema.
    type MapKey;

    /// The output type produced when the schema is finalized.
    type Ok;

    /// The error type for schema building.
    type Error: Error;

    /// Builder for describing a single field.
    type FieldSchemaBuilder<'a>: IntoSchemaBuilder<MapKey = Self::MapKey, Ok = (), Error = Self::Error>
    where
        Self: 'a;

    /// Describe a field in the tuple struct schema.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_tuple_struct`] function.
    ///
    /// # Paramaters
    /// - `description` - Optional description for the field.
    /// - `deprecated` - Whether the field is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_field<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error>;

    /// Collect and describe a field using a closure.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_tuple_struct`] function.
    ///
    /// # Paramaters
    /// - `description` - Optional description for the field.
    /// - `deprecated` - Whether the field is deprecated.
    /// - `describe` - Closure to describe the field schema.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn collect_field<'a, D, E: Iterator<Item: Serialize + 'static>>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(
            <Self::FieldSchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<E>,
        ) -> Result<(), Self::Error>,
    {
        describe(
            TupleStructSchemaBuilder::describe_field(self, description, deprecated)?
                .into_schema_builder(),
        )
    }

    /// Finalize the tuple struct schema and return the result.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_tuple_struct`] function.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

//
// Combinator
//

/// Indicates the combinator type for schema composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// The schema is one of several alternatives.
    ///
    /// The value must validate against exactly one of the subschemas.
    ///   This is used when a value should match only one alternative schema.
    ///   See [OpenAPI oneOf](https://spec.openapis.org/oas/v3.1.0#oneof).
    OneOf,

    /// The schema is the intersection of several alternatives.
    ///
    /// The value must validate against all of the subschemas (intersection).
    ///   See [OpenAPI allOf](https://spec.openapis.org/oas/v3.1.0#allof).
    AllOf,

    /// The schema is any of several alternatives.
    ///
    /// The value must validate against any (one or more) of the subschemas.
    ///   This is used when a value may match multiple alternative schemas simultaneously.
    ///   See [OpenAPI anyOf](https://spec.openapis.org/oas/v3.1.0#anyof).
    AnyOf,
}

/// Builder for describing combinator schemas (oneOf, allOf, anyOf).
///
/// For a usage example see the [`SchemaBuilder::describe_combinator`] function.
pub trait CombinatorSchemaBuilder {
    /// The type used for keys in the schema.
    type MapKey;

    /// The output type produced when the schema is finalized.
    type Ok;

    /// The error type for schema building.
    type Error: Error;

    /// Builder for describing a subschema.
    type SubSchemaBuilder<'a>: IntoSchemaBuilder<MapKey = Self::MapKey, Ok = (), Error = Self::Error>
    where
        Self: 'a;

    /// Describe a subschema in the combinator schema.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_combinator`] function.
    ///
    /// # Paramaters
    /// - `description` - Optional description for the subschema.
    /// - `deprecated` - Whether the subschema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_subschema<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::SubSchemaBuilder<'a>, Self::Error>;

    /// Collect and describe a subschema using a closure.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_combinator`] function.
    ///
    /// # Paramaters
    /// - `description` - Optional description for the subschema.
    /// - `deprecated` - Whether the subschema is deprecated.
    /// - `describe` - Closure to describe the subschema schema.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn collect_subschema<'a, D, E: Iterator<Item: Serialize + 'static>>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(
            <Self::SubSchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<E>,
        ) -> Result<(), Self::Error>,
    {
        describe(
            CombinatorSchemaBuilder::describe_subschema(self, description, deprecated)?
                .into_schema_builder(),
        )
    }

    /// Finalize the combinator schema and return the result.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_combinator`] function.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

//
// Map
//

/// Builder for describing the schema of a map type.
///
/// For a usage example see the [`SchemaBuilder::describe_map`] function.
pub trait MapSchemaBuilder {
    /// The type used for keys in the schema.
    type MapKey;

    /// The output type produced when the schema is finalized.
    type Ok;

    /// The error type for schema building.
    type Error: Error;

    /// Builder for describing map keys.
    type MapKeySchemaBuilder: IntoSchemaBuilder<MapKey = Self::MapKey, Ok = Self::MapKey, Error = Self::Error>;

    /// Builder for describing map values.
    type MapValueSchemaBuilder<'a>: IntoSchemaBuilder<MapKey = Self::MapKey, Ok = (), Error = Self::Error>
    where
        Self: 'a;

    /// Describe an element in the map schema.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_map`] function.
    ///
    /// # Paramaters
    /// - `key` - The key of the map element.
    /// - `modifier` - The field access modifier.
    /// - `description` - Optional description for the element.
    /// - `deprecated` - Whether the element is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_element<'a, K: Schema + Serialize>(
        &'a mut self,
        key: K,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'a>, Self::Error>;

    /// Collect and describe an element using a closure.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_map`] function.
    ///
    /// # Paramaters
    /// - `key` - The key of the map element.
    /// - `modifier` - The field access modifier.
    /// - `description` - Optional description for the element.
    /// - `deprecated` - Whether the element is deprecated.
    /// - `describe_value` - Closure to describe the element schema.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn collect_element<'a, J: Iterator<Item: Serialize + 'static>, K: Schema + Serialize, V>(
        &'a mut self,
        key: K,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
        describe_value: V,
    ) -> Result<(), Self::Error>
    where
        V: FnOnce(
            <Self::MapValueSchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<J>,
        ) -> Result<(), Self::Error>,
    {
        describe_value(
            MapSchemaBuilder::describe_element(self, key, modifier, description, deprecated)?
                .into_schema_builder(),
        )
    }

    /// Describe an optional element in the map schema.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_map`] function.
    ///
    /// # Paramaters
    /// - `key` - The key of the map element.
    /// - `modifier` - The field access modifier.
    /// - `default` - Optional default value for the element.
    /// - `description` - Optional description for the element.
    /// - `deprecated` - Whether the element is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_element_optional<'a, K: Schema + Serialize, F: Serialize>(
        &'a mut self,
        key: K,
        modifier: FieldMod,
        default: Option<F>,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::MapValueSchemaBuilder<'a>, Self::Error>;

    /// Collect and describe an optional element using a closure.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_map`] function.
    ///
    /// # Paramaters
    /// - `key` - The key of the map element.
    /// - `modifier` - The field access modifier.
    /// - `default` - Optional default value for the element.
    /// - `description` - Optional description for the element.
    /// - `deprecated` - Whether the element is deprecated.
    /// - `describe_value` - Closure to describe the element schema.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn collect_element_optional<
        'a,
        J: Iterator<Item: Serialize + 'static>,
        K: Schema + Serialize,
        V,
        F: Serialize,
    >(
        &'a mut self,
        key: K,
        modifier: FieldMod,
        default: Option<F>,
        description: Option<&'static str>,
        deprecated: bool,
        describe_value: V,
    ) -> Result<(), Self::Error>
    where
        V: FnOnce(
            <Self::MapValueSchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<J>,
        ) -> Result<(), Self::Error>,
    {
        describe_value(
            MapSchemaBuilder::describe_element_optional(
                self,
                key,
                modifier,
                default,
                description,
                deprecated,
            )?
            .into_schema_builder(),
        )
    }

    /// Describe additional elements in the map schema.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_map`] function.
    ///
    /// # Paramaters
    /// - `describe_key` - Closure to describe the keys of additional elements.
    /// - `description` - Optional description for the additional elements.
    /// - `deprecated` - Whether the additional elements are deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
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
            -> Result<<Self::MapKeySchemaBuilder as IntoSchemaBuilder>::Ok, Self::Error>;

    /// Collect and describe additional elements using closures.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_map`] function.
    ///
    /// # Paramaters
    /// - `describe_key` - Closure to describe the keys of additional elements.
    /// - `description` - Optional description for the additional elements.
    /// - `deprecated` - Whether the additional elements are deprecated.
    /// - `describe_value` - Closure to describe the values of additional elements.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn collect_additional_elements<
        'a,
        I: Iterator<Item: Serialize + 'static>,
        J: Iterator<Item: Serialize + 'static>,
        K,
        V,
    >(
        &'a mut self,
        describe_key: K,
        description: Option<&'static str>,
        deprecated: bool,
        describe_value: V,
    ) -> Result<(), Self::Error>
    where
        K: FnOnce(
            <Self::MapKeySchemaBuilder as IntoSchemaBuilder>::SchemaBuilder<I>,
        )
            -> Result<<Self::MapKeySchemaBuilder as IntoSchemaBuilder>::Ok, Self::Error>,
        V: FnOnce(
            <Self::MapValueSchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<J>,
        ) -> Result<(), Self::Error>,
    {
        describe_value(
            MapSchemaBuilder::describe_additional_elements(
                self,
                describe_key,
                description,
                deprecated,
            )?
            .into_schema_builder(),
        )
    }

    /// Skip an element in the map schema.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_map`] function.
    ///
    /// # Paramaters
    /// - `key` - The key of the element to skip.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn skip_element<K: Schema + Serialize>(&mut self, key: K) -> Result<(), Self::Error> {
        let _ = key;
        Ok(())
    }

    /// Finalize the map schema and return the result.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_map`] function.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

//
// Enum
//

/// Indicates the tagging strategy for enum variants in a schema.
///
/// See [Serde enum representations](https://serde.rs/enum-representations.html) for more details.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantTag {
    /// No tag is used; variants are distinguished by their structure.
    Untagged,
    /// Tag is external to the variant.
    #[default]
    ExternallyTagged,
    /// Tag is internal to the variant.
    InternallyTagged {
        /// The name of the tag field used to distinguish variants.
        tag: &'static str,
    },
    /// Tag and content are adjacent.
    AdjacentlyTagged {
        /// The name of the tag field used to distinguish variants.
        tag: &'static str,
        /// The name of the content field holding the variant's data.
        content: &'static str,
    },
}

/// Builder for describing the schema of a struct variant in an enum.
///
/// For a usage example see the [`SchemaBuilder::describe_enum`] function.
pub trait StructVariantSchemaBuilder {
    /// The type used for keys in the schema.
    type MapKey;
    /// The error type for schema building.
    type Error: Error;

    /// Builder for describing a single field.
    type FieldSchemaBuilder<'a>: IntoSchemaBuilder<MapKey = Self::MapKey, Ok = (), Error = Self::Error>
    where
        Self: 'a;

    /// Describe a field in the struct variant schema.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_enum`] function.
    ///
    /// # Paramaters
    /// - `key` - The name of the field.
    /// - `modifier` - The field access modifier.
    /// - `description` - Optional description for the field.
    /// - `deprecated` - Whether the field is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_field<'a>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error>;

    /// Collect and describe a field using a closure.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_enum`] function.
    ///
    /// # Paramaters
    /// - `key` - The name of the field.
    /// - `modifier` - The field access modifier.
    /// - `description` - Optional description for the field.
    /// - `deprecated` - Whether the field is deprecated.
    /// - `describe` - Closure to describe the field schema.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn collect_field<'a, D, E: Iterator<Item: Serialize + 'static>>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        description: Option<&'static str>,
        deprecated: bool,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(
            <Self::FieldSchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<E>,
        ) -> Result<(), Self::Error>,
    {
        describe(
            StructVariantSchemaBuilder::describe_field(
                self,
                key,
                modifier,
                description,
                deprecated,
            )?
            .into_schema_builder(),
        )
    }

    /// Describe an optional field in the struct variant schema.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_enum`] function.
    ///
    /// # Paramaters
    /// - `key` - The name of the field.
    /// - `modifier` - The field access modifier.
    /// - `default` - Optional default value for the field.
    /// - `description` - Optional description for the field.
    /// - `deprecated` - Whether the field is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_field_optional<'a, F: Serialize>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        default: Option<F>,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error>;

    /// Collect and describe an optional field using a closure.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_enum`] function.
    ///
    /// # Paramaters
    /// - `key` - The name of the field.
    /// - `modifier` - The field access modifier.
    /// - `default` - Optional default value for the field.
    /// - `description` - Optional description for the field.
    /// - `deprecated` - Whether the field is deprecated.
    /// - `describe` - Closure to describe the field schema.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn collect_field_optional<'a, D, E: Iterator<Item: Serialize + 'static>, F: Serialize>(
        &'a mut self,
        key: &'static str,
        modifier: FieldMod,
        default: Option<F>,
        description: Option<&'static str>,
        deprecated: bool,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(
            <Self::FieldSchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<E>,
        ) -> Result<(), Self::Error>,
    {
        describe(
            StructVariantSchemaBuilder::describe_field_optional(
                self,
                key,
                modifier,
                default,
                description,
                deprecated,
            )?
            .into_schema_builder(),
        )
    }

    /// Skip a field in the struct variant schema.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_enum`] function.
    ///
    /// # Paramaters
    /// - `key` - The name of the field to skip.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
        let _ = key;
        Ok(())
    }

    /// Finalize the struct variant schema and return the result.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_enum`] function.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn end(self) -> Result<(), Self::Error>;
}

/// Builder for describing the schema of a tuple variant in an enum.
///
/// For a usage example see the [`SchemaBuilder::describe_enum`] function.
pub trait TupleVariantSchemaBuilder {
    /// The type used for keys in the schema.
    type MapKey;
    /// The error type for schema building.
    type Error: Error;

    /// Builder for describing a single field.
    type FieldSchemaBuilder<'a>: IntoSchemaBuilder<MapKey = Self::MapKey, Ok = (), Error = Self::Error>
    where
        Self: 'a;

    /// Describe a field in the tuple variant schema.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_enum`] function.
    ///
    /// # Paramaters
    /// - `description` - Optional description for the field.
    /// - `deprecated` - Whether the field is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_field<'a>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::FieldSchemaBuilder<'a>, Self::Error>;

    /// Collect and describe a field using a closure.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_enum`] function.
    ///
    /// # Paramaters
    /// - `description` - Optional description for the field.
    /// - `deprecated` - Whether the field is deprecated.
    /// - `describe` - Closure to describe the field schema.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn collect_field<'a, D, E: Iterator<Item: Serialize + 'static>>(
        &'a mut self,
        description: Option<&'static str>,
        deprecated: bool,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(
            <Self::FieldSchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<E>,
        ) -> Result<(), Self::Error>,
    {
        describe(
            TupleVariantSchemaBuilder::describe_field(self, description, deprecated)?
                .into_schema_builder(),
        )
    }

    /// Finalize the tuple variant schema and return the result.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_enum`] function.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn end(self) -> Result<(), Self::Error>;
}

/// Builder for describing the schema of an enum type.
///
/// For a usage example see the [`SchemaBuilder::describe_enum`] function.
pub trait EnumSchemaBuilder {
    /// The type used for keys in the schema.
    type MapKey;

    /// The output type produced when the schema is finalized.
    type Ok;

    /// The error type for schema building.
    type Error: Error;

    /// Builder for tuple variants.
    type TupleVariantSchemaBuilder<'a>: TupleVariantSchemaBuilder<MapKey = Self::MapKey, Error = Self::Error>
    where
        Self: 'a;

    /// Builder for struct variants.
    type StructVariantSchemaBuilder<'a>: StructVariantSchemaBuilder<MapKey = Self::MapKey, Error = Self::Error>
    where
        Self: 'a;

    /// Builder for newtype variants.
    type NewTypeVariantSchemaBuilder<'a>: IntoSchemaBuilder<MapKey = Self::MapKey, Ok = (), Error = Self::Error>
    where
        Self: 'a;

    /// Describe a unit variant in the enum schema.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_enum`] function.
    ///
    /// # Paramaters
    /// - `index` - The index of the variant.
    /// - `id` - The schema identifier.
    /// - `description` - Optional description for the variant.
    /// - `deprecated` - Whether the variant is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_unit_variant(
        &mut self,
        index: u32,
        id: SchemaId,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<(), Self::Error>;

    /// Describe a newtype variant in the enum schema.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_enum`] function.
    ///
    /// # Paramaters
    /// - `index` - The index of the variant.
    /// - `id` - The schema identifier.
    /// - `description` - Optional description for the variant.
    /// - `deprecated` - Whether the variant is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_newtype_variant<'a>(
        &'a mut self,
        index: u32,
        id: SchemaId,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::NewTypeVariantSchemaBuilder<'a>, Self::Error>;

    /// Collect and describe a newtype variant using a closure.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_enum`] function.
    ///
    /// # Paramaters
    /// - `index` - The index of the variant.
    /// - `id` - The schema identifier.
    /// - `description` - Optional description for the variant.
    /// - `deprecated` - Whether the variant is deprecated.
    /// - `describe` - Closure to describe the variant schema.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn collect_newtype_variant<'a, D, J: Iterator<Item: Serialize + 'static>>(
        &'a mut self,
        index: u32,
        id: SchemaId,
        description: Option<&'static str>,
        deprecated: bool,
        describe: D,
    ) -> Result<(), Self::Error>
    where
        D: FnOnce(
            <Self::NewTypeVariantSchemaBuilder<'a> as IntoSchemaBuilder>::SchemaBuilder<J>,
        ) -> Result<(), Self::Error>,
    {
        describe(
            EnumSchemaBuilder::describe_newtype_variant(self, index, id, description, deprecated)?
                .into_schema_builder(),
        )
    }

    /// Describe a tuple variant in the enum schema.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_enum`] function.
    ///
    /// # Paramaters
    /// - `index` - The index of the variant.
    /// - `id` - The schema identifier.
    /// - `len` - The number of elements in the tuple.
    /// - `description` - Optional description for the variant.
    /// - `deprecated` - Whether the variant is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_tuple_variant<'a>(
        &'a mut self,
        index: u32,
        id: SchemaId,
        len: usize,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::TupleVariantSchemaBuilder<'a>, Self::Error>;

    /// Describe a struct variant in the enum schema.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_enum`] function.
    ///
    /// # Paramaters
    /// - `index` - The index of the variant.
    /// - `id` - The schema identifier.
    /// - `len` - The number of fields in the struct.
    /// - `description` - Optional description for the variant.
    /// - `deprecated` - Whether the variant is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_struct_variant<'a>(
        &'a mut self,
        index: u32,
        id: SchemaId,
        len: usize,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::StructVariantSchemaBuilder<'a>, Self::Error>;

    /// Finalize the enum schema and return the result.
    ///
    /// For a usage example see the [`SchemaBuilder::describe_enum`] function.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

//
// Schema
//

/// Identifier for a schema, including its name and callsite.
///
/// This struct is used to uniquely identify a schema definition within the `OpenAPI` schema builder.
/// It contains the name of the schema and the callsite information, which helps with tracking
/// where the schema was defined in the codebase. This is useful for documentation, debugging,
/// and ensuring schema uniqueness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaId {
    /// The name of the schema.
    name: &'static str,

    /// The callsite information.
    callsite: Callsite,
}

impl SchemaId {
    /// Create a new schema identifier.
    ///
    /// # Paramaters
    /// - `name` - The name of the schema.
    /// - `callsite` - The callsite information.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaId;
    /// use nexustack::callsite;
    ///
    /// callsite!(MyTypeCallsite);
    ///
    /// let id = SchemaId::new("MyType", *MyTypeCallsite);
    /// ```
    #[must_use]
    pub const fn new(name: &'static str, callsite: Callsite) -> Self {
        Self { name, callsite }
    }

    /// The name of the schema.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// The callsite information.
    #[must_use]
    pub const fn callsite(&self) -> &Callsite {
        &self.callsite
    }
}

impl Display for SchemaId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ {}", self.name, self.callsite)
    }
}

/// Trait for converting into a schema builder for a specific example iterator type.
pub trait IntoSchemaBuilder: Sized {
    /// The type used for keys in the schema.
    type MapKey;

    /// The output type produced when the schema is finalized.
    type Ok;

    /// The error type for schema building.
    type Error: Error;

    /// The schema builder for the iterator type.
    type SchemaBuilder<E: Iterator<Item: Serialize + 'static>>: SchemaBuilder<E, MapKey = Self::MapKey, Ok = Self::Ok, Error = Self::Error>;

    /// Convert the builder into a schema builder for a specific iterator type.
    fn into_schema_builder<E: Iterator<Item: Serialize + 'static>>(self) -> Self::SchemaBuilder<E>;
}

/// The main trait for building schemas for types.
///
/// This trait provides methods for describing all supported Rust types,
/// including primitives, options, sequences, tuples, structs, enums, maps,
/// and combinators.
///
pub trait SchemaBuilder<E: Iterator<Item: Serialize + 'static>>: Sized {
    /// The type used for keys in the schema.
    type MapKey;

    /// The output type produced when the schema is finalized.
    type Ok;

    /// The error type for schema building.
    type Error: Error;

    /// Builder for tuple schemas.
    type TupleSchemaBuilder: TupleSchemaBuilder<MapKey = Self::MapKey, Ok = Self::Ok, Error = Self::Error>;

    /// Builder for tuple struct schemas.
    type TupleStructSchemaBuilder: TupleStructSchemaBuilder<MapKey = Self::MapKey, Ok = Self::Ok, Error = Self::Error>;

    /// Builder for struct schemas.
    type StructSchemaBuilder: StructSchemaBuilder<MapKey = Self::MapKey, Ok = Self::Ok, Error = Self::Error>;

    /// Builder for combinator schemas.
    type CombinatorSchemaBuilder: CombinatorSchemaBuilder<MapKey = Self::MapKey, Ok = Self::Ok, Error = Self::Error>;

    /// Builder for enum schemas.
    type EnumSchemaBuilder: EnumSchemaBuilder<MapKey = Self::MapKey, Ok = Self::Ok, Error = Self::Error>;

    /// Builder for map schemas.
    type MapSchemaBuilder: MapSchemaBuilder<MapKey = Self::MapKey, Ok = Self::Ok, Error = Self::Error>;

    /// Builder for optional schemas.
    type OptionSchemaBuilder: IntoSchemaBuilder<MapKey = Self::MapKey, Ok = Self::Ok, Error = Self::Error>;

    /// Builder for newtype struct schemas.
    type NewtypeStructSchemaBuilder: IntoSchemaBuilder<MapKey = Self::MapKey, Ok = Self::Ok, Error = Self::Error>;

    /// Builder for sequence schemas.
    type SeqSchemaBuilder: IntoSchemaBuilder<MapKey = Self::MapKey, Ok = Self::Ok, Error = Self::Error>;

    /// Builder for "not" schemas.
    type NotSchemaBuilder: IntoSchemaBuilder<MapKey = Self::MapKey, Ok = Self::Ok, Error = Self::Error>;

    /// Describe an optional schema.
    ///
    /// # Paramaters
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use crate::nexustack::openapi::IntoSchemaBuilder;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = Option<bool>;
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         let option_schema_builder = schema_builder.describe_option(
    ///             None,
    ///             || Ok([Some(true), Some(false), None]),
    ///             false
    ///         )?;
    ///         
    ///         <bool as Schema>::describe(option_schema_builder.into_schema_builder())
    ///     }
    /// }
    ///
    /// ```
    fn describe_option<I: IntoIterator<IntoIter = E>>(
        self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::OptionSchemaBuilder, Self::Error>;

    // TODO: collect_option

    /// Describe a boolean schema.
    ///
    /// # Paramaters
    /// - `only` - Optional fixed value for the boolean.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = bool;
    ///     type Examples = <[Self::Example; 2] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.describe_bool(None, None, || Ok([true, false]), false)
    ///     }
    /// }
    ///
    /// ```
    fn describe_bool<I: IntoIterator<IntoIter = E>>(
        self,
        only: Option<bool>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error>;

    /// Describe an 8-bit signed integer schema.
    ///
    /// # Paramaters
    /// - `min` - Minimum value constraint.
    /// - `max` - Maximum value constraint.
    /// - `multiple_of` - Constraint for multiple values.
    /// - `format` - Optional format string as defined by <https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-7.3>.
    ///   For possible values see <https://https://spec.openapis.org/registry/format/>.
    /// - `only` - Optional fixed set of values.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use std::ops::Bound;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = i8;
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.describe_i8(
    ///             Bound::Unbounded,
    ///             Bound::Unbounded,
    ///             None,
    ///             None,
    ///             Some(&[-6i8, -4i8, -2i8, -1i8, 0i8, 1i8]),
    ///             None,
    ///             || Ok([-1i8, 0i8, 1i8]),
    ///             false
    ///         )
    ///     }
    /// }
    ///
    /// ```
    #[allow(clippy::too_many_arguments)]
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
    ) -> Result<Self::Ok, Self::Error>;

    /// Describe an 16-bit signed integer schema.
    ///
    /// # Paramaters
    /// - `min` - Minimum value constraint.
    /// - `max` - Maximum value constraint.
    /// - `multiple_of` - Constraint for multiple values.
    /// - `format` - Optional format string as defined by <https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-7.3>.
    ///   For possible values see <https://https://spec.openapis.org/registry/format/>.
    /// - `only` - Optional fixed set of values.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use std::ops::Bound;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = i16;
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.describe_i16(
    ///             Bound::Unbounded,
    ///             Bound::Unbounded,
    ///             None,
    ///             None,
    ///             Some(&[-6i16, -4i16, -2i16, -1i16, 0i16, 1i16]),
    ///             None,
    ///             || Ok([-1i16, 0i16, 1i16]),
    ///             false
    ///         )
    ///     }
    /// }
    ///
    /// ```
    #[allow(clippy::too_many_arguments)]
    fn describe_i16<I: IntoIterator<IntoIter = E>>(
        self,
        min: std::ops::Bound<i16>,
        max: std::ops::Bound<i16>,
        multiple_of: Option<i16>,
        format: Option<&'static str>,
        only: Option<&'static [i16]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error>;

    /// Describe an 32-bit signed integer schema.
    ///
    /// # Paramaters
    /// - `min` - Minimum value constraint.
    /// - `max` - Maximum value constraint.
    /// - `multiple_of` - Constraint for multiple values.
    /// - `format` - Optional format string as defined by <https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-7.3>.
    ///   For possible values see <https://https://spec.openapis.org/registry/format/>.
    /// - `only` - Optional fixed set of values.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use std::ops::Bound;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = i32;
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.describe_i32(
    ///             Bound::Unbounded,
    ///             Bound::Unbounded,
    ///             None,
    ///             None,
    ///             Some(&[-6i32, -4i32, -2i32, -1i32, 0i32, 1i32]),
    ///             None,
    ///             || Ok([-1i32, 0i32, 1i32]),
    ///             false
    ///         )
    ///     }
    /// }
    ///
    /// ```
    #[allow(clippy::too_many_arguments)]
    fn describe_i32<I: IntoIterator<IntoIter = E>>(
        self,
        min: std::ops::Bound<i32>,
        max: std::ops::Bound<i32>,
        multiple_of: Option<i32>,
        format: Option<&'static str>,
        only: Option<&'static [i32]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error>;

    /// Describe an i64-bit signed integer schema.
    ///
    /// # Paramaters
    /// - `min` - Minimum value constraint.
    /// - `max` - Maximum value constraint.
    /// - `multiple_of` - Constraint for multiple values.
    /// - `format` - Optional format string as defined by <https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-7.3>.
    ///   For possible values see <https://https://spec.openapis.org/registry/format/>.
    /// - `only` - Optional fixed set of values.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use std::ops::Bound;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = i64;
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.describe_i64(
    ///             Bound::Unbounded,
    ///             Bound::Unbounded,
    ///             None,
    ///             None,
    ///             Some(&[-6i64, -4i64, -2i64, -1i64, 0i64, 1i64]),
    ///             None,
    ///             || Ok([-1i64, 0i64, 1i64]),
    ///             false
    ///         )
    ///     }
    /// }
    ///
    /// ```
    #[allow(clippy::too_many_arguments)]
    fn describe_i64<I: IntoIterator<IntoIter = E>>(
        self,
        min: std::ops::Bound<i64>,
        max: std::ops::Bound<i64>,
        multiple_of: Option<i64>,
        format: Option<&'static str>,
        only: Option<&'static [i64]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error>;

    /// Describe an 128-bit signed integer schema.
    ///
    /// # Paramaters
    /// - `min` - Minimum value constraint.
    /// - `max` - Maximum value constraint.
    /// - `multiple_of` - Constraint for multiple values.
    /// - `format` - Optional format string as defined by <https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-7.3>.
    ///   For possible values see <https://https://spec.openapis.org/registry/format/>.
    /// - `only` - Optional fixed set of values.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use std::ops::Bound;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = i128;
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.describe_i128(
    ///             Bound::Unbounded,
    ///             Bound::Unbounded,
    ///             None,
    ///             None,
    ///             Some(&[-6i128, -4i128, -2i128, -1i128, 0i128, 1i128]),
    ///             None,
    ///             || Ok([-1i128, 0i128, 1i128]),
    ///             false
    ///         )
    ///     }
    /// }
    ///
    /// ```
    #[allow(clippy::too_many_arguments)]
    fn describe_i128<I: IntoIterator<IntoIter = E>>(
        self,
        min: std::ops::Bound<i128>,
        max: std::ops::Bound<i128>,
        multiple_of: Option<i128>,
        format: Option<&'static str>,
        only: Option<&'static [i128]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        let _ = min;
        let _ = max;
        let _ = multiple_of;
        let _ = format;
        let _ = only;
        let _ = description;
        let _ = examples;
        let _ = deprecated;
        Err(Error::custom("i128 is not supported"))
    }

    /// Describe an 8-bit unsigned integer schema.
    ///
    /// # Paramaters
    /// - `min` - Minimum value constraint.
    /// - `max` - Maximum value constraint.
    /// - `multiple_of` - Constraint for multiple values.
    /// - `format` - Optional format string as defined by <https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-7.3>.
    ///   For possible values see <https://https://spec.openapis.org/registry/format/>.
    /// - `only` - Optional fixed set of values.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use std::ops::Bound;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = u8;
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.describe_u8(
    ///             Bound::Included(6u8),
    ///             Bound::Excluded(200u8),
    ///             Some(2u8),
    ///             None,
    ///             None,
    ///             None,
    ///             || Ok([6u8, 8u8, 198u8]),
    ///             false
    ///         )
    ///     }
    /// }
    ///
    /// ```
    #[allow(clippy::too_many_arguments)]
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
    ) -> Result<Self::Ok, Self::Error>;

    /// Describe an 16-bit unsigned integer schema.
    ///
    /// # Paramaters
    /// - `min` - Minimum value constraint.
    /// - `max` - Maximum value constraint.
    /// - `multiple_of` - Constraint for multiple values.
    /// - `format` - Optional format string as defined by <https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-7.3>.
    ///   For possible values see <https://https://spec.openapis.org/registry/format/>.
    /// - `only` - Optional fixed set of values.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use std::ops::Bound;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = u16;
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.describe_u16(
    ///             Bound::Included(6u16),
    ///             Bound::Excluded(200u16),
    ///             Some(2u16),
    ///             None,
    ///             None,
    ///             None,
    ///             || Ok([6u16, 8u16, 198u16]),
    ///             false
    ///         )
    ///     }
    /// }
    ///
    /// ```
    #[allow(clippy::too_many_arguments)]
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
    ) -> Result<Self::Ok, Self::Error>;

    /// Describe an 32-bit unsigned integer schema.
    ///
    /// # Paramaters
    /// - `min` - Minimum value constraint.
    /// - `max` - Maximum value constraint.
    /// - `multiple_of` - Constraint for multiple values.
    /// - `format` - Optional format string as defined by <https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-7.3>.
    ///   For possible values see <https://https://spec.openapis.org/registry/format/>.
    /// - `only` - Optional fixed set of values.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use std::ops::Bound;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = u32;
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.describe_u32(
    ///             Bound::Included(6u32),
    ///             Bound::Excluded(200u32),
    ///             Some(2u32),
    ///             None,
    ///             None,
    ///             None,
    ///             || Ok([6u32, 8u32, 198u32]),
    ///             false
    ///         )
    ///     }
    /// }
    ///
    /// ```
    #[allow(clippy::too_many_arguments)]
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
    ) -> Result<Self::Ok, Self::Error>;

    /// Describe an 64-bit unsigned integer schema.
    ///
    /// # Paramaters
    /// - `min` - Minimum value constraint.
    /// - `max` - Maximum value constraint.
    /// - `multiple_of` - Constraint for multiple values.
    /// - `format` - Optional format string as defined by <https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-7.3>.
    ///   For possible values see <https://https://spec.openapis.org/registry/format/>.
    /// - `only` - Optional fixed set of values.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use std::ops::Bound;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = u64;
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.describe_u64(
    ///             Bound::Included(6u64),
    ///             Bound::Excluded(200u64),
    ///             Some(2u64),
    ///             None,
    ///             None,
    ///             None,
    ///             || Ok([6u64, 8u64, 198u64]),
    ///             false
    ///         )
    ///     }
    /// }
    ///
    /// ```
    #[allow(clippy::too_many_arguments)]
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
    ) -> Result<Self::Ok, Self::Error>;

    /// Describe an 128-bit unsigned integer schema.
    ///
    /// # Paramaters
    /// - `min` - Minimum value constraint.
    /// - `max` - Maximum value constraint.
    /// - `multiple_of` - Constraint for multiple values.
    /// - `format` - Optional format string as defined by <https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-7.3>.
    ///   For possible values see <https://https://spec.openapis.org/registry/format/>.
    /// - `only` - Optional fixed set of values.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use std::ops::Bound;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = u128;
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.describe_u128(
    ///             Bound::Included(6u128),
    ///             Bound::Excluded(200u128),
    ///             Some(2u128),
    ///             None,
    ///             None,
    ///             None,
    ///             || Ok([6u128, 8u128, 198u128]),
    ///             false
    ///         )
    ///     }
    /// }
    ///
    /// ```
    #[allow(clippy::too_many_arguments)]
    fn describe_u128<I: IntoIterator<IntoIter = E>>(
        self,
        min: std::ops::Bound<u128>,
        max: std::ops::Bound<u128>,
        multiple_of: Option<u128>,
        format: Option<&'static str>,
        only: Option<&'static [u128]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error> {
        let _ = min;
        let _ = max;
        let _ = multiple_of;
        let _ = format;
        let _ = only;
        let _ = description;
        let _ = examples;
        let _ = deprecated;
        Err(Error::custom("u128 is not supported"))
    }

    /// Describe a 32-bit floating-point schema.
    ///
    /// # Paramaters
    /// - `allow_nan` - Whether NaN values are allowed.
    /// - `allow_inf` - Whether infinite values are allowed.
    /// - `min` - Minimum value constraint.
    /// - `max` - Maximum value constraint.
    /// - `format` - Optional format string as defined by <https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-7.3>.
    ///   For possible values see <https://https://spec.openapis.org/registry/format/>.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use std::ops::Bound;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = f32;
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.describe_f32(
    ///             false,
    ///             false,
    ///             Bound::Included(-5f32),
    ///             Bound::Excluded(5f32),
    ///             None,
    ///             None,
    ///             || Ok([-5f32, 0f32, 1f32]),
    ///             false
    ///         )
    ///     }
    /// }
    ///
    /// ```
    #[allow(clippy::too_many_arguments)]
    fn describe_f32<I: IntoIterator<IntoIter = E>>(
        self,
        allow_nan: bool,
        allow_inf: bool,
        min: std::ops::Bound<f32>,
        max: std::ops::Bound<f32>,
        format: Option<&'static str>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error>;

    /// Describe a f64-bit floating-point schema.
    ///
    /// # Paramaters
    /// - `allow_nan` - Whether NaN values are allowed.
    /// - `allow_inf` - Whether infinite values are allowed.
    /// - `min` - Minimum value constraint.
    /// - `max` - Maximum value constraint.
    /// - `format` - Optional format string as defined by <https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-7.3>.
    ///   For possible values see <https://https://spec.openapis.org/registry/format/>.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use std::ops::Bound;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = f64;
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.describe_f64(
    ///             false,
    ///             false,
    ///             Bound::Included(-5f64),
    ///             Bound::Excluded(5f64),
    ///             None,
    ///             None,
    ///             || Ok([-5f64, 0f64, 1f64]),
    ///             false
    ///         )
    ///     }
    /// }
    ///
    /// ```
    #[allow(clippy::too_many_arguments)]
    fn describe_f64<I: IntoIterator<IntoIter = E>>(
        self,
        allow_nan: bool,
        allow_inf: bool,
        min: std::ops::Bound<f64>,
        max: std::ops::Bound<f64>,
        format: Option<&'static str>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error>;

    /// Describe a character schema.
    ///
    /// # Paramaters
    /// - `pattern` - Optional regex pattern for the character.
    /// - `format` - Optional format string as defined by <https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-7.3>.
    ///   For possible values see <https://https://spec.openapis.org/registry/format/>.
    /// - `only` - Optional fixed set of characters.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use std::ops::Bound;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = char;
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.describe_char(
    ///             Some("^[a-zA-Z]$"),
    ///             None,
    ///             None,
    ///             None,
    ///             || Ok(['a', 'b', 'Z']),
    ///             false
    ///         )
    ///     }
    /// }
    ///
    /// ```
    fn describe_char<I: IntoIterator<IntoIter = E>>(
        self,
        pattern: Option<&'static str>,
        format: Option<&'static str>,
        only: Option<&'static [char]>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error>;

    /// Describe a string schema.
    ///
    /// # Paramaters
    /// - `min_len` - Minimum length constraint.
    /// - `max_len` - Maximum length constraint.
    /// - `pattern` - Optional regex pattern for the string.
    /// - `format` - Optional format string as defined by <https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-7.3>.
    ///   For possible values see <https://https://spec.openapis.org/registry/format/>.
    /// - `only` - Optional fixed set of strings.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use std::ops::Bound;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = &'static str;
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.describe_str(
    ///             Some(1usize),
    ///             Some(10usize),
    ///             Some("^[a-zA-Z]*$"),
    ///             None,
    ///             None,
    ///             None,
    ///             || Ok(["a", "bbbbbbbbbb", "ZZ"]),
    ///             false
    ///         )
    ///     }
    /// }
    ///
    /// ```
    #[allow(clippy::too_many_arguments)]
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
    ) -> Result<Self::Ok, Self::Error>;

    /// Describe a bytes schema.
    ///
    /// # Paramaters
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use std::ops::Bound;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = &'static [u8];
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.describe_bytes(
    ///             None,
    ///             || Ok([&b"a"[..], &b"b"[..], &b"0123456789"[..]]),
    ///             false
    ///         )
    ///     }
    /// }
    ///
    /// ```
    fn describe_bytes<I: IntoIterator<IntoIter = E>>(
        self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error>;

    /// Describe a unit schema.
    ///
    /// # Paramaters
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use std::ops::Bound;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = ();
    ///     type Examples = <[Self::Example; 1] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.describe_unit(None, || Ok([()]), false)
    ///     }
    /// }
    ///
    /// ```
    fn describe_unit<I: IntoIterator<IntoIter = E>>(
        self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error>;

    /// Describe a unit struct schema.
    ///
    /// # Paramaters
    /// - `id` - Optional schema identifier.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use nexustack::openapi::SchemaId;
    /// use nexustack::callsite;
    ///
    ///
    /// struct MyType;
    ///
    /// callsite!(MyTypeCallsite);
    ///
    /// impl Schema for MyType {
    ///     type Example = ();
    ///     type Examples = <[Self::Example; 1] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.describe_unit_struct(
    ///             Some(SchemaId::new("MyType", *MyTypeCallsite)),
    ///             None,
    ///             || Ok([()]),
    ///             false
    ///         )
    ///     }
    /// }
    ///
    /// ```
    fn describe_unit_struct<I: IntoIterator<IntoIter = E>>(
        self,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::Ok, Self::Error>;

    /// Describe a newtype struct schema.
    ///
    /// # Paramaters
    /// - `id` - Optional schema identifier.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use nexustack::openapi::SchemaId;
    /// use nexustack::openapi::IntoSchemaBuilder;
    /// use nexustack::callsite;
    /// use std::ops::Bound;
    ///
    /// struct MyType;
    ///
    /// callsite!(MyTypeCallsite);
    ///
    /// impl Schema for MyType {
    ///     type Example = bool;
    ///     type Examples = <[Self::Example; 2] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         let newtype_schema_builder = schema_builder.describe_newtype_struct(
    ///             Some(SchemaId::new("MyType", *MyTypeCallsite)),
    ///             None,
    ///             || Ok([true, false]),
    ///             false
    ///         )?;
    ///
    ///         <bool as Schema>::describe(newtype_schema_builder.into_schema_builder())
    ///     }
    /// }
    ///
    /// ```
    fn describe_newtype_struct<I: IntoIterator<IntoIter = E>>(
        self,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::NewtypeStructSchemaBuilder, Self::Error>;

    /// Collect and describe a newtype struct using a closure.
    ///
    /// # Paramaters
    /// - `id` - Optional schema identifier.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    /// - `describe` - Closure to describe the underlying schema.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use nexustack::openapi::SchemaId;
    /// use nexustack::callsite;
    ///
    /// struct MyType;
    ///
    /// callsite!(MyTypeCallsite);
    ///
    /// impl Schema for MyType {
    ///     type Example = bool;
    ///     type Examples = <[Self::Example; 2] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         schema_builder.collect_newtype_struct(
    ///             Some(SchemaId::new("MyType", *MyTypeCallsite)),
    ///             None,
    ///             || Ok([true, false]),
    ///             false,
    ///             <bool as Schema>::describe
    ///         )
    ///     }
    /// }
    ///
    /// ```
    fn collect_newtype_struct<I, D, J>(
        self,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
        describe: D,
    ) -> Result<Self::Ok, Self::Error>
    where
        I: IntoIterator<IntoIter = E>,
        D: FnOnce(
            <Self::NewtypeStructSchemaBuilder as IntoSchemaBuilder>::SchemaBuilder<J>,
        ) -> Result<Self::Ok, Self::Error>,
        J: Iterator<Item: Serialize + 'static>,
    {
        describe(
            SchemaBuilder::describe_newtype_struct(self, id, description, examples, deprecated)?
                .into_schema_builder(),
        )
    }

    /// Describe a sequence schema.
    ///
    /// # Paramaters
    /// - `min_len` - Minimum length constraint.
    /// - `max_len` - Maximum length constraint.
    /// - `unique` - Whether all items must be unique.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use nexustack::openapi::IntoSchemaBuilder;
    /// use std::ops::Bound;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = Vec<i32>;
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         let seq_schema_builder = schema_builder.describe_seq(
    ///             Some(1usize),
    ///             None,
    ///             false,
    ///             None,
    ///             || Ok([
    ///                 vec![1i32],
    ///                 vec![1i32, 2i32],
    ///                 vec![1i32, 2i32, 3i32]
    ///             ]),
    ///             false
    ///         )?;
    ///
    ///         <i32 as Schema>::describe(seq_schema_builder.into_schema_builder())
    ///     }
    /// }
    ///
    /// ```
    fn describe_seq<I: IntoIterator<IntoIter = E>>(
        self,
        min_len: Option<usize>,
        max_len: Option<usize>,
        unique: bool,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::SeqSchemaBuilder, Self::Error>;

    // TODO: collect_seq

    /// Describe a tuple schema.
    ///
    /// # Paramaters
    /// - `len` - The number of elements in the tuple.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_tuple<I: IntoIterator<IntoIter = E>>(
        // TODO: Example in doc
        self,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::TupleSchemaBuilder, Self::Error>;

    /// Describe a tuple struct schema.
    ///
    /// # Paramaters
    /// - `id` - Optional schema identifier.
    /// - `len` - The number of fields in the struct.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_tuple_struct<I: IntoIterator<IntoIter = E>>(
        // TODO: Example in doc
        self,
        id: Option<SchemaId>,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::TupleStructSchemaBuilder, Self::Error>;

    /// Describe a map schema.
    ///
    /// # Paramaters
    /// - `id` - Optional schema identifier.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_map<I: IntoIterator<IntoIter = E>>(
        // TODO: Example in doc
        self,
        id: Option<SchemaId>,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::MapSchemaBuilder, Self::Error>;

    /// Describe a struct schema.
    ///
    /// # Paramaters
    /// - `id` - Optional schema identifier.
    /// - `len` - The number of fields in the struct.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use nexustack::openapi::StructSchemaBuilder;
    /// use nexustack::openapi::FieldMod;
    /// use nexustack::openapi::IntoSchemaBuilder;
    /// use nexustack::openapi::SchemaId;
    /// use nexustack::callsite;
    ///
    /// #[derive(serde::Serialize)]
    /// struct MyType {
    ///     a: u8,
    ///     b: u16,
    ///     c: u32,
    ///     d: u64,
    /// }
    ///
    /// callsite!(MyTypeCallsite);
    ///
    /// impl Schema for MyType {
    ///     type Example = MyType;
    ///     type Examples = <[Self::Example; 1] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         let mut struct_schema_builder = schema_builder.describe_struct(
    ///             Some(SchemaId::new("MyType", *MyTypeCallsite)),
    ///             5usize,
    ///             Some("My custom struct description"),
    ///             || Ok([
    ///                 MyType {
    ///                     a: 0u8,
    ///                     b: 1u16,
    ///                     c: 2u32,
    ///                     d: 3u64,
    ///                 }
    ///             ]),
    ///             false
    ///         )?;
    ///
    ///         let field_a_schema_builder = struct_schema_builder.describe_field(
    ///             "a",
    ///             FieldMod::ReadWrite,
    ///             Some("Field a"),
    ///             false
    ///         )?;
    ///
    ///         <u8 as Schema>::describe(field_a_schema_builder.into_schema_builder())?;
    ///         
    ///         struct_schema_builder.collect_field(
    ///             "b",
    ///             FieldMod::ReadWrite,
    ///             Some("Field b"),
    ///             false,
    ///             <u16 as Schema>::describe
    ///         )?;
    ///
    ///         let field_c_schema_builder = struct_schema_builder.describe_field_optional(
    ///             "c",
    ///             FieldMod::ReadWrite,
    ///             Some(0u32),
    ///             Some("Field c"),
    ///             false
    ///         )?;
    ///
    ///         <u32 as Schema>::describe(field_c_schema_builder.into_schema_builder())?;
    ///         
    ///         struct_schema_builder.collect_field_optional(
    ///             "d",
    ///             FieldMod::ReadWrite,
    ///             Some(0u64),
    ///             Some("Field d"),
    ///             false,
    ///             <u64 as Schema>::describe
    ///         )?;
    ///
    ///         struct_schema_builder.skip_field("e")?;
    ///
    ///         struct_schema_builder.end()
    ///     }
    /// }
    ///
    /// ```
    fn describe_struct<I: IntoIterator<IntoIter = E>>(
        self,
        id: Option<SchemaId>,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::StructSchemaBuilder, Self::Error>;

    /// Describe an enum schema.
    ///
    /// # Paramaters
    /// - `id` - Optional schema identifier.
    /// - `len` - The number of variants in the enum.
    /// - `exhaustive` - Whether all possible variants are included.
    /// - `tag` - The tagging strategy for the variants.
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    #[allow(clippy::too_many_arguments)]
    fn describe_enum<I: IntoIterator<IntoIter = E>>(
        // TODO: Example in doc
        self,
        id: Option<SchemaId>,
        len: usize,
        exhaustive: bool,
        tag: VariantTag,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::EnumSchemaBuilder, Self::Error>;

    /// Describe a "not" schema.
    ///
    /// # Paramaters
    /// - `description` - Optional description for the schema.
    /// - `examples` - Function providing example values.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use nexustack::openapi::IntoSchemaBuilder;
    ///
    /// struct MyType;
    ///
    /// impl Schema for MyType {
    ///     type Example = i32;
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         let seq_schema_builder = schema_builder.describe_not(
    ///             None,
    ///             || Ok([1i32, 2i32, 3i32]),
    ///             false
    ///         )?;
    ///
    ///         <&str as Schema>::describe(seq_schema_builder.into_schema_builder())
    ///     }
    /// }
    ///
    /// ```
    fn describe_not<I: IntoIterator<IntoIter = E>>(
        self,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::NotSchemaBuilder, Self::Error>;

    // TODO: collect_not

    /// Describe a combinator schema (oneOf, allOf, anyOf).
    ///
    /// This method allows you to specify a schema that is composed using a combinator type.
    /// The combinator can be one of `Combinator::OneOf`, `Combinator::AllOf`, or `Combinator::AnyOf`.
    /// It is typically used when a value may conform to one, all, or any of several alternative schemas.
    ///
    /// # Paramaters
    /// - `combinator` - The combinator type (`OneOf`, `AllOf`, or `AnyOf`).
    /// - `len` - The number of alternative schemas.
    /// - `description` - Optional description for the combinator schema.
    /// - `examples` - Function providing example values for the schema.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_combinator<I: IntoIterator<IntoIter = E>>(
        // TODO: Example in doc
        self,
        combinator: Combinator,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::CombinatorSchemaBuilder, Self::Error>;

    /// Describe an "allOf" combinator schema.
    ///
    /// This method allows you to specify that the schema is the intersection of several alternatives.
    /// It is typically used when a value must conform to all schemas from a set.
    ///
    /// # Paramaters
    /// - `len` - The number of alternative schemas.
    /// - `description` - Optional description for the combinator schema.
    /// - `examples` - Function providing example values for the schema.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_all_of<I: IntoIterator<IntoIter = E>>(
        // TODO: Example in doc
        self,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::CombinatorSchemaBuilder, Self::Error> {
        self.describe_combinator(Combinator::AllOf, len, description, examples, deprecated)
    }

    /// Describe an "anyOf" combinator schema.
    ///
    /// This method allows you to specify that the schema may conform to any of several alternatives.
    /// It is typically used when a value may match one or more schemas from a set.
    ///
    /// # Paramaters
    /// - `len` - The number of alternative schemas.
    /// - `description` - Optional description for the combinator schema.
    /// - `examples` - Function providing example values for the schema.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_any_of<I: IntoIterator<IntoIter = E>>(
        // TODO: Example in doc
        self,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::CombinatorSchemaBuilder, Self::Error> {
        self.describe_combinator(Combinator::AnyOf, len, description, examples, deprecated)
    }

    /// Describe a "oneOf" combinator schema.
    ///
    /// This method allows you to specify that the schema is one of several alternatives.
    /// It is typically used when a value may conform to one of multiple schemas.
    ///
    /// # Paramaters
    /// - `len` - The number of alternative schemas.
    /// - `description` - Optional description for the combinator schema.
    /// - `examples` - Function providing example values for the schema.
    /// - `deprecated` - Whether the schema is deprecated.
    ///
    /// # Errors
    ///
    /// Returns an error if schema construction fails, for example due to:
    /// - Invalid type information or unsupported types.
    /// - Serialization errors when generating example values.
    /// - Builder-specific errors encountered during schema description.
    fn describe_one_of<I: IntoIterator<IntoIter = E>>(
        // TODO: Example in doc
        self,
        len: usize,
        description: Option<&'static str>,
        examples: impl Fn() -> Result<I, Self::Error>,
        deprecated: bool,
    ) -> Result<Self::CombinatorSchemaBuilder, Self::Error> {
        self.describe_combinator(Combinator::OneOf, len, description, examples, deprecated)
    }

    /// Determine whether schemas and examples should be produced in human-readable form.
    ///
    /// Some types have a human-readable form that may be somewhat expensive to construct,
    /// as well as a binary form that is compact and efficient. Generally, text-based formats
    /// like JSON and YAML will prefer to use the human-readable one, and binary formats like
    /// Postcard will prefer the compact one.
    ///
    /// This method is used by [`SchemaBuilder`] and [`SchemaExamples`] to decide how to produce
    /// example values and schema representations. See [`serde::Serializer::is_human_readable`]
    /// for more details.
    ///
    /// The default implementation returns `true`. Implementors may override this to request
    /// a compact form for types that support one.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexustack::openapi::SchemaBuilder;
    /// use nexustack::openapi::Schema;
    /// use std::ops::Bound;
    ///
    /// struct MyType(u8);
    ///
    /// impl serde::Serialize for MyType {
    ///     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    ///         where
    ///             S: serde::Serializer
    ///     {
    ///         if (serializer.is_human_readable()) {
    ///             serializer.serialize_u8(self.0)
    ///         }  else {
    ///             serializer.serialize_str(format!("0x{value:x}", value = self.0).as_str())
    ///         }
    ///     }
    /// }
    ///
    /// impl Schema for MyType {
    ///     type Example = MyType;
    ///     type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;
    ///
    ///     #[inline]
    ///     fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    ///     where
    ///         B: SchemaBuilder<Self::Examples>,
    ///     {
    ///         let examples = || Ok([Self(0u8), Self(18u8), Self(255u8)]);
    ///
    ///         if (schema_builder.is_human_readable()) {
    ///             schema_builder.describe_str(
    ///                 None,
    ///                 None,
    ///                 Some("^0x[0-9a-fA-F]{2}$"),
    ///                 None,
    ///                 None,
    ///                 None,
    ///                 examples,
    ///                 false
    ///             )
    ///         } else {
    ///             schema_builder.describe_u8(
    ///                 Bound::Unbounded,
    ///                 Bound::Unbounded,
    ///                 None,
    ///                 None,
    ///                 None,
    ///                 None,
    ///                 examples,
    ///                 false
    ///             )
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// [`SchemaBuilder`]: crate::openapi::schema_builder::SchemaBuilder
    /// [`SchemaExamples`]: crate::openapi::example::SchemaExamples
    /// [`serde::Serializer::is_human_readable`]: https://docs.rs/serde/latest/serde/ser/trait.Serializer.html#method.is_human_readable
    fn is_human_readable(&self) -> bool {
        true
    }
}
