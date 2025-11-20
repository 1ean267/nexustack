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

use std::{borrow::Cow, collections::HashMap};

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

/// A list of security requirement objects for an operation, as defined by the `OpenAPI` Specification.
///
/// Each entry in the outer vector represents an alternative set of security requirements (logical OR).
/// Each `HashMap` maps a security scheme name to a list of required scopes (logical AND within the map).
/// An empty map (`{}`) indicates that security is optional for the operation.
///
/// See [OpenAPI Specification: Security Requirement Object](https://swagger.io/specification/#security-requirement-object)
pub type SecurityRequirements = Vec<HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>>;
