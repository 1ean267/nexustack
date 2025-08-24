/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

//! `OpenAPI` JSON specification module.
//!
//! This module provides types and utilities for representing and working with
//! `OpenAPI` specifications in JSON format, including support fo`OpenAPI` 3.0 and 3.1.
//! It re-exports all relevant object types for convenient access.

mod callback_object;
mod components_object;
mod contact_object;
mod discriminator_object;
mod encoding_object;
mod example_object;
mod external_documentation_object;
mod header_object;
mod info_object;
mod license_object;
mod link_object;
mod media_type_object;
mod oauth_flow_object;
mod oauth_flows_object;
mod open_api_object;
mod operation_object;
mod parameter_location;
mod parameter_object;
mod parameter_style;
mod path_item_object;
mod paths_object;
mod reference_object;
mod request_body_object;
mod response_object;
mod schema_object;
mod security_scheme_location;
mod security_scheme_object;
mod security_scheme_type;
mod server_object;
mod server_variable_object;
mod tag_object;
mod xml_object;

pub use callback_object::*;
pub use components_object::*;
pub use contact_object::*;
pub use discriminator_object::*;
pub use encoding_object::*;
pub use example_object::*;
pub use external_documentation_object::*;
pub use header_object::*;
pub use info_object::*;
pub use license_object::*;
pub use link_object::*;
pub use media_type_object::*;
pub use oauth_flow_object::*;
pub use oauth_flows_object::*;
pub use open_api_object::*;
pub use operation_object::*;
pub use parameter_location::*;
pub use parameter_object::*;
pub use parameter_style::*;
pub use path_item_object::*;
pub use paths_object::*;
pub use reference_object::*;
pub use request_body_object::*;
pub use response_object::*;
pub use schema_object::*;
pub use security_scheme_location::*;
pub use security_scheme_object::*;
pub use security_scheme_type::*;
pub use server_object::*;
pub use server_variable_object::*;
pub use tag_object::*;
pub use xml_object::*;

/// Represents the supported `OpenAPI` specification versions.
///
/// This enum is used to distinguish between different versions of the `OpenAPI` specification.
/// Currently, only `OpenAPI` 3.0 an`OpenAPI` 3.1 are supported.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum Specification {
    /// `OpenAPI` Specification version 3.0
    OpenAPI3_0,
    /// `OpenAPI` Specification version 3.1
    OpenAPI3_1,
}

impl std::fmt::Display for Specification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAPI3_0 => f.write_str("Open API 3.0"),
            Self::OpenAPI3_1 => f.write_str("Open API 3.1"),
        }
    }
}
