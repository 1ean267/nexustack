/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    ApplicationPart,
    application::{ApplicationPartBuilder, configurable::Configurable},
    inject::{ConstructionResult, ServiceProvider},
};
use std::convert::Infallible;
use tokio_util::sync::CancellationToken;

impl ApplicationPart for () {
    type Error = Infallible;

    async fn run(&self, _cancellation_token: CancellationToken) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl ApplicationPartBuilder for () {
    type ApplicationPart = ();

    fn build(
        self,
        _service_provider: ServiceProvider,
    ) -> ConstructionResult<Self::ApplicationPart> {
        Ok(())
    }
}

impl Configurable<'_> for () {
    fn configure<I, C>(&mut self, configure: C) -> Result<(), C>
    where
        C: FnOnce(&mut I),
    {
        Err(configure)
    }
}
