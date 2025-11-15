/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/attr.rs
 */

use crate::http::internals::symbol::*;
use crate::internals::Ctxt;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use std::{borrow::Cow, collections::BTreeSet, iter::FromIterator, num::NonZeroU16};
use syn::{
    Ident, Token, meta::ParseNestedMeta, parse::Parser, parse_quote, punctuated::Punctuated,
    spanned::Spanned, token,
};

// This module handles parsing of attributes. The entrypoints
// are `attr::Container::from_ast`, `attr::Variant::from_ast`, and
// `attr::Field::from_ast`. Each returns an instance of the corresponding
// struct. Note that none of them return a Result. Unrecognized, malformed, or
// duplicated attributes result in a span_err but otherwise are ignored. The
// user will see errors simultaneously for all bad attributes in the crate
// rather than just the first.

pub(crate) struct Attr<'c, T> {
    cx: &'c Ctxt,
    name: Symbol,
    tokens: TokenStream,
    value: Option<T>,
}

impl<'c, T> Attr<'c, T> {
    fn none(cx: &'c Ctxt, name: Symbol) -> Self {
        Attr {
            cx,
            name,
            tokens: TokenStream::new(),
            value: None,
        }
    }

    fn set<A: ToTokens>(&mut self, obj: A, value: T) {
        let tokens = obj.into_token_stream();

        if self.value.is_some() {
            let msg = format!("duplicate attribute `{}`", self.name);
            self.cx.error_spanned_by(tokens, msg);
        } else {
            self.tokens = tokens;
            self.value = Some(value);
        }
    }

    fn set_opt<A: ToTokens>(&mut self, obj: A, value: Option<T>) {
        if let Some(value) = value {
            self.set(obj, value);
        }
    }

    fn set_if_none(&mut self, value: T) {
        if self.value.is_none() {
            self.value = Some(value);
        }
    }

    pub(crate) fn get(self) -> Option<T> {
        self.value
    }

    fn get_with_tokens(self) -> Option<(TokenStream, T)> {
        match self.value {
            Some(v) => Some((self.tokens, v)),
            None => None,
        }
    }
}

struct BoolAttr<'c>(Attr<'c, ()>);

impl<'c> BoolAttr<'c> {
    fn none(cx: &'c Ctxt, name: Symbol) -> Self {
        BoolAttr(Attr::none(cx, name))
    }

    fn set_true<A: ToTokens>(&mut self, obj: A) {
        self.0.set(obj, ());
    }

    fn get(&self) -> bool {
        self.0.value.is_some()
    }
}

pub(crate) struct VecAttr<'c, T> {
    cx: &'c Ctxt,
    name: Symbol,
    first_dup_tokens: TokenStream,
    values: Vec<T>,
}

impl<'c, T> VecAttr<'c, T> {
    fn none(cx: &'c Ctxt, name: Symbol) -> Self {
        VecAttr {
            cx,
            name,
            first_dup_tokens: TokenStream::new(),
            values: Vec::new(),
        }
    }

    fn insert<A: ToTokens>(&mut self, obj: A, value: T) {
        if self.values.len() == 1 {
            self.first_dup_tokens = obj.into_token_stream();
        }
        self.values.push(value);
    }

    fn at_most_one(mut self) -> Option<T> {
        if self.values.len() > 1 {
            let dup_token = self.first_dup_tokens;
            let msg = format!("duplicate attribute `{}`", self.name);
            self.cx.error_spanned_by(dup_token, msg);
            None
        } else {
            self.values.pop()
        }
    }

    pub(crate) fn get(self) -> Vec<T> {
        self.values
    }
}

fn unraw(ident: &Ident) -> Ident {
    Ident::new(ident.to_string().trim_start_matches("r#"), ident.span())
}

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

        Container {
            api_skip: api_skip.get().unwrap_or(false),
            crate_path: crate_path.get(),
            deprecated: deprecated.get().unwrap_or(false),
            description: match description.get() {
                Some(description) => description,
                None => {
                    cx.error_spanned_by(item, "No description provided");
                    String::new()
                }
            },
            encoder: encoder.get(),
            status_code: status_code.get(),
        }
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
}

/// Represents variant attribute information
pub struct Variant {
    api_skip: bool,
    deprecated: bool,
    description: String,
    status_code: Option<syn::Ident>,
    encoder: Option<syn::Path>,
}

