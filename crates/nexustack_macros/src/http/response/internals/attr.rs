/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/attr.rs
 */

use crate::internals::{
    Ctxt,
    attr::{
        Attr, get_lit_str, get_lit_str2_expr, parse_lit_into_bool, parse_lit_into_ident,
        parse_lit_into_path,
    },
    symbol::*,
};
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{Token, parse::Parser as _};

/// Represents struct or enum attribute information.
#[derive(Debug)]
pub struct Container {
    api_skip: bool,
    crate_path: Option<syn::Path>,
    deprecated: bool,
    description: String,
    status_code: Option<syn::Ident>,
    encoder: Option<syn::Path>,
}

impl Container {
    /// Extract out the `#[api_property(...)]` attributes from an item.
    pub fn from_ast(cx: &Ctxt, meta: TokenStream, item: &syn::DeriveInput) -> Self {
        let mut api_skip = Attr::none(cx, API_SKIP);
        let mut crate_path = Attr::none(cx, CRATE);
        let mut deprecated = Attr::none(cx, DEPRECATED);
        let mut description = Attr::none(cx, DESCRIPTION);
        let mut encoder = Attr::none(cx, ENCODER);
        let mut status_code = Attr::none(cx, STATUS_CODE);

        if !meta.is_empty() {
            let parser = syn::meta::parser(|meta| {
                if meta.path == API_SKIP {
                    if meta.input.peek(Token![=]) {
                        // #[http_response(api_skip = "...")]
                        if let Some(value) = parse_lit_into_bool(cx, API_SKIP, &meta)? {
                            api_skip.set(&meta.path, value)
                        }
                    } else {
                        // #[http_response(api_skip)]
                        api_skip.set(&meta.path, true)
                    }
                } else if meta.path == CRATE {
                    // #[http_response(crate = "foo")]
                    if let Some(path) = parse_lit_into_path(cx, CRATE, &meta)? {
                        crate_path.set(&meta.path, path);
                    }
                } else if meta.path == DEPRECATED {
                    if meta.input.peek(Token![=]) {
                        // #[http_response(deprecated = "...")]
                        if let Some(value) = parse_lit_into_bool(cx, DEPRECATED, &meta)? {
                            deprecated.set(&meta.path, value)
                        }
                    } else {
                        // #[http_response(deprecated)]
                        deprecated.set(&meta.path, true)
                    }
                } else if meta.path == DESCRIPTION {
                    // #[http_response(description = "...")]
                    if let Some(s) = get_lit_str(cx, DESCRIPTION, &meta)? {
                        description.set(&meta.path, s.value());
                    }
                } else if meta.path == ENCODER {
                    // #[http_response(encoder = "...")]
                    if let Some(path) = parse_lit_into_path(cx, ENCODER, &meta)? {
                        if is_primitive_path(&path, "Self") {
                            encoder.set(&meta.path, item.ident.clone().into());
                        } else {
                            encoder.set(&meta.path, path);
                        }
                    }
                } else if meta.path == STATUS_CODE {
                    // #[http_response(status_code = "...")]
                    if let Some(ident) = parse_lit_into_ident(cx, STATUS_CODE, &meta)? {
                        status_code.set(&meta.path, ident);
                    }
                } else {
                    let path = meta.path.to_token_stream().to_string().replace(' ', "");
                    return Err(meta.error(format_args!("unknown container attribute `{path}`")));
                }
                Ok(())
            });

            // Parse
            let parse_res = parser.parse2(meta);
            if let Err(err) = parse_res {
                cx.syn_error(err);
            }
        }

        for attr in &item.attrs {
            if matches!(&attr.meta, syn::Meta::Path(path) if path == DEPRECATED) {
                deprecated.set_if_none(true);
            }

            if let syn::Meta::NameValue(meta) = &attr.meta
                && meta.path == DOC
                && let Ok(Some(s)) = get_lit_str2_expr(cx, DOC, DOC, &meta.value)
            {
                description.set_if_none(s.value().trim().to_string());
            }
        }

        let api_skip = api_skip.get().unwrap_or(false);

        Container {
            api_skip,
            crate_path: crate_path.get(),
            deprecated: deprecated.get().unwrap_or(false),
            description: match description.get() {
                Some(description) => description,
                None => {
                    if !api_skip {
                        cx.error_spanned_by(item, "No description provided");
                    }
                    String::new()
                }
            },
            encoder: encoder.get(),
            status_code: status_code.get(),
        }
    }

    pub fn api_skip(&self) -> bool {
        self.api_skip
    }

    pub fn custom_crate_path(&self) -> Option<&syn::Path> {
        self.crate_path.as_ref()
    }

