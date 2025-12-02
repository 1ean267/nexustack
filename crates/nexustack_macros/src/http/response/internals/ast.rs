/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/ast.rs
 */

use crate::{http::response::internals::attr, internals::Ctxt};
use proc_macro2::{Span, TokenStream};
use syn::{Token, punctuated::Punctuated, spanned::Spanned};

/// A source data structure annotated with `#[http_response]`,
/// parsed into an internal representation.
pub struct Container<'a> {
    /// The struct or enum name (without generics).
    pub ident: syn::Ident,
    /// Attributes on the structure, parsed for OpenApi.
    pub attrs: attr::Container,
    /// The contents of the struct or enum.
    pub data: Data<'a>,
    /// Original input.
    pub original: syn::DeriveInput,
}

/// The fields of a struct or enum.
///
/// Analogous to `syn::Data`.
pub enum Data<'a> {
    Enum(Vec<Variant<'a>>),
    Struct(Style<'a>),
}

/// A variant of an enum.
pub struct Variant<'a> {
    pub ident: syn::Ident,
    pub attrs: attr::Variant,
    pub style: Style<'a>,
    pub original: syn::Variant,
}

/// A field of a struct.
#[derive(Clone)]
pub struct Field<'a> {
    // pub attrs: attr::Field,
    pub ty: &'a syn::Type,
    pub original: &'a syn::Field,
}

#[derive(Clone)]
pub enum Style<'a> {
    /// One unnamed field.
    Newtype(Field<'a>),
    /// No fields.
    Unit,
}

impl<'a> Container<'a> {
    /// Convert the raw Syn ast into a parsed container object, collecting errors in `cx`.
    pub fn from_ast(
        cx: &Ctxt,
        attr: TokenStream,
        item: &'a mut syn::DeriveInput,
    ) -> Option<Container<'a>> {
        let attrs = attr::Container::from_ast(cx, attr, item);
        let original = item.clone();
        let ident = original.ident.clone();
        let span = original.span();

        let data = match item {
            syn::DeriveInput {
                data: syn::Data::Enum(data),
                ..
            } => Data::Enum(enum_from_ast(
                cx,
                &original,
                &mut data.variants,
                attrs.api_skip(),
            )),
            syn::DeriveInput {
                data: syn::Data::Struct(data),
                ..
            } => {
                let style = struct_from_ast(cx, span, &mut data.fields);
                Data::Struct(style)
            }
            item => {
                cx.error_spanned_by(item, "Unions are not supported");
                return None;
            }
        };

        let item = Container {
            ident,
            attrs,
            data,
            original,
        };
        // check::check(cx, &mut item);
        Some(item)
    }
}

fn enum_from_ast<'a>(
    cx: &Ctxt,
    item: &syn::DeriveInput,
    variants: &'a mut Punctuated<syn::Variant, Token![,]>,
    cont_api_skip: bool,
) -> Vec<Variant<'a>> {
    variants
        .iter_mut()
        .map(|variant| {
            let original = variant.clone();
            let attrs = attr::Variant::from_ast(cx, item, variant, cont_api_skip);
            let style = struct_from_ast(cx, variant.ident.span(), &mut variant.fields);
            Variant {
                ident: variant.ident.clone(),
                attrs,
                style,
                original,
            }
        })
        .collect::<Vec<Variant>>()
}

fn struct_from_ast<'a>(cx: &Ctxt, item_span: Span, fields: &'a mut syn::Fields) -> Style<'a> {
    match fields {
        syn::Fields::Named(_) => {
            cx.error(item_span, "Expected zero or one unnamed field.");
            Style::Unit
        }
        syn::Fields::Unnamed(fields) => {
            if fields.unnamed.len() == 1 {
                // TODO: Better pattern matching
                let field = fields.unnamed.first().unwrap();

                Style::Newtype(Field {
                    // attrs: attr::Field::from_ast(cx, i, field, container_default),
                    ty: &field.ty,
                    original: field,
                })
            } else {
                cx.error(item_span, "Expected zero or one unnamed field.");
                Style::Unit
            }
        }
        syn::Fields::Unit => Style::Unit,
    }
}
