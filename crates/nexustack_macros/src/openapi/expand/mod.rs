/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://github.com/serde-rs/serde/blob/master/serde_derive/src/ser.rs
 */

mod r#enum;
mod from_into;
mod newtype_struct;
mod r#struct;
mod transparent;
mod tuple_struct;
mod unit_struct;

use crate::{
    internals::{Ctxt, IntoIteratorExt, replace_receiver},
    openapi::{
        bound, dummy,
        internals::{
            Derive,
            ast::{Container, Data, Field, Style, Variant},
            attr,
        },
        serde::build_cont_attribute,
    },
};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote, quote_spanned};
use syn::{Ident, parse_quote, spanned::Spanned};

pub fn expand_api_schema(
    attr: TokenStream,
    input: &mut syn::DeriveInput,
) -> syn::Result<TokenStream> {
    replace_receiver(input);

    let ctxt = Ctxt::new();
    let cont = match Container::from_ast(&ctxt, attr, input) {
        Some(cont) => cont,
        None => return Err(ctxt.check().unwrap_err()),
    };
    precondition(&ctxt, &cont);
    ctxt.check()?;

    let impl_block = if cont.attrs.transparent() {
        transparent::expand_transparent(&cont)
    } else if let Some(type_into) = cont.attrs.type_into() {
        from_into::expand_from_into(&cont, type_into)
    } else if let Some(type_from) = cont.attrs.type_from() {
        from_into::expand_from_into(&cont, type_from)
    } else if let Some(type_try_from) = cont.attrs.type_try_from() {
        from_into::expand_from_into(&cont, type_try_from)
    } else {
        match &cont.data {
            Data::Enum(variants) => r#enum::expand_enum(&cont, variants),
            Data::Struct(Style::Struct, fields) => r#struct::expand_struct(&cont, fields),
            Data::Struct(Style::Tuple, fields) => {
                let single_non_skipped_field = fields
                    .iter()
                    .filter(|field| !field.attrs.skip())
                    .exactly_one();

                if let Some(single_non_skipped_field) = single_non_skipped_field {
                    newtype_struct::expand_newtype_struct(&cont, single_non_skipped_field)
                } else {
                    tuple_struct::expand_tuple_struct(&cont, fields)
                }
            }
            Data::Struct(Style::Newtype, fields) => {
                newtype_struct::expand_newtype_struct(&cont, &fields[0])
            }
            Data::Struct(Style::Unit, _) => unit_struct::expand_unit_struct(&cont),
        }
    };

    let serde = cont.attrs.serde_path();
    let container_serde_attr = build_cont_attribute(&cont);

    let impl_block = dummy::wrap_in_const(
        cont.attrs.custom_serde_path(),
        cont.attrs.custom_crate_path(),
        impl_block,
    );

    let serde_derive = match cont.attrs.derive() {
        Derive::Write => quote! {
            #[derive(#serde::Serialize)]
            #container_serde_attr
        },
        Derive::Read => quote! {
            #[derive(#serde::Deserialize)]
            #container_serde_attr
        },
        Derive::ReadWrite => quote! {
            #[derive(#serde::Serialize, #serde::Deserialize)]
            #container_serde_attr
        },
    };

    Ok(quote! {
        #serde_derive
        #input
        #impl_block
    })
}

struct ExampleContainerIdentifier {
    ident: Ident,
    generic_args: Vec<TokenStream>,
}

impl ToTokens for ExampleContainerIdentifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        let generic_args = &self.generic_args;

        if generic_args.is_empty() {
            quote! { #ident }.to_tokens(tokens);
        } else {
            quote! { #ident <#(#generic_args),*> }.to_tokens(tokens);
        };
    }
}

