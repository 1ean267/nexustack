/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use nexustack::openapi::api_schema;

/// Represents a note.
#[derive(Debug)]
#[api_schema(write, rename = "Note")]
pub struct NoteViewModel {
    /// The unique identifier of the note.
    pub id: u32,

    /// The title of the note.
    pub title: String,

    /// The content of the note.
    pub content: String,

    /// Whether the note is published.
    pub is_published: bool,

    /// The creation timestamp of the note.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// The last updated timestamp of the note.
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::notes::note::Note> for NoteViewModel {
    fn from(note: crate::notes::note::Note) -> Self {
        Self {
            id: note.id,
            title: note.title,
            content: note.content,
            is_published: note.is_published,
            created_at: note.created_at,
            updated_at: note.updated_at,
        }
    }
}
