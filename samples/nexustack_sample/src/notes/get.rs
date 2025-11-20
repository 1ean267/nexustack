/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    notes::{
        api::{NoteCreateCommand, NoteViewModel},
        data_access::NoteRepository,
    },
    response::{
        GetManyHttpError, GetManyHttpResponse, GetOneHttpError, GetOneHttpResponse,
        HttpCreateResponse, HttpOperationError,
    },
};
use nexustack::http::decoding::JsonDecoder;
use nexustack::{http::http_controller, openapi::api_schema};

/// Represents the fields by which notes can be sorted.
#[api_schema(read)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteSortBy {
    /// Sort by the unique identifier of the note.
    Id,
    /// Sort by the title of the note.
    Title,
    /// Sort by the content of the note.
    Content,
    /// Sort by the publication status of the note.
    IsPublished,
    /// Sort by the creation timestamp of the note.
    CreatedAt,
    /// Sort by the last updated timestamp of the note.
    UpdatedAt,
}

#[derive(Debug, Clone)]
pub struct NoteController {
    note_repository: NoteRepository,
}

/// HTTP controller for managing notes.
#[http_controller(tags = "Notes")]
impl NoteController {
    #[http_controller::ctor]
    const fn new(note_repository: NoteRepository) -> Self {
        Self { note_repository }
    }

    /// Retrieves a single note using its unique identifier.
    ///
    /// # Parameters
    /// - `id`: The unique identifier of the note to retrieve.
    #[get(route = "/api/notes/{note_id}")]
    pub async fn get_one(
        &self,
        #[param(rename = "note_id")] id: u32,
    ) -> Result<GetOneHttpResponse<NoteViewModel>, GetOneHttpError> {
        let note = self
            .note_repository
            .get_note_by_id(id)
            .await?
            .ok_or(GetOneHttpError::NotFound)?;

        Ok(GetOneHttpResponse(NoteViewModel::from(note)))
    }

    /// Retrieves all notes.
    ///
    /// # Parameters
    /// - `sort_by`: The field by which to sort the notes.
    #[get(route = "/api/notes")]
    pub async fn get_many(
        &self,
        #[query(default)] sort_by: Option<NoteSortBy>,
    ) -> Result<GetManyHttpResponse<NoteViewModel>, GetManyHttpError> {
        let notes = self.note_repository.get_all_notes().await?;
        let mut view_models = notes
            .into_iter()
            .map(NoteViewModel::from)
            .collect::<Vec<_>>();

        if let Some(sort_by) = sort_by {
            view_models.sort_by(|a, b| match sort_by {
                NoteSortBy::Id => a.id.cmp(&b.id),
                NoteSortBy::Title => a.title.cmp(&b.title),
                NoteSortBy::Content => a.content.cmp(&b.content),
                NoteSortBy::IsPublished => a.is_published.cmp(&b.is_published),
                NoteSortBy::CreatedAt => a.created_at.cmp(&b.created_at),
                NoteSortBy::UpdatedAt => a.updated_at.cmp(&b.updated_at),
            });
        }

        Ok(GetManyHttpResponse(view_models))
    }

    /// Creates a new note.
    /// # Parameters
    /// - `command`: The command containing the details of the note to create.
    #[post(route = "/api/notes")]
    pub async fn create(
        &self,
        #[body(decoder = "JsonDecoder")] command: NoteCreateCommand,
    ) -> Result<HttpCreateResponse<NoteViewModel>, HttpOperationError> {
        todo!()
    }
}
