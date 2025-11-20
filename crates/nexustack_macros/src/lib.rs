/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(any(feature = "inject", feature = "cron"))]
mod dummy;

mod internals;

#[cfg(feature = "openapi")]
#[macro_use]
mod fragment;

#[cfg(feature = "inject")]
mod inject;

#[cfg(feature = "openapi")]
mod openapi;

#[cfg(feature = "cron")]
mod cron;

#[cfg(feature = "module")]
mod module;

#[cfg(feature = "http")]
mod http;

#[cfg(any(feature = "http", feature = "openapi"))]
mod bound;

#[cfg(feature = "inject")]
use crate::inject::injectable as injectable_impl;

#[cfg(feature = "openapi")]
use crate::openapi::api_schema as api_schema_impl;

#[cfg(feature = "cron")]
use crate::cron::{cron as cron_impl, cron_jobs as cron_jobs_impl};

#[cfg(feature = "module")]
use crate::module::module as module_impl;

#[cfg(feature = "http")]
use crate::http::{http_controller as http_controller_impl, http_response as http_response_impl};

/// Returns `true` when `attr` is a path-only attribute whose path matches a non-empty
/// suffix of `segments`.
///
/// The path must not start with `::`, and it must not contain generic arguments.
///
/// # Examples
///
/// Given `segments` equal to `&["nexustack", "http", "http_response", "variant"]`,
/// all of the following match:
///
/// - `#[nexustack::http::http_response::variant]`
/// - `#[http::http_response::variant]`
/// - `#[http_response::variant]`
/// - `#[variant]`
///
/// This does not match:
///
/// - `#[::nexustack::http::http_response::variant]`
/// - `#[some::other::path]`
///
pub(crate) fn is_attr(attr: &syn::Attribute, segments: &[&str]) -> bool {
    match &attr.meta {
        syn::Meta::Path(attr_path) => is_path(attr_path, segments),
        syn::Meta::List(attr_list) => is_path(&attr_list.path, segments),
        _ => false,
    }
}

fn is_path(path: &syn::Path, segments: &[&str]) -> bool {
    if path.leading_colon.is_some() || path.segments.is_empty() || segments.is_empty() {
        return false;
    }

    for (path_seg, seg_str) in Iterator::zip(path.segments.iter().rev(), segments.iter().rev()) {
        if !path_seg.arguments.is_empty() || path_seg.ident != *seg_str {
            return false;
        }
    }

    true
}

#[cfg(feature = "inject")]
#[proc_macro_attribute]
pub fn injectable(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    injectable_impl(attr.into(), item.into()).into()
}

#[cfg(feature = "openapi")]
#[proc_macro_attribute]
pub fn api_schema(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    api_schema_impl(attr.into(), item.into()).into()
}

#[cfg(feature = "cron")]
#[proc_macro_attribute]
#[cfg_attr(not(doctest), doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "src/cron/CRON.md")))]
pub fn cron(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    cron_impl(attr.into(), item.into()).into()
}

/// A macro to register multiple cron jobs with a configuration function.
///
/// This expands to a closure that configures the provided cron jobs.
#[cfg(feature = "cron")]
#[proc_macro]
pub fn cron_jobs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    cron_jobs_impl(input.into()).into()
}

#[cfg(feature = "module")]
#[cfg_attr(not(doctest), doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "/src/module/MODULE.md")))]
#[proc_macro_attribute]
pub fn module(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    module_impl(attr.into(), item.into()).into()
}

#[cfg(feature = "http")]
#[proc_macro_attribute]
pub fn http_response(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    http_response_impl(attr.into(), item.into()).into()
}

#[cfg(feature = "http")]
#[proc_macro_attribute]
pub fn http_controller(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    http_controller_impl(attr.into(), item.into()).into()
}

#[cfg(test)]
mod tests {
    use super::is_attr;
    use syn::parse_quote;

    #[test]
    fn matches_suffix_paths() {
        assert!(is_attr(
            &parse_quote!(#[nexustack::http::http_response::variant]),
            &["nexustack", "http", "http_response", "variant"]
        ));
        assert!(is_attr(
            &parse_quote!(#[http::http_response::variant]),
            &["nexustack", "http", "http_response", "variant"]
        ));
        assert!(is_attr(
            &parse_quote!(#[http_response::variant]),
            &["nexustack", "http", "http_response", "variant"]
        ));
        assert!(is_attr(
            &parse_quote!(#[variant]),
            &["nexustack", "http", "http_response", "variant"]
        ));
    }

    #[test]
    fn rejects_leading_colon_paths() {
        let attr: syn::Attribute = parse_quote!(#[::nexustack::http::http_response::variant]);

        assert!(!is_attr(
            &attr,
            &["nexustack", "http", "http_response", "variant"]
        ));
    }

    #[test]
    fn rejects_wrong_paths() {
        let attr: syn::Attribute = parse_quote!(#[some::other::path]);

        assert!(!is_attr(
            &attr,
            &["nexustack", "http", "http_response", "variant"]
        ));
    }

    #[test]
    fn matches_list_attributes() {
        let attr: syn::Attribute = parse_quote!(#[http_response::variant(foo)]);

        assert!(is_attr(
            &attr,
            &["nexustack", "http", "http_response", "variant"]
        ));
    }

    #[test]
    fn rejects_name_value_attributes() {
        let attr: syn::Attribute = parse_quote!(#[http_response::variant = foo]);

        assert!(!is_attr(
            &attr,
            &["nexustack", "http", "http_response", "variant"]
        ));
    }
}