impl ExampleContainerIdentifier {
    fn instantiation(&self) -> TokenStream {
        let ident = &self.ident;
        let generic_args = &self.generic_args;

        if generic_args.is_empty() {
            quote! { #ident }
        } else {
            quote! { #ident::<#(#generic_args),*> }
        }
    }
}

fn precondition(cx: &Ctxt, cont: &Container) {
    match cont.attrs.identifier() {
        attr::Identifier::No => {}
        attr::Identifier::Field => {
            cx.error_spanned_by(&cont.original, "field identifiers cannot be serialized");
        }
        attr::Identifier::Variant => {
            cx.error_spanned_by(&cont.original, "variant identifiers cannot be serialized");
        }
    }
}

struct Parameters {
    /// Generics including any explicit and inferred bounds for the impl.
    generics: syn::Generics,
}

impl Parameters {
    fn new(cont: &Container) -> Self {
        let generics = build_generics(cont);

        Parameters { generics }
    }
}

// All the generics in the input, plus a bound `T: Schema` for each generic
// field type that will be handled by us.
fn build_generics(cont: &Container) -> syn::Generics {
    let mut generics = bound::without_defaults(cont.generics);
    let mut default_bound_checked = false;

    if cont.attrs.derive().read() {
        generics = bound::with_where_predicates_from_fields(cont, &generics, attr::Field::de_bound);

        generics =
            bound::with_where_predicates_from_variants(cont, &generics, attr::Variant::de_bound);

        if let Some(predicates) = cont.attrs.de_bound() {
            generics = bound::with_where_predicates(&generics, predicates);
        } else {
            default_bound_checked = true;
            generics = bound::with_bound(
                cont,
                &generics,
                needs_describe_bound,
                &parse_quote!(_nexustack::openapi::Schema),
            );
        }
    }

    if cont.attrs.derive().write() {
        generics =
            bound::with_where_predicates_from_fields(cont, &generics, attr::Field::ser_bound);

        generics =
            bound::with_where_predicates_from_variants(cont, &generics, attr::Variant::ser_bound);

        if let Some(predicates) = cont.attrs.ser_bound() {
            generics = bound::with_where_predicates(&generics, predicates);
        } else if !default_bound_checked {
            generics = bound::with_bound(
                cont,
                &generics,
                needs_describe_bound,
                &parse_quote!(_nexustack::openapi::Schema),
            );
        }
    }

    generics
}

// Fields with a `skip` attribute, or which
// belong to a variant with a `skip` attribute,
// are not handled by us so we do not generate a bound. Fields with a `bound`
// attribute specify their own bound so we do not generate one. All other fields
// may need a `T: Schema` bound where T is the type of the field.
fn needs_describe_bound(field: &attr::Field, variant: Option<&attr::Variant>) -> bool {
    !field.skip()
        && field.ser_bound().is_none()
        && variant.is_none_or(|variant| !variant.skip() && variant.ser_bound().is_none())
}

fn describe_tuple_struct_visitor<'a>(
    fields: impl IntoIterator<Item = &'a Field<'a>>,
    tuple_trait: &TupleTrait,
) -> Vec<TokenStream> {
    fields
        .into_iter()
        .map(|field| {
            let skip = field.attrs.skip();

            if skip {
                // TODO: Is this correct?
                TokenStream::new()
            } else {
                let span = field.original.span();
                let func = tuple_trait.describe_element(span);
                let description = field.attrs.description();
                let deprecated = field.attrs.deprecated();
                let ty = field.ty;
                quote! {
                    #func(
                        &mut __builder,
                        _nexustack::__private::Option::Some(#description),
                        #deprecated,
                        <#ty as _nexustack::openapi::Schema>::describe
                    )?;
                }
            }
        })
        .collect()
}

fn describe_struct_visitor<'a>(
    fields: impl IntoIterator<Item = &'a Field<'a>>,
    cattrs: &attr::Container,
    struct_trait: &StructTrait,
) -> Vec<TokenStream> {
    fields
        .into_iter()
        .map(|field| {
            let key_expr = field.attrs.name().serialize_name();
            let span = field.original.span();
            let ty = field.ty;

            if field.attrs.flatten() {
                let schema_ty = quote_spanned!(span => <#ty as _nexustack::openapi::Schema>);
                return quote! {
                    #schema_ty::describe(_nexustack::openapi::__private::FlatMapSchemaBuilder(&mut __builder))?;
                };
            }

            // if field.attrs.skip() {
            //     if let Some(skip_func) = struct_trait.skip_field(span) {
            //         return quote! {
            //             #skip_func(&mut __builder, #key_expr)?;
            //         };
            //     } else {
            //         // TODO: Is this correct?
            //         return TokenStream::new();
            //     }
            // }

            let description = field.attrs.description();
            let deprecated = field.attrs.deprecated();

            let default = match field.attrs.default().or(cattrs.default()) {
                attr::Default::None => None,
                attr::Default::Default => Some(quote!(<#ty as _nexustack::__private::Default>::default())),
                attr::Default::Path(expr_path) => Some(quote!(#expr_path())),
            };

            if let Some(default) = default {
                let func = struct_trait.describe_field_optional(span);
                quote! {
                    #func(
                        &mut __builder,
                        #key_expr,
                        _nexustack::openapi::FieldMod::ReadWrite,
                        _nexustack::__private::Option::Some(#default),
                        _nexustack::__private::Option::Some(#description),
                        #deprecated,
                        <#ty as _nexustack::openapi::Schema>::describe,
                    )?;
                }
            } else {
                let func = struct_trait.describe_field(span);
                quote! {
                    #func(
                        &mut __builder,
                        #key_expr,
                        _nexustack::openapi::FieldMod::ReadWrite,
                        _nexustack::__private::Option::Some(#description),
                        #deprecated,
                        <#ty as _nexustack::openapi::Schema>::describe,
                    )?;
                }
            }
        })
        .collect()
}

// where we want to omit the `mut` to avoid a warning.
fn mut_if(is_mut: bool) -> Option<TokenStream> {
    if is_mut { Some(quote!(mut)) } else { None }
}

fn effective_style(variant: &Variant) -> Style {
    match variant.style {
        Style::Newtype if variant.fields[0].attrs.skip() => Style::Unit,
        other => other,
    }
}

enum StructTrait {
    Map,
    Struct,
    StructVariant,
}

impl StructTrait {
    fn describe_field(&self, span: Span) -> TokenStream {
        match *self {
            StructTrait::Map => {
                quote_spanned!(span => _nexustack::openapi::MapSchemaBuilder::collect_element)
            }
            StructTrait::Struct => {
                quote_spanned!(span => _nexustack::openapi::StructSchemaBuilder::collect_field)
            }
            StructTrait::StructVariant => {
                quote_spanned!(span => _nexustack::openapi::StructVariantSchemaBuilder::collect_field)
            }
        }
    }

    fn describe_field_optional(&self, span: Span) -> TokenStream {
        match *self {
            StructTrait::Map => {
                quote_spanned!(span => _nexustack::openapi::MapSchemaBuilder::collect_element_optional)
            }
            StructTrait::Struct => {
                quote_spanned!(span => _nexustack::openapi::StructSchemaBuilder::collect_field_optional)
            }
            StructTrait::StructVariant => {
                quote_spanned!(span => _nexustack::openapi::StructVariantSchemaBuilder::collect_field_optional)
            }
        }
    }
}

enum TupleTrait {
    TupleStruct,
    TupleVariant,
}

impl TupleTrait {
    fn describe_element(&self, span: Span) -> TokenStream {
        match *self {
            TupleTrait::TupleStruct => {
                quote_spanned!(span => _nexustack::openapi::TupleStructSchemaBuilder::collect_field)
            }
            TupleTrait::TupleVariant => {
                quote_spanned!(span => _nexustack::openapi::TupleVariantSchemaBuilder::collect_field)
            }
        }
    }
}
