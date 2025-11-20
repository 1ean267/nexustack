/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    Callsite,
    openapi::{schema::builder::SchemaId, spec::SchemaOrReferenceObject},
};
use std::{borrow::Cow, collections::HashMap};

/// Errors that can occur during schema resolution in a [`SchemaCollection`].
///
/// This error type is returned by [`SchemaCollection::resolve_ref`] when a schema cannot be found
/// or when there is a conflicting definition for a schema.
#[derive(Debug, thiserror::Error)]
pub enum SchemaCollectionResolutionError {
    /// The requested schema was not found in the collection.
    #[error("Schema not found: {schema_id:?}")]
    NotFound {
        /// The [`SchemaId`] that was not found.
        schema_id: SchemaId,
    },
    /// There is a conflicting definition for the schema in the collection.
    #[error("Conflicting definition for schema {schema_id:?} at callsite {conflicting_callsite:?}")]
    ConflictingDefinition {
        /// The [`SchemaId`] for which the conflict occurred.
        schema_id: SchemaId,
        /// The [`Callsite`] where the conflicting definition was found.
        conflicting_callsite: Callsite,
    },
}

/// A collection for storing and resolving `OpenAPI` schemas by name.
///
/// This struct manages a set of schemas, allowing you to add schemas, resolve references,
/// and convert the collection into an `OpenAPI` schemas object.
pub struct SchemaCollection {
    /// The map of schema names to their schema object and callsite.
    entries: HashMap<&'static str, (SchemaOrReferenceObject, Callsite)>,
    /// The base path used for schema references.
    base_path: &'static str,
}

impl SchemaCollection {
    /// Creates a new [`SchemaCollection`] with the default base path.
    ///
    /// # Returns
    ///
    /// A new `SchemaCollection` with base path set to `#/components/schemas`.
    #[must_use]
    pub fn new() -> Self {
        Self::with_base_path("#/components/schemas")
    }

    /// Creates a new [`SchemaCollection`] with a custom base path.
    ///
    /// # Paramaters
    /// - `base_path` - The base path to use for schema references.
    ///
    /// # Returns
    ///
    /// A new `SchemaCollection` with the specified base path.
    #[must_use]
    pub fn with_base_path(base_path: &'static str) -> Self {
        Self {
            base_path,
            entries: HashMap::new(),
        }
    }

    /// Resolves a reference to a schema by its [`SchemaId`].
    ///
    /// # Paramaters
    /// - `schema_id` - The identifier of the schema to resolve.
    ///
    /// # Returns
    ///
    /// - `Ok(String)` - The reference string for the schema.
    /// - `Err(SchemaCollectionResolutionError)` - If the schema is not found or there is a conflict.
    ///
    /// # Errors
    ///
    /// - [`SchemaCollectionResolutionError`] if the schema is not present in the collection or if a schema with the same name but a different callsite exists.
    pub fn resolve_ref(
        &self,
        schema_id: &SchemaId,
    ) -> Result<String, SchemaCollectionResolutionError> {
        let entry = self.entries.get(schema_id.name());

        if let Some(entry) = entry {
            let (_, callsite) = entry;

            if callsite == schema_id.callsite() {
                let base_path = self.base_path;
                let name = schema_id.name();
                return Ok(format!("{base_path}/{name}"));
            }

            return Err(SchemaCollectionResolutionError::ConflictingDefinition {
                schema_id: schema_id.clone(),
                conflicting_callsite: *callsite,
            });
        }

        Err(SchemaCollectionResolutionError::NotFound {
            schema_id: schema_id.clone(),
        })
    }

    /// Adds a schema to the collection.
    ///
    /// # Paramaters
    /// - `schema_id` - The identifier for the schema.
    /// - `schema` - The schema object to add.
    ///
    /// # Returns
    ///
    /// The reference string for the added schema.
    pub fn set(&mut self, schema_id: &SchemaId, schema: SchemaOrReferenceObject) -> String {
        self.entries
            .insert(schema_id.name(), (schema, *schema_id.callsite()));

        let base_path = self.base_path;
        let name = schema_id.name();

        format!("{base_path}/{name}")
    }

    /// Converts the collection into an `OpenAPI` schemas object.
    ///
    /// # Returns
    ///
    /// A `HashMap` mapping schema names to their schema objects.
    #[must_use]
    pub fn to_schemas_object(self) -> HashMap<Cow<'static, str>, SchemaOrReferenceObject> {
        let mut result = HashMap::with_capacity(self.entries.len());

        for (name, (schema, _)) in self.entries {
            result.insert(Cow::Borrowed(name), schema);
        }

        result
    }
}

impl Default for SchemaCollection {
    fn default() -> Self {
        Self::new()
    }
}
