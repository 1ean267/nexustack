/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::http::ConfigureRouter;
use std::convert::Infallible;

pub trait UserExtractor: Clone + Send + Sync + 'static {
    type User: Clone + Send + Sync + 'static;
    type Error: axum::response::IntoResponse;

    fn extract_user(self, req: &mut axum::extract::Request) -> Result<Self::User, Self::Error>; // TODO: What to do with the body type??
}

#[derive(Debug, Clone, Copy)]
pub struct NoUser;

impl<'a> From<&'a NoUser> for NoUser {
    fn from(value: &'a NoUser) -> Self {
        Self
    }
}

impl<'a> From<&'a NoUser> for () {
    fn from(value: &'a NoUser) -> Self {
        ()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NoUserExtractor;

impl UserExtractor for NoUserExtractor {
    type User = NoUser;
    type Error = Infallible;

    fn extract_user(self, _req: &mut axum::extract::Request) -> Result<Self::User, Self::Error> {
        Ok(NoUser)
    }
}

pub(crate) struct UserExtractorConfigureRouter<E>(pub(crate) E);

impl<E: UserExtractor + Clone> ConfigureRouter for UserExtractorConfigureRouter<E> {
    fn configure_router(&self, router: axum::Router) -> axum::Router {
        router.layer(axum::middleware::from_fn_with_state(
            self.0.clone(),
            user_extract_middleware::<E>,
        ))
    }
}

async fn user_extract_middleware<Extractor: UserExtractor>(
    axum::extract::State(user_extractor): axum::extract::State<Extractor>,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, Extractor::Error> {
    let user = user_extractor.extract_user(&mut request)?;

    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

#[derive(Debug, thiserror::Error)]
#[error("Unable to resolve user, user not authenticated.")]
pub struct UserResolveError;