impl Variant {
    pub fn from_ast(cx: &Ctxt, variant: &mut syn::Variant) -> Self {
        let mut api_skip = Attr::none(cx, API_SKIP);
        let mut description = Attr::none(cx, DESCRIPTION);
        let mut deprecated = Attr::none(cx, DESCRIPTION);
        let mut encoder = Attr::none(cx, ENCODER);
        let mut status_code = Attr::none(cx, STATUS_CODE);

        for i in (0..variant.attrs.len()).rev() {
            let attr = &variant.attrs[i];

            if attr.path() != HTTP_RESPONSE_VARIANT {
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
                            encoder.set(&meta.path, container.ident.clone().into());
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

        Variant {
            api_skip: api_skip.get().unwrap_or(false),
            deprecated: deprecated.get().unwrap_or(false),
            description: match description.get() {
                Some(description) => description,
                None => {
                    cx.error_spanned_by(variant, "No description provided");
                    String::new()
                }
            },
            status_code: status_code.get(),
            encoder: encoder.get(),
        }
    }

    pub fn deprecated(&self) -> bool {
        self.deprecated
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

fn is_primitive_path(path: &syn::Path, primitive: &str) -> bool {
    path.leading_colon.is_none()
        && path.segments.len() == 1
        && path.segments[0].ident == primitive
        && path.segments[0].arguments.is_empty()
}

fn get_lit_str(
    cx: &Ctxt,
    attr_name: Symbol,
    meta: &ParseNestedMeta,
) -> syn::Result<Option<syn::LitStr>> {
    get_lit_str2(cx, attr_name, attr_name, meta)
}

fn get_lit_str2(
    cx: &Ctxt,
    attr_name: Symbol,
    meta_item_name: Symbol,
    meta: &ParseNestedMeta,
) -> syn::Result<Option<syn::LitStr>> {
    let expr: syn::Expr = meta.value()?.parse()?;
    get_lit_str2_expr(cx, attr_name, meta_item_name, &expr)
}

fn get_lit_str2_expr(
    cx: &Ctxt,
    attr_name: Symbol,
    meta_item_name: Symbol,
    expr: &syn::Expr,
) -> syn::Result<Option<syn::LitStr>> {
    let mut value = expr;
    while let syn::Expr::Group(e) = value {
        value = &e.expr;
    }
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(lit),
        ..
    }) = value
    {
        let suffix = lit.suffix();
        if !suffix.is_empty() {
            cx.error_spanned_by(
                lit,
                format!("unexpected suffix `{suffix}` on string literal"),
            );
        }
        Ok(Some(lit.clone()))
    } else {
        cx.error_spanned_by(
            expr,
            format!("expected {attr_name} attribute to be a string: `{meta_item_name} = \"...\"`"),
        );
        Ok(None)
    }
}

fn parse_lit_into_bool(
    cx: &Ctxt,
    attr_name: Symbol,
    meta: &ParseNestedMeta,
) -> syn::Result<Option<bool>> {
    let string = match get_lit_str(cx, attr_name, meta)? {
        Some(string) => string,
        None => return Ok(None),
    };

    if string.value().eq("true") {
        return Ok(Some(true));
    }

    if string.value().eq(&"false") {
        return Ok(Some(false));
    }

    cx.error_spanned_by(
        &string,
        format!("failed to parse path: {:?}", string.value()),
    );

    Ok(None)
}

fn parse_lit_into_path(
    cx: &Ctxt,
    attr_name: Symbol,
    meta: &ParseNestedMeta,
) -> syn::Result<Option<syn::Path>> {
    let string = match get_lit_str(cx, attr_name, meta)? {
        Some(string) => string,
        None => return Ok(None),
    };

    Ok(match string.parse() {
        Ok(path) => Some(path),
        Err(_) => {
            cx.error_spanned_by(
                &string,
                format!("failed to parse path: {:?}", string.value()),
            );
            None
        }
    })
}

fn parse_lit_into_ident(
    cx: &Ctxt,
    attr_name: Symbol,
    meta: &ParseNestedMeta,
) -> syn::Result<Option<syn::Ident>> {
    let string = match get_lit_str(cx, attr_name, meta)? {
        Some(string) => string,
        None => return Ok(None),
    };

    Ok(match string.parse() {
        Ok(ident) => Some(ident),
        Err(_) => {
            cx.error_spanned_by(
                &string,
                format!("failed to parse ident: {:?}", string.value()),
            );
            None
        }
    })
}

fn parse_lit_into_expr_path(
    cx: &Ctxt,
    attr_name: Symbol,
    meta: &ParseNestedMeta,
) -> syn::Result<Option<syn::ExprPath>> {
    let string = match get_lit_str(cx, attr_name, meta)? {
        Some(string) => string,
        None => return Ok(None),
    };

    Ok(match string.parse() {
        Ok(expr) => Some(expr),
        Err(_) => {
            cx.error_spanned_by(
                &string,
                format!("failed to parse path: {:?}", string.value()),
            );
            None
        }
    })
}

fn parse_lit_into_where(
    cx: &Ctxt,
    attr_name: Symbol,
    meta_item_name: Symbol,
    meta: &ParseNestedMeta,
) -> syn::Result<Vec<syn::WherePredicate>> {
    let string = match get_lit_str2(cx, attr_name, meta_item_name, meta)? {
        Some(string) => string,
        None => return Ok(Vec::new()),
    };

    Ok(
        match string.parse_with(Punctuated::<syn::WherePredicate, Token![,]>::parse_terminated) {
            Ok(predicates) => Vec::from_iter(predicates),
            Err(err) => {
                cx.error_spanned_by(string, err);
                Vec::new()
            }
        },
    )
}

fn parse_lit_into_ty(
    cx: &Ctxt,
    attr_name: Symbol,
    meta: &ParseNestedMeta,
) -> syn::Result<Option<syn::Type>> {
    let string = match get_lit_str(cx, attr_name, meta)? {
        Some(string) => string,
        None => return Ok(None),
    };

    Ok(match string.parse() {
        Ok(ty) => Some(ty),
        Err(_) => {
            cx.error_spanned_by(
                &string,
                format!("failed to parse type: {} = {:?}", attr_name, string.value()),
            );
            None
        }
    })
}
