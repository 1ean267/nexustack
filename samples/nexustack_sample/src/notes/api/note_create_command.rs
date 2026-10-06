/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use nexustack::openapi::api_schema;

/// Command to create a new note.
#[derive(Debug)]
#[api_schema(read)]
pub struct NoteCreateCommand {
    /// The title of the note.
    pub title: String,
    /// The content of the note.
    pub content: String,
    /// Whether the note is published.
    pub is_published: bool,
}
