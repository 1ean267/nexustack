/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    notes::{api::NoteViewModel, data_access::NoteRepository},
    response::{GetManyHttpError, GetManyHttpResponse, GetOneHttpError, GetOneHttpResponse},
};
use axum::{
    body::Body,
    extract::{FromRequest, Request},
};
use nexustack::{
    callsite,
    http::HttpEndpoint,
    inject::{self, injectable},
    openapi::{self, HttpOperationBuilder, HttpOperationId},
};

#[derive(Debug, Clone)]
#[injectable]
pub struct NoteController {
    note_repository: NoteRepository,
}

// #[controller(tags = "Notes")]
impl NoteController {
    /// Retrieves a single note using its unique identifier.
    ///
    /// # Parameters
    /// - `note_id`: The unique identifier of the note to retrieve.
    // #[get("/api/notes/{id}")]
    pub async fn get_one(
        &self,
        // #[path("id")]
        note_id: u32,
    ) -> Result<GetOneHttpResponse<NoteViewModel>, GetOneHttpError> {
        let note = self.note_repository.get_note_by_id(note_id).await?;

        if let Some(note) = note {
            Ok(GetOneHttpResponse(NoteViewModel::from(note)))
        } else {
            Err(GetOneHttpError::NotFound)
        }
    }

    /// Retrieves all notes.
    // #[get("/api/notes")]
    pub async fn get_many(&self) -> Result<GetManyHttpResponse<NoteViewModel>, GetManyHttpError> {
        let notes = self.note_repository.get_all_notes().await?;
        let view_models = notes.into_iter().map(NoteViewModel::from).collect();

        Ok(GetManyHttpResponse(view_models))
    }
}

const _: () = {
    struct GetOneEndpoint(NoteController);

    impl inject::FromInjector for GetOneEndpoint {
        fn from_injector(injector: &inject::Injector) -> inject::ConstructionResult<Self>
        where
            Self: Sized,
        {
            let service_provider = injector.resolve::<inject::ServiceProvider>()?;
            let controller = service_provider.construct()?;
            Ok(Self(controller))
        }
    }

    struct GetManyEndpoint(NoteController);

    impl inject::FromInjector for GetManyEndpoint {
        fn from_injector(injector: &inject::Injector) -> inject::ConstructionResult<Self>
        where
            Self: Sized,
        {
            let service_provider = injector.resolve::<inject::ServiceProvider>()?;
            let controller = service_provider.construct()?;
            Ok(Self(controller))
        }
    }

    impl nexustack::http::HttpController for NoteController {
        fn build_endpoints<B>(mut builder: B)
        where
            B: nexustack::http::HttpEndpointsBuilder,
        {
            builder.add_endpoint::<GetOneEndpoint>();
            builder.add_endpoint::<GetManyEndpoint>();
        }
    }

    impl HttpEndpoint for GetOneEndpoint {
        type Request = Request<Body>;
        type Response = Result<GetOneHttpResponse<NoteViewModel>, GetOneHttpError>;
        type Routes = [&'static str; 1];

        fn method() -> nexustack::http::HttpMethod {
            nexustack::http::HttpMethod::Get
        }

        fn routes() -> Self::Routes {
            ["/api/notes/{id}"]
        }

        async fn handle(&mut self, request: Self::Request) -> Self::Response {
            let axum::extract::Path((note_id,)) =
                axum::extract::Path::from_request(request, &()).await?;

            self.0.get_one(note_id).await
        }
    }

    callsite!(GetOneEndpointCallsite);

    impl openapi::HttpOperation for GetOneEndpoint {
        fn describe<B>(mut operation_builder: B) -> Result<B::Ok, B::Error>
        where
            B: HttpOperationBuilder,
        {
            operation_builder.collect_path_parameter(
                "id",
                Some("The unique identifier of the note"),
                false,
                <u32 as openapi::Schema>::describe,
            )?;

            operation_builder.collect_operation(
                HttpOperationId::new("NoteController.get_one", *GetOneEndpointCallsite),
                "GET",
                "/api/notes/{id}",
                Some(["Notes"]),
                Some("Retrieves a single note using its unique identifier."),
                false,
                <Result<GetOneHttpResponse<NoteViewModel>, GetOneHttpError> as openapi::HttpResponse>::describe
            )
        }
    }

    impl HttpEndpoint for GetManyEndpoint {
        type Request = Request<Body>;
        type Response = Result<GetManyHttpResponse<NoteViewModel>, GetManyHttpError>;
        type Routes = [&'static str; 1];

        fn method() -> nexustack::http::HttpMethod {
            nexustack::http::HttpMethod::Get
        }

        fn routes() -> Self::Routes {
            ["/api/notes"]
        }

        async fn handle(&mut self, request: Self::Request) -> Self::Response {
            self.0.get_many().await
        }
    }

    callsite!(GetManyEndpointCallsite);

    impl openapi::HttpOperation for GetManyEndpoint {
        fn describe<B>(mut operation_builder: B) -> Result<B::Ok, B::Error>
        where
            B: HttpOperationBuilder,
        {
            operation_builder.collect_operation(
                HttpOperationId::new("NoteController.get_many", *GetManyEndpointCallsite),
                "GET",
                "/api/notes",
                Some(["Notes"]),
                Some("Retrieves all notes."),
                false,
                <Result<GetManyHttpResponse<NoteViewModel>, GetManyHttpError> as openapi::HttpResponse>::describe
            )
        }
    }
};
