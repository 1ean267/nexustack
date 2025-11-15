/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::ApplicationPart;
use std::{borrow::Cow, time::Instant};
use tokio_util::sync::CancellationToken;

pub struct WithInstrumentation<T>(pub(crate) T);

impl<T> ApplicationPart for WithInstrumentation<T>
where
    T: ApplicationPart + Send + Sync,
{
    type Error = T::Error;

    fn name() -> Cow<'static, str> {
        T::name()
    }

    #[tracing::instrument(
        name = "application_part.before_startup",
        skip(self, cancellation_token),
        fields(application_part = Self::name().to_string())
    )]
    async fn before_startup(
        &mut self,
        cancellation_token: CancellationToken,
    ) -> Result<(), Self::Error> {
        let start = Instant::now();
        tracing::debug!("Executing before_startup phase for application part");

        self.0
            .before_startup(cancellation_token)
            .await.inspect(|()| tracing::debug!(took_ms = start.elapsed().as_millis(), "Executed before_startup phase for application part sucessfully"))
            .inspect_err(
                |err| tracing::error!(took_ms = start.elapsed().as_millis(), %err, "Error during before_startup phase for application part"),
            )
    }

    #[tracing::instrument(
        name = "application_part.run",
        skip(self, cancellation_token),
        fields(application_part = Self::name().to_string())
    )]
    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<(), Self::Error> {
        let start = Instant::now();
        tracing::debug!("Executing run phase for application part");

        self.0
            .run(cancellation_token)
            .await.inspect(|()| tracing::debug!(took_ms = start.elapsed().as_millis(), "Executed run phase for application part sucessfully"))
            .inspect_err(
                |err| tracing::error!(took_ms = start.elapsed().as_millis(), %err, "Error during run phase for application part"),
            )
    }

    #[tracing::instrument(
        name = "application_part.before_shutdown",
        skip(self, cancellation_token),
        fields(application_part = Self::name().to_string())
    )]
    async fn before_shutdown(
        &mut self,
        cancellation_token: CancellationToken,
    ) -> Result<(), Self::Error> {
        let start = Instant::now();
        tracing::debug!("Executing before_shutdown phase for application part");

        self.0
            .before_shutdown(cancellation_token)
            .await.inspect(|()| tracing::debug!(took_ms = start.elapsed().as_millis(), "Executed before_shutdown phase for application part sucessfully"))
            .inspect_err(
                |err| tracing::error!(took_ms = start.elapsed().as_millis(), %err, "Error during before_shutdown phase for application part"),
            )
    }
}
