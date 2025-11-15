/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/ast.rs
 */

use crate::http::internals::{attr, check};
use crate::internals::Ctxt;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Token;
use syn::punctuated::Punctuated;

/// A source data structure annotated with `#[api_schema]`,
/// parsed into an internal representation.
pub struct Container<'a> {
    /// The struct or enum name (without generics).
    pub ident: syn::Ident,
    /// Attributes on the structure, parsed for OpenApi.
    pub attrs: attr::Container,
    /// The contents of the struct or enum.
    pub data: Data<'a>,
    /// Any generics on the struct or enum.
    pub generics: &'a syn::Generics,
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

        let mut data = match item {
            syn::DeriveInput {
                data: syn::Data::Enum(data),
                ..
            } => Data::Enum(enum_from_ast(cx, &mut data.variants)),
            syn::DeriveInput {
                data: syn::Data::Struct(data),
                ..
            } => {
                let style = struct_from_ast(cx, item, &mut data.fields);
                Data::Struct(style)
            }
            item => {
                cx.error_spanned_by(item, "Unions are not supported");
                return None;
            }
        };

        let mut item = Container {
            ident: item.ident.clone(),
            attrs,
            data,
            generics: &item.generics,
            original,
        };
        check::check(cx, &mut item);
        Some(item)
    }
}

fn enum_from_ast<'a>(
    cx: &Ctxt,
    variants: &'a mut Punctuated<syn::Variant, Token![,]>,
) -> Vec<Variant<'a>> {
    variants
        .iter_mut()
        .map(|variant| {
            let original = variant.clone();
            let attrs = attr::Variant::from_ast(cx, variant);
            let style = struct_from_ast(cx, variant.ident.clone(), &mut variant.fields);
            Variant {
                ident: variant.ident.clone(),
                attrs,
                style,
                original,
            }
        })
        .collect::<Vec<Variant>>()
}

fn struct_from_ast<'a, A: ToTokens>(cx: &Ctxt, item: A, fields: &'a mut syn::Fields) -> Style<'a> {
    match fields {
        syn::Fields::Named(_) => {
            cx.error_spanned_by(item, "Expected zero or one unnamed field.");
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
                cx.error_spanned_by(item, "Expected zero or one unnamed field.");
                Style::Unit
            }
        }
        syn::Fields::Unit => Style::Unit,
    }
}
