/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::service_provider::ServiceProvider;

pub struct ServiceScope {
    service_provider: ServiceProvider,
}

impl ServiceScope {
    pub(crate) fn new(service_provider: ServiceProvider) -> Self {
        Self { service_provider }
    }

    pub fn service_provider(&self) -> &ServiceProvider {
        &self.service_provider
    }
}
