/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    ApplicationPart,
    application::{ApplicationPartBuilder, configurable::Configurable},
    inject::ServiceProvider,
};
use either::Either;
use std::borrow::Cow;
use tokio_util::sync::CancellationToken;

impl<Left, Right> ApplicationPart for Either<Left, Right>
where
    Left: ApplicationPart + Sync,
    Right: ApplicationPart + Sync,
{
    type Error = Either<Left::Error, Right::Error>;

    fn name(&self) -> Cow<'static, str> {
        match self {
            Self::Left(left) => left.name(),
            Self::Right(right) => right.name(),
        }
    }

    async fn before_startup(
        &self,
        cancellation_token: CancellationToken,
    ) -> Result<(), Self::Error> {
        match self {
            Self::Left(left) => left
                .before_startup(cancellation_token)
                .await
                .map_err(Either::Left),
            Self::Right(right) => right
                .before_startup(cancellation_token)
                .await
                .map_err(Either::Right),
        }
    }

    async fn run(&self, cancellation_token: CancellationToken) -> Result<(), Self::Error> {
        match self {
            Self::Left(left) => left.run(cancellation_token).await.map_err(Either::Left),
            Self::Right(right) => right.run(cancellation_token).await.map_err(Either::Right),
        }
    }

    async fn before_shutdown(
        &self,
        cancellation_token: CancellationToken,
    ) -> Result<(), Self::Error> {
        match self {
            Self::Left(left) => left
                .before_shutdown(cancellation_token)
                .await
                .map_err(Either::Left),
            Self::Right(right) => right
                .before_shutdown(cancellation_token)
                .await
                .map_err(Either::Right),
        }
    }
}

impl<Left, Right> ApplicationPartBuilder for Either<Left, Right>
where
    Left: ApplicationPartBuilder,
    <Left as ApplicationPartBuilder>::ApplicationPart: Sync,
    Right: ApplicationPartBuilder,
    <Right as ApplicationPartBuilder>::ApplicationPart: Sync,
{
    type ApplicationPart = Either<Left::ApplicationPart, Right::ApplicationPart>;

    fn build(
        self,
        service_provider: ServiceProvider,
    ) -> crate::inject::ConstructionResult<Self::ApplicationPart> {
        match self {
            Self::Left(left) => left.build(service_provider).map(Either::Left),
            Self::Right(right) => right.build(service_provider).map(Either::Right),
        }
    }
}

impl<'a, Left, Right> Configurable<'a> for Either<Left, Right>
where
    Left: Configurable<'a>,
    Right: Configurable<'a>,
{
    fn configure<I: 'a, C>(&mut self, configure: C) -> Result<(), C>
    where
        C: FnOnce(&mut I),
    {
        match self {
            Self::Left(left) => left.configure(configure),
            Self::Right(right) => right.configure(configure),
        }
    }
}