    pub fn deprecated(&self) -> bool {
        self.deprecated
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn status_code(&self) -> syn::Ident {
        match &self.status_code {
            Some(ident) => ident.clone(),
            None => syn::Ident::new("OK", Span::call_site()),
        }
    }

    pub fn encoder(&self) -> syn::Path {
        match &self.encoder {
            Some(path) => path.clone(),
            None => syn::parse_quote! { _nexustack::http::encoding::DefaultEncoder },
        }
    }
}

/// Represents variant attribute information
pub struct Variant {
    api_skip: bool,
    deprecated: bool,
    description: String,
    status_code: Option<syn::Ident>,
    encoder: Option<syn::Path>,
}

// TODO: Stolen from cron
fn is_http_response_variant_attr(attr: &syn::Attribute) -> bool {
    let path = attr.path();

    is_path(path, &["http_response", "variant"])
        || is_path(path, &["http", "http_response", "variant"])
        || is_path(path, &["nexustack", "http", "http_response", "variant"])
}

// TODO: Stolen from inject expand, refactor later
fn is_path(path: &syn::Path, segments: &[&str]) -> bool {
    if path.leading_colon.is_some() {
        return false;
    }

    if path.segments.len() != segments.len() {
        return false;
    }

    for (i, segment) in path.segments.iter().enumerate() {
        if !segment.arguments.is_none() {
            return false;
        }

        if segment.ident != segments[i] {
            return false;
        }
    }

    true
}

impl Variant {
    pub fn from_ast(
        cx: &Ctxt,
        item: &syn::DeriveInput,
        variant: &mut syn::Variant,
        cont_api_skip: bool,
    ) -> Self {
        let mut api_skip = Attr::none(cx, API_SKIP);
        let mut description = Attr::none(cx, DESCRIPTION);
        let mut deprecated = Attr::none(cx, DESCRIPTION);
        let mut encoder = Attr::none(cx, ENCODER);
        let mut status_code = Attr::none(cx, STATUS_CODE);

        for i in (0..variant.attrs.len()).rev() {
            let attr = &variant.attrs[i];

            if !is_http_response_variant_attr(attr) {
                continue;
            }

            if let syn::Meta::List(meta) = &attr.meta {
                if meta.tokens.is_empty() {
                    variant.attrs.remove(i);
                    continue;
                }
            } else if let syn::Meta::Path(_) = &attr.meta {
                variant.attrs.remove(i);
                continue;
            }

            if let Err(err) = attr.parse_nested_meta(|meta| {
                if meta.path == API_SKIP {
                    if meta.input.peek(Token![=]) {
                        // #[http_response_variant(api_skip = "...")]
                        if let Some(value) = parse_lit_into_bool(cx, API_SKIP, &meta)? {
                            api_skip.set(&meta.path, value)
                        }
                    } else {
                        // #[http_response_variant(api_skip)]
                        api_skip.set(&meta.path, true)
                    }
                } else if meta.path == DESCRIPTION {
                    // #[http_response_variant(description = "...")]
                    if let Some(s) = get_lit_str(cx, DESCRIPTION, &meta)? {
                        description.set(&meta.path, s.value());
                    }
                } else if meta.path == DEPRECATED {
                    if meta.input.peek(Token![=]) {
                        // #[http_response_variant(deprecated = "...")]
                        if let Some(value) = parse_lit_into_bool(cx, DEPRECATED, &meta)? {
                            deprecated.set(&meta.path, value)
                        }
                    } else {
                        // #[http_response_variant(deprecated)]
                        deprecated.set(&meta.path, true)
                    }
                } else if meta.path == ENCODER {
                    // #[http_response_variant(encoder = "...")]
                    if let Some(path) = parse_lit_into_path(cx, ENCODER, &meta)? {
                        if is_primitive_path(&path, "Self") {
                            encoder.set(&meta.path, item.ident.clone().into());
                        } else {
                            encoder.set(&meta.path, path);
                        }
                    }
                } else if meta.path == STATUS_CODE {
                    // #[http_response_variant(status_code = "...")]
                    if let Some(ident) = parse_lit_into_ident(cx, STATUS_CODE, &meta)? {
                        status_code.set(&meta.path, ident);
                    }
                }
                Ok(())
            }) {
                cx.syn_error(err);
            }

            variant.attrs.remove(i);
        }

        for attr in &variant.attrs {
            if matches!(&attr.meta, syn::Meta::Path(path) if path == DEPRECATED) {
                deprecated.set_if_none(true);
            }

            if let syn::Meta::NameValue(meta) = &attr.meta
                && meta.path == DOC
                && let Ok(Some(s)) = get_lit_str2_expr(cx, DOC, DOC, &meta.value)
            {
                description.set_if_none(s.value().trim().to_string());
            }
        }

        let api_skip = api_skip.get().unwrap_or(false);

        Variant {
            api_skip,
            deprecated: deprecated.get().unwrap_or(false),
            description: match description.get() {
                Some(description) => description,
                None => {
                    if !api_skip && !cont_api_skip {
                        cx.error_spanned_by(variant, "No description provided");
                    }
                    String::new()
                }
            },
            status_code: status_code.get(),
            encoder: encoder.get(),
        }
    }

    pub fn api_skip(&self) -> bool {
        self.api_skip
    }

    pub fn deprecated(&self) -> bool {
        self.deprecated
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn status_code(&self) -> Option<&syn::Ident> {
        self.status_code.as_ref()
    }

    pub fn encoder(&self) -> Option<&syn::Path> {
        self.encoder.as_ref()
    }
}

fn is_primitive_path(path: &syn::Path, primitive: &str) -> bool {
    path.leading_colon.is_none()
        && path.segments.len() == 1
        && path.segments[0].ident == primitive
        && path.segments[0].arguments.is_empty()
}
