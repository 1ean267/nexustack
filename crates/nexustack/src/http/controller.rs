/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    http::{HttpEndpoint, response::IntoResponseWithContext},
    inject::FromInjector,
};

#[cfg(feature = "openapi")]
use crate::openapi;

pub trait HttpEndpointsBuilder {
    #[cfg(feature = "openapi")]
    fn add_endpoint<E>(&mut self)
    where
        E: HttpEndpoint + FromInjector + openapi::HttpOperation + Send + Sync + 'static,
        E::Request: Send,
        <<E as HttpEndpoint>::Response as IntoResponseWithContext<()>>::Context: Send;

    fn add_hidden_endpoint<E>(&mut self)
    where
        E: HttpEndpoint + FromInjector + Send + Sync + 'static,
        E::Request: Send,
        <<E as HttpEndpoint>::Response as IntoResponseWithContext<()>>::Context: Send;
}

pub trait HttpController {
    fn build_endpoints<B>(builder: B)
    where
        B: HttpEndpointsBuilder;
}
