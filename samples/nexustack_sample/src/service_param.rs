/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::response::{GetOneHttpResponse, InternalServerError};
use nexustack::{
    ApplicationBuilder,
    http::{Http, HttpApplicationBuilder, http_controller},
    inject::injectable,
    module,
};

/// Extension trait to add the `ServiceParamModule` module to the application builder.
#[module(features = "Http")]
pub trait ServiceParamModule {
    /// Adds the `ServiceParamModule` module to the application builder.
    fn add_service_param(self) -> impl ApplicationBuilder {
        self.configure_services(|services| {
            services.add_scoped_factory(|_| Ok(MyService("Hello from scoped service".to_string())));
        })
        .configure_http(|http_builder| {
            http_builder.add_controller::<ServiceParamController>();
        })
    }
}

#[injectable]
pub struct OptionalService(String);

impl Default for OptionalService {
    fn default() -> Self {
        Self("[Optional service not found]".to_string())
    }
}

#[injectable]
#[derive(Clone)]
pub struct MyService(String);

impl MyService {
    fn get_message(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct ServiceParamController;

/// HTTP controller for showing service parameters.
#[http_controller(tags = "ServiceParam")]
impl ServiceParamController {
    #[ctor]
    const fn new() -> Self {
        Self {}
    }

    /// Shows the use of a service parameter
    #[get(route = "/api/service_param")]
    pub async fn get(
        &self,
        #[service] my_service: MyService,
    ) -> Result<GetOneHttpResponse<String>, InternalServerError> {
        Ok(GetOneHttpResponse(format!(
            "Message from service: {}",
            my_service.get_message()
        )))
    }

    /// Shows the use of a service parameter
    #[get(route = "/api/optional_service_param")]
    pub async fn get_optional(
        &self,
        #[service] my_service: MyService,
        #[service(default)] optional_service: OptionalService,
    ) -> Result<GetOneHttpResponse<String>, InternalServerError> {
        Ok(GetOneHttpResponse(format!(
            "Message from service: {}, message from optional service: {}",
            my_service.get_message(),
            &optional_service.0,
        )))
    }
}
