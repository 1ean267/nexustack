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

// impl<T> ApplicationPartBuilder for WithInstrumentation<T>
// where
//     T: ApplicationPartBuilder,
//     <T as ApplicationPartBuilder>::ApplicationPart: Send + Sync,
//     <<T as ApplicationPartBuilder>::ApplicationPart as ApplicationPart>::Error: std::fmt::Display,
// {
//     type ApplicationPart = WithInstrumentation<T::ApplicationPart>;

//     fn build(self, service_provider: ServiceProvider) -> ConstructionResult<Self::ApplicationPart> {
//         Ok(WithInstrumentation(self.0.build(service_provider)?))
//     }
// }

// impl<'a, T> Configurable<'a> for WithInstrumentation<T>
// where
//     T: Configurable<'a>,
// {
//     fn has_item<I: 'static>() -> bool {
//         <T as Configurable<'_>>::has_item::<I>()
//     }
// }

impl<T> ApplicationPart for WithInstrumentation<T>
where
    T: ApplicationPart + Send + Sync,
    <T as ApplicationPart>::Error: std::fmt::Display,
{
    type Error = T::Error;

    fn name(&self) -> Cow<'static, str> {
        self.0.name()
    }

    #[tracing::instrument(
        name = "application_part.before_startup",
        skip(self, cancellation_token),
        fields(application_part = self.name().to_string())
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
        fields(application_part = self.name().to_string())
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
        fields(application_part = self.name().to_string())
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

// impl<T, FromInner, InnerIndex> Chain<T, InnerIndex> for WithInstrumentation<FromInner>
// where
//     FromInner: Chain<T, InnerIndex>,
//     InnerIndex: Index,
// {
//     fn get(&self) -> &T {
//         self.0.get()
//     }

//     fn get_mut(&mut self) -> &mut T {
//         self.0.get_mut()
//     }
// }
