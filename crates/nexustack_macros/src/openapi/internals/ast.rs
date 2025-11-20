/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/ast.rs
 */

//! An OpenApi ast, parsed from the Syn ast and ready to generate Rust code.

use crate::internals::{Ctxt, default::Default};
use crate::openapi::internals::{attr, check};
use proc_macro2::TokenStream;
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
    Struct(Style, Vec<Field<'a>>),
}

/// A variant of an enum.
pub struct Variant<'a> {
    pub ident: syn::Ident,
    pub attrs: attr::Variant,
    pub style: Style,
    pub fields: Vec<Field<'a>>,
    pub original: syn::Variant,
}

/// A field of a struct.
pub struct Field<'a> {
    pub attrs: attr::Field,
    pub ty: &'a syn::Type,
    pub original: &'a syn::Field,
}

#[derive(Copy, Clone)]
pub enum Style {
    /// Named fields.
    Struct,
    /// Many unnamed fields.
    Tuple,
    /// One unnamed field.
    Newtype,
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
            } => Data::Enum(enum_from_ast(cx, &mut data.variants, attrs.default())),
            syn::DeriveInput {
                data: syn::Data::Struct(data),
                ..
            } => {
                let (style, fields) = struct_from_ast(cx, &mut data.fields, attrs.default());
                Data::Struct(style, fields)
            }
            item => {
                cx.error_spanned_by(item, "Unions are not supported");
                return None;
            }
        };

        match &mut data {
            Data::Enum(variants) => {
                for variant in variants {
                    variant.attrs.rename_by_rules(attrs.rename_all_rules());
                    for field in &mut variant.fields {
                        field.attrs.rename_by_rules(
                            variant
                                .attrs
                                .rename_all_rules()
                                .or(attrs.rename_all_fields_rules()),
                        );
                    }
                }
            }
            Data::Struct(_, fields) => {
                for field in fields {
                    field.attrs.rename_by_rules(attrs.rename_all_rules());
                }
            }
        }

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

impl<'a> Data<'a> {
    pub fn all_fields(&'a self) -> Box<dyn Iterator<Item = &'a Field<'a>> + 'a> {
        match self {
            Data::Enum(variants) => {
                Box::new(variants.iter().flat_map(|variant| variant.fields.iter()))
            }
            Data::Struct(_, fields) => Box::new(fields.iter()),
        }
    }
}

fn enum_from_ast<'a>(
    cx: &Ctxt,
    variants: &'a mut Punctuated<syn::Variant, Token![,]>,
    container_default: &Default,
) -> Vec<Variant<'a>> {
    let variants: Vec<Variant> = variants
        .iter_mut()
        .map(|variant| {
            let original = variant.clone();
            let attrs = attr::Variant::from_ast(cx, variant);
            let (style, fields) = struct_from_ast(cx, &mut variant.fields, container_default);
            Variant {
                ident: variant.ident.clone(),
                attrs,
                style,
                fields,
                original,
            }
        })
        .collect();

    let index_of_last_tagged_variant = variants
        .iter()
        .rposition(|variant| !variant.attrs.untagged());
    if let Some(index_of_last_tagged_variant) = index_of_last_tagged_variant {
        for variant in &variants[..index_of_last_tagged_variant] {
            if variant.attrs.untagged() {
                cx.error_spanned_by(&variant.ident, "all variants with the #[api_variant(untagged)] attribute must be placed at the end of the enum");
            }
        }
    }

    variants
}

fn struct_from_ast<'a>(
    cx: &Ctxt,
    fields: &'a mut syn::Fields,
    container_default: &Default,
) -> (Style, Vec<Field<'a>>) {
    match fields {
        syn::Fields::Named(fields) => (
            Style::Struct,
            fields_from_ast(cx, &mut fields.named, container_default),
        ),
        syn::Fields::Unnamed(fields) => {
            if fields.unnamed.len() == 1 {
                (
                    Style::Newtype,
                    fields_from_ast(cx, &mut fields.unnamed, container_default),
                )
            } else {
                (
                    Style::Tuple,
                    fields_from_ast(cx, &mut fields.unnamed, container_default),
                )
            }
        }
        syn::Fields::Unit => (Style::Unit, Vec::new()),
    }
}

fn fields_from_ast<'a>(
    cx: &Ctxt,
    fields: &'a mut Punctuated<syn::Field, Token![,]>,
    container_default: &Default,
) -> Vec<Field<'a>> {
    fields
        .iter_mut()
        .enumerate()
        .map(|(i, field)| Field {
            attrs: attr::Field::from_ast(cx, i, field, container_default),
            ty: &field.ty,
            original: field,
        })
        .collect()
}
