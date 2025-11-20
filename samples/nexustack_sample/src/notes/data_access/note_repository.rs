/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{notes::note::Note, response::DataAccessError};
use nexustack::inject::injectable;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct NoteRepository(Arc<Vec<Note>>);

#[injectable]
impl NoteRepository {
    pub fn new() -> Self {
        NoteRepository(Arc::new(vec![
            Note {
                id: 1,
                title: "First Note".to_string(),
                content: "This is the content of the first note.".to_string(),
                is_published: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Note {
                id: 2,
                title: "Second Note".to_string(),
                content: "This is the content of the second note.".to_string(),
                is_published: false,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ]))
    }

    pub async fn get_note_by_id(&self, note_id: u32) -> Result<Option<Note>, DataAccessError> {
        Ok(self.0.iter().find(|note| note.id == note_id).cloned())
    }

    pub async fn get_all_notes(&self) -> Result<Vec<Note>, DataAccessError> {
        Ok(self.0.as_ref().clone())
    }
}
