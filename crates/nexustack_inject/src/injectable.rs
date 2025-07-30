/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{injection_error::ConstructionResult, injector::Injector};

// Split into two traits, as there may be services, that we want to construct from the service-provider
// but disallow to be added to the service-collection.

pub trait FromInjector {
    fn from_injector(injector: &Injector) -> ConstructionResult<Self>
    where
        Self: Sized;
}

pub trait Injectable: FromInjector {}
