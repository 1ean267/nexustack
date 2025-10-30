/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://github.com/serde-rs/serde/blob/master/serde_derive/src/ser.rs
 */

use crate::{
    fragment::{Fragment, Stmts},
    internals::callsite,
    openapi::{
        expand::{
            ExampleContainerIdentifier, Parameters, StructTrait, TupleTrait,
            describe_struct_visitor, describe_tuple_struct_visitor, effective_style, mut_if,
        },
        generics::{field_contains_generic_params, make_lifetimes_static},
        internals::{
            ast::{Container, Field, Style, Variant},
            attr,
        },
        serde::{
            build_example_enum_attribute, build_example_field_attribute,
            build_example_variant_attribute,
        },
    },
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;

pub fn expand_enum<'a>(cont: &Container, variants: &[Variant<'a>]) -> TokenStream {
    let ident = &cont.ident;
    let params = Parameters::new(cont);
    let (impl_generics, ty_generics, where_clause) = params.generics.split_for_impl();
    let (example_cont, example_cont_id) = example_container(variants, cont);
    let body = Stmts(describe(variants, cont, &example_cont_id));
    let examples = examples_type(variants, &example_cont_id);

    quote! {
        #[automatically_derived]
        #example_cont

        #[automatically_derived]
        impl #impl_generics _nexustack::openapi::Schema for #ident #ty_generics #where_clause {
            type Example = <Self::Examples as Iterator>::Item;
            type Examples = #examples;

            fn describe<__B>(__schema_builder: __B) -> _nexustack::__private::Result<__B::Ok, __B::Error>
            where
                __B: _nexustack::openapi::SchemaBuilder<Self::Examples>,
            {
                #body
            }
        }
    }
}

fn example_container(
    variants: &[Variant],
    cont: &Container,
) -> (TokenStream, ExampleContainerIdentifier) {
    let example_cont_ident = format_ident!("__{}_Example", cont.ident);
    let serde_attr = build_example_enum_attribute(cont);

    let mut where_clause = vec![];
    let mut generic_params = vec![];
    let mut generic_args = vec![];
    let mut g_variants = Vec::with_capacity(variants.len());

    for variant in variants {
        if variant.attrs.skip() || variant.attrs.other() {
            continue;
        }

        let ident = &variant.ident;
        let serde_variant_attr = build_example_variant_attribute(variant);

        let mut g_fields = Vec::with_capacity(variant.fields.len());

        for (field_idx, field) in variant.fields.iter().enumerate() {
            if field.attrs.skip() {
                continue;
            }

            let mut ty = field.ty.clone();
            make_lifetimes_static(&mut ty);
            let serde_field_attr = build_example_field_attribute(field);

            if field_contains_generic_params(field, cont) {
                let generic_param = format_ident!("T__{}__{}_Example", ident, field_idx);
                where_clause.push(quote! { #generic_param: _serde::Serialize + 'static });
                generic_args.push(quote! { <#ty as _nexustack::openapi::Schema>::Example });

                match variant.style {
                    Style::Unit => unreachable!("Unit variants have no fields"),
                    Style::Newtype => {
                        g_fields.push(quote! {
                            #serde_field_attr
                            #generic_param
                        });
                    }
                    Style::Tuple => {
                        g_fields.push(quote! {
                            #serde_field_attr
                            #generic_param
                        });
                    }
                    Style::Struct => {
                        let ident = field
                            .original
                            .ident
                            .as_ref()
                            .expect("Brace structs have named fields");

                        g_fields.push(quote! {
                            #serde_field_attr #ident:
                            #generic_param
                        });
                    }
                }

                generic_params.push(generic_param);
            } else {
                match variant.style {
                    Style::Unit => unreachable!("Unit variants have no fields"),
                    Style::Newtype => {
                        g_fields.push(quote! {
                            #serde_field_attr
                            <#ty as _nexustack::openapi::Schema>::Example
                        });
                    }
                    Style::Tuple => {
                        g_fields.push(quote! {
                            #serde_field_attr
                            <#ty as _nexustack::openapi::Schema>::Example
                        });
                    }
                    Style::Struct => {
                        let ident = field
                            .original
                            .ident
                            .as_ref()
                            .expect("Brace structs have named fields");

                        g_fields.push(quote! {
                            #serde_field_attr
                            #ident: <#ty as _nexustack::openapi::Schema>::Example
                        });
                    }
                }
            }
        }

        match variant.style {
            Style::Unit => {
                g_variants.push(quote! {
                    #serde_variant_attr
                    #ident
                });
            }
            Style::Newtype => {
                let g_field = &g_fields[0];
                g_variants.push(quote! {
                    #serde_variant_attr
                    #ident ( #g_field )
                });
            }
            Style::Tuple => {
                g_variants.push(quote! {
                    #serde_variant_attr
                    #ident ( #(#g_fields),* )
                });
            }
            Style::Struct => {
                g_variants.push(quote! {
                    #serde_variant_attr
                    #ident { #(#g_fields),* }
                });
            }
        }
    }

    let generic_params = if generic_params.is_empty() {
        TokenStream::new()
    } else {
        quote! { <#(#generic_params),*> }
    };

    let where_clause = if where_clause.is_empty() {
        quote!()
    } else {
        quote! ( where #(#where_clause),* )
    };

    (
        quote! {
            #[derive(_serde::Serialize)]
            #serde_attr
            pub enum #example_cont_ident #generic_params #where_clause { #(#g_variants),* }
        },
        ExampleContainerIdentifier {
            ident: example_cont_ident,
            generic_args,
        },
    )
}

fn examples_type(variants: &[Variant], example_cont: &ExampleContainerIdentifier) -> TokenStream {
    variants
        .iter()
        .filter(|variant| !variant.attrs.skip() && !variant.attrs.other())
        .map(|variant| variant_example_type(variant, example_cont))
        .reduce(|acc, n| quote! { _nexustack::__private::Chain<#acc, #n> })
        .expect("Enum must contain at least one non-skipped variant.")
}

fn variant_example_type(
    variant: &Variant<'_>,
    example_cont: &ExampleContainerIdentifier,
) -> TokenStream {
    match variant.style {
        Style::Unit => {
            quote!(_nexustack::__private::Once<#example_cont>)
        }
        Style::Newtype => {
            let field = &variant.fields[0];
            let ty = field.ty;
            quote! {
                _nexustack::__private::Map<
                    <#ty as _nexustack::openapi::Schema> :: Examples,
                    fn(<#ty as _nexustack::openapi::Schema>::Example) -> #example_cont,
                >
            }
        }
        Style::Tuple => tuple_variant_examples_type(&variant.fields, example_cont),
        Style::Struct => struct_variant_examples_type(&variant.fields, example_cont),
    }
}

// TODO: Stolen from tuple_struct
fn tuple_variant_examples_type(
    fields: &[Field],
    example_cont: &ExampleContainerIdentifier,
) -> TokenStream {
    let fields = fields
        .iter()
        .filter(|field| !field.attrs.skip())
        .collect::<Vec<_>>();

    // This should never occur
    if fields.is_empty() {
        return quote!(_nexustack::__private::Once<#example_cont>);
    }

    // This should never occur
    if fields.len() == 1 {
        let ty = fields[0].ty;

        return quote! {
            _nexustack::__private::Map<
                <#ty as _nexustack::openapi::Schema>::Examples,
                fn(<#ty as _nexustack::openapi::Schema>::Example) -> #example_cont
            >
        };
    }

    let field_ty_0 = fields[0].ty;
    let field_ty_1 = fields[1].ty;

    let mut examples_iter_ty = quote! { _nexustack::__private::Zip<<#field_ty_0 as _nexustack::openapi::Schema>::Examples, <#field_ty_1 as _nexustack::openapi::Schema>::Examples> };
    let mut example_ty_list = vec![
        quote! { <#field_ty_0 as _nexustack::openapi::Schema>::Example },
        quote! { <#field_ty_1 as _nexustack::openapi::Schema>::Example },
    ];

    for field in fields.iter().skip(2) {
        let field_ty = field.ty;

        examples_iter_ty = quote! { _nexustack::__private::Zip<#examples_iter_ty, <#field_ty as _nexustack::openapi::Schema>::Examples> };
        examples_iter_ty = quote! {
            _nexustack::__private::Map<
                #examples_iter_ty,
                fn(((#(#example_ty_list),*), <#field_ty as _nexustack::openapi::Schema>::Example)) -> (#(#example_ty_list),*, <#field_ty as _nexustack::openapi::Schema>::Example),
            >
        };
        example_ty_list.push(quote! { <#field_ty as _nexustack::openapi::Schema>::Example });
    }

    quote! {
        _nexustack::__private::Map<#examples_iter_ty, fn((#(#example_ty_list),*)) -> #example_cont>
    }
}

// TODO: Stolen from struct
fn struct_variant_examples_type(
    fields: &[Field],
    example_cont: &ExampleContainerIdentifier,
) -> TokenStream {
    let fields = fields
        .iter()
        .filter(|field| !field.attrs.skip())
        .collect::<Vec<_>>();

    if fields.is_empty() {
        return quote! { _nexustack::__private::Once<#example_cont> };
    }

    if fields.len() == 1 {
        let ty = fields[0].ty;

        return quote! {
            _nexustack::__private::Map<<#ty as _nexustack::openapi::Schema>::Examples, fn(<#ty as _nexustack::openapi::Schema>::Example) -> #example_cont>
        };
    }

    let field_ty_0 = fields[0].ty;
    let field_ty_1 = fields[1].ty;

    let mut examples_iter_ty = quote! { _nexustack::__private::Zip<<#field_ty_0 as _nexustack::openapi::Schema>::Examples, <#field_ty_1 as _nexustack::openapi::Schema>::Examples> };
    let mut example_ty_list = vec![
        quote! { <#field_ty_0 as _nexustack::openapi::Schema>::Example },
        quote! { <#field_ty_1 as _nexustack::openapi::Schema>::Example },
    ];

    for field in fields.iter().skip(2) {
        let field_ty = field.ty;

        examples_iter_ty = quote! { _nexustack::__private::Zip<#examples_iter_ty, <#field_ty as _nexustack::openapi::Schema>::Examples> };
        examples_iter_ty = quote! { _nexustack::__private::Map<#examples_iter_ty, fn(((#(#example_ty_list),*), <#field_ty as _nexustack::openapi::Schema>::Example)) -> (#(#example_ty_list),*, <#field_ty as _nexustack::openapi::Schema>::Example)> };
        example_ty_list.push(quote! { <#field_ty as _nexustack::openapi::Schema>::Example });
    }

    quote! {
        _nexustack::__private::Map<#examples_iter_ty, fn((#(#example_ty_list),*)) -> #example_cont>
    }
}

fn examples(variants: &[Variant], example_cont: &ExampleContainerIdentifier) -> TokenStream {
    variants
        .iter()
        .filter(|variant| !variant.attrs.skip() && !variant.attrs.other())
        .map(|variant| variant_example(variant, example_cont))
        .reduce(|acc, n| quote! { _nexustack::__private::Iterator::chain(#acc, #n) })
        .expect("Enum must contain at least one non-skipped variant.")
}

fn variant_example(
    variant: &Variant<'_>,
    example_cont: &ExampleContainerIdentifier,
) -> TokenStream {
    let example_cont_instantiation = example_cont.instantiation();
    let variant_ident = &variant.ident;
    match variant.style {
        Style::Unit => {
            quote!(_nexustack::__private::once(#example_cont_instantiation :: #variant_ident))
        }
        Style::Newtype => {
            let field = &variant.fields[0];
            let ty = field.ty;
            quote! {
                _nexustack::__private::Iterator::map(
                    <#ty as _nexustack::openapi::SchemaExamples>::examples(is_human_readable)?,
                    (|e: <#ty as _nexustack::openapi::Schema>::Example| #example_cont_instantiation :: #variant_ident(e)) as _,
                )
            }
        }
        Style::Tuple => {
            tuple_variant_example(&variant.fields, example_cont_instantiation, variant_ident)
        }
        Style::Struct => {
            struct_variant_example(&variant.fields, example_cont_instantiation, variant_ident)
        }
    }
}

// TODO: Stolen from tuple_struct
fn tuple_variant_example(
    fields: &[Field],
    example_cont_instantiation: TokenStream,
    variant_ident: &syn::Ident,
) -> TokenStream {
    let fields = fields
        .iter()
        .filter(|field| !field.attrs.skip())
        .collect::<Vec<_>>();

    // This should never occur
    if fields.is_empty() {
        return quote!(_nexustack::__private::once(#example_cont_instantiation :: #variant_ident()));
    }

    // This should never occur
    if fields.len() == 1 {
        let ty = fields[0].ty;

        return quote! {
            _nexustack::__private::Iterator::map(
                <#ty as _nexustack::openapi::SchemaExamples>::examples(is_human_readable)?,
                (|e: <#ty as _nexustack::openapi::Schema>::Example| #example_cont_instantiation :: #variant_ident(e)) as _,
            )
        };
    }

    let field_ty_0 = fields[0].ty;
    let field_ty_1 = fields[1].ty;

    let mut result = quote! {
        _nexustack::__private::Iterator::zip(
            <#field_ty_0 as _nexustack::openapi::SchemaExamples>::examples(is_human_readable)?,
            <#field_ty_1 as _nexustack::openapi::SchemaExamples>::examples(is_human_readable)?,
        )
    };

    let mut example_ty_list = vec![
        quote! { <#field_ty_0 as _nexustack::openapi::Schema>::Example },
        quote! { <#field_ty_1 as _nexustack::openapi::Schema>::Example },
    ];

    for (index, field) in fields.iter().enumerate().skip(2) {
        let field_ty = field.ty;

        result = quote! {
            _nexustack::__private::Iterator::zip(
                #result,
                <#field_ty as _nexustack::openapi::SchemaExamples>::examples(is_human_readable)?,
            )
        };

        let f_existing_entries = (0..index).map(syn::Index::from).map(|i| quote! { f.0.#i });

        result = quote! {
            _nexustack::__private::Iterator::map(
                #result,
                (|f: ((#(#example_ty_list),*), <#field_ty as _nexustack::openapi::Schema>::Example)| (#(#f_existing_entries),*, f.1)) as _,
            )
        };

        example_ty_list.push(quote! { <#field_ty as _nexustack::openapi::Schema>::Example });
    }

    let f_entries = (0..fields.len())
        .map(syn::Index::from)
        .map(|i| quote! { f.#i });
    quote! {
        _nexustack::__private::Iterator::map(
            #result,
            (|f: (#(#example_ty_list),*)| #example_cont_instantiation :: #variant_ident(#(#f_entries),*)) as _,
        )
    }
}

// TODO: Stolen from struct
fn struct_variant_example(
    fields: &[Field],
    example_cont_instantiation: TokenStream,
    variant_ident: &syn::Ident,
) -> TokenStream {
    let fields = fields
        .iter()
        .filter(|field| !field.attrs.skip())
        .collect::<Vec<_>>();

    if fields.is_empty() {
        return quote!(_nexustack::__private::once(#example_cont_instantiation :: #variant_ident { }));
    }

    if fields.len() == 1 {
        let ty = fields[0].ty;
        let ident = fields[0]
            .original
            .ident
            .as_ref()
            .expect("Brace structs have named fields");

        return quote! {
            _nexustack::__private::Iterator::map(
                <#ty as _nexustack::openapi::SchemaExamples>::examples(is_human_readable)?,
                (|e: <#ty as _nexustack::openapi::Schema>::Example| #example_cont_instantiation :: #variant_ident { #ident: e }) as _,
            )
        };
    }

    let field_ty_0 = fields[0].ty;
    let field_ty_1 = fields[1].ty;

    let mut result = quote! {
        _nexustack::__private::Iterator::zip(
            <#field_ty_0 as _nexustack::openapi::SchemaExamples>::examples(is_human_readable)?,
            <#field_ty_1 as _nexustack::openapi::SchemaExamples>::examples(is_human_readable)?,
        )
    };

    let mut example_ty_list = vec![
        quote! { <#field_ty_0 as _nexustack::openapi::Schema>::Example },
        quote! { <#field_ty_1 as _nexustack::openapi::Schema>::Example },
    ];

    for (index, field) in fields.iter().enumerate().skip(2) {
        let field_ty = field.ty;

        result = quote! {
            _nexustack::__private::Iterator::zip(
                #result,
                <#field_ty as _nexustack::openapi::SchemaExamples>::examples(is_human_readable)?,
            )
        };

        let f_existing_entries = (0..index).map(syn::Index::from).map(|i| quote! { f.0.#i });

        result = quote! {
            _nexustack::__private::Iterator::map(
                #result,
                (|f:((#(#example_ty_list),*), <#field_ty as _nexustack::openapi::Schema>::Example)| (#(#f_existing_entries),*, f.1)) as _,
            )
        };

        example_ty_list.push(quote! { <#field_ty as _nexustack::openapi::Schema>::Example });
    }

    let f_entries = fields
        .iter()
        .enumerate()
        .map(|(index, field)| (syn::Index::from(index), field))
        .map(|(index, field)| {
            let ident = field
                .original
                .ident
                .as_ref()
                .expect("Brace structs have named fields");
            quote! { #ident: f.#index }
        });

    quote! {
        _nexustack::__private::Iterator::map(
            #result,
            (|f: (#(#example_ty_list),*)| #example_cont_instantiation :: #variant_ident{ #(#f_entries),* }) as _,
        )
    }
}

fn describe(
    variants: &[Variant],
    cont: &Container,
    example_cont: &ExampleContainerIdentifier,
) -> Fragment {
    let cattrs = &cont.attrs;
    assert!(variants.len() as u64 <= u64::from(u32::MAX));

    let has_other = variants
        .iter()
        .any(|variant| !variant.attrs.skip() && variant.attrs.other());

    let describe_variants = variants
        .iter()
        .filter(|variant| !variant.attrs.skip() && !variant.attrs.other())
        .enumerate()
        .map(|(variant_index, variant)| describe_variant(variant, variant_index as u32, cattrs))
        .collect::<Vec<_>>();

    let mut serialized_variants = variants
        .iter()
        .filter(|variant| !variant.attrs.skip() && !variant.attrs.other())
        .peekable();

    let let_mut = mut_if(serialized_variants.peek().is_some());
    let type_name = cattrs.name().serialize_name();

    let len = serialized_variants
        .map(|_| quote!(1))
        .fold(quote!(0), |sum, expr| quote!(#sum + #expr));

    let exhaustive = !cattrs.non_exhaustive() && !has_other;

    let tag = match cattrs.tag() {
        attr::TagType::External => quote!(_nexustack::openapi::VariantTag::ExternallyTagged),
        attr::TagType::Internal { tag } => {
            quote!(_nexustack::openapi::VariantTag::InternallyTagged { tag: #tag })
        }
        attr::TagType::Adjacent { tag, content } => {
            quote!(_nexustack::openapi::VariantTag::AdjacentlyTagged { tag: #tag, content: #content })
        }
        attr::TagType::None => quote!(_nexustack::openapi::VariantTag::Untagged),
    };

    let description = cattrs.description();
    let deprecated = cattrs.deprecated();
    let examples = examples(variants, example_cont);

    // If any of the fields contains one of the containers generic parameters,
    // we cannot provide a unique type name.
    let id = if variants
        .iter()
        .filter(|variant| !variant.attrs.skip() && !variant.attrs.other())
        .any(|variant| {
            variant
                .fields
                .iter()
                .any(|field| !field.attrs.skip() && field_contains_generic_params(field, cont))
        }) {
        quote! { _nexustack::__private::Option::None }
    } else {
        let cont_span = cont.original.span();
        let cont_callsite = callsite(&cont_span);

        quote! {
            _nexustack::__private::Option::Some(_nexustack::openapi::SchemaId::new(#type_name, #cont_callsite))
        }
    };

    quote_block! {
        let is_human_readable = _nexustack::openapi::SchemaBuilder::is_human_readable(&__schema_builder);
        let #let_mut __enum_builder = _nexustack::openapi::SchemaBuilder::describe_enum(
            __schema_builder,
            #id,
            #len,
            #exhaustive,
            #tag,
            _nexustack::__private::Option::Some(#description),
            || Ok(#examples),
            #deprecated
        )?;
        #(#describe_variants)*
        _nexustack::openapi::EnumSchemaBuilder::end(__enum_builder)
    }
}

fn describe_variant(
    variant: &Variant,
    variant_index: u32,
    cattrs: &attr::Container,
) -> TokenStream {
    let body = Stmts(match (cattrs.tag(), variant.attrs.untagged()) {
        (attr::TagType::External, false) => {
            describe_externally_tagged_variant(variant, variant_index, cattrs)
        }
        (attr::TagType::Internal { .. }, false) => {
            describe_internally_tagged_variant(variant, cattrs, variant_index)
        }
        (attr::TagType::Adjacent { .. }, false) => {
            describe_adjacently_tagged_variant(variant, cattrs, variant_index)
        }
        (attr::TagType::None, _) | (_, true) => {
            describe_untagged_variant(variant, cattrs, variant_index)
        }
    });

    quote! {
        #body
    }
}

fn describe_externally_tagged_variant(
    variant: &Variant,
    variant_index: u32,
    cattrs: &attr::Container,
) -> Fragment {
    let variant_name = variant.attrs.name().serialize_name();
    let variant_span = variant.original.span();
    let variant_callsite = callsite(&variant_span);
    let variant_id =
        quote! { _nexustack::openapi::SchemaId::new(#variant_name, #variant_callsite) };
    let description = variant.attrs.description();
    let deprecated = variant.attrs.deprecated();

    match effective_style(variant) {
        Style::Unit => quote_block! {
            _nexustack::openapi::EnumSchemaBuilder::describe_unit_variant(
                &mut __enum_builder,
                #variant_index,
                #variant_id,
                Some(#description),
                #deprecated,
            )?;
        },
        Style::Newtype => {
            let field = &variant.fields[0];

            let span = field.original.span();
            let ty = field.ty;
            let func = quote_spanned!(span =>  _nexustack::openapi::EnumSchemaBuilder::collect_newtype_variant);
            quote_block! {
                #func(
                    &mut __enum_builder,
                    #variant_index,
                    #variant_id,
                    Some(#description),
                    #deprecated,
                    <#ty as _nexustack::openapi::Schema>::describe
                )?;
            }
        }
        Style::Tuple => describe_tuple_variant(variant, &variant.fields, variant_index),
        Style::Struct => describe_struct_variant(variant, cattrs, &variant.fields, variant_index),
    }
}

fn describe_internally_tagged_variant(
    variant: &Variant,
    cattrs: &attr::Container,
    variant_index: u32,
) -> Fragment {
    let variant_name = variant.attrs.name().serialize_name();
    let variant_span = variant.original.span();
    let variant_callsite = callsite(&variant_span);
    let variant_id =
        quote! { _nexustack::openapi::SchemaId::new(#variant_name, #variant_callsite) };
    let description = variant.attrs.description();
    let deprecated = variant.attrs.deprecated();

    match effective_style(variant) {
        Style::Unit => quote_block! {
            _nexustack::openapi::EnumSchemaBuilder::describe_unit_variant(
                &mut __enum_builder,
                #variant_index,
                #variant_id,
                Some(#description),
                #deprecated,
            )?;
        },
        Style::Newtype => {
            let field = &variant.fields[0];

            let span = field.original.span();
            let ty = field.ty;
            let func = quote_spanned!(span =>  _nexustack::openapi::EnumSchemaBuilder::collect_newtype_variant);
            quote_block! {
                #func(
                    &mut __enum_builder,
                    #variant_index,
                    #variant_id,
                    Some(#description),
                    #deprecated,
                    <#ty as _nexustack::openapi::Schema>::describe
                )?;
            }
        }
        Style::Struct => describe_struct_variant(variant, cattrs, &variant.fields, variant_index),
        Style::Tuple => unreachable!("checked in internals/check"),
    }
}

fn describe_adjacently_tagged_variant(
    variant: &Variant,
    cattrs: &attr::Container,
    variant_index: u32,
) -> Fragment {
    let variant_name = variant.attrs.name().serialize_name();
    let variant_span = variant.original.span();
    let variant_callsite = callsite(&variant_span);
    let variant_id =
        quote! { _nexustack::openapi::SchemaId::new(#variant_name, #variant_callsite) };
    let description = variant.attrs.description();
    let deprecated = variant.attrs.deprecated();

    match effective_style(variant) {
        Style::Unit => quote_block! {
            _nexustack::openapi::EnumSchemaBuilder::describe_unit_variant(
                &mut __enum_builder,
                #variant_index,
                #variant_id,
                Some(#description),
                #deprecated,
            )?;
        },
        Style::Newtype => {
            let field = &variant.fields[0];

            let span = field.original.span();
            let ty = field.ty;
            let func = quote_spanned!(span =>  _nexustack::openapi::EnumSchemaBuilder::collect_newtype_variant);
            quote_block! {
                #func(
                    &mut __enum_builder,
                    #variant_index,
                    #variant_id,
                    Some(#description),
                    #deprecated,
                    <#ty as _nexustack::openapi::Schema>::describe
                )?;
            }
        }
        Style::Tuple => describe_tuple_variant(variant, &variant.fields, variant_index),
        Style::Struct => describe_struct_variant(variant, cattrs, &variant.fields, variant_index),
    }
}

fn describe_untagged_variant(
    variant: &Variant,
    cattrs: &attr::Container,
    variant_index: u32,
) -> Fragment {
    let variant_name = variant.attrs.name().serialize_name();
    let variant_span = variant.original.span();
    let variant_callsite = callsite(&variant_span);
    let variant_id =
        quote! { _nexustack::openapi::SchemaId::new(#variant_name, #variant_callsite) };
    let description = variant.attrs.description();
    let deprecated = variant.attrs.deprecated();

    match effective_style(variant) {
        Style::Unit => {
            quote_block! {
                _nexustack::openapi::EnumSchemaBuilder::describe_unit_variant(
                    &mut __enum_builder,
                    #variant_index,
                    #variant_id,
                    Some(#description),
                    #deprecated,
                )?;
            }
        }
        Style::Newtype => {
            let field = &variant.fields[0];

            let span = field.original.span();
            let ty = field.ty;
            let func = quote_spanned!(span =>  _nexustack::openapi::EnumSchemaBuilder::collect_newtype_variant);
            quote_block! {
                    #func(
                        &mut __enum_builder,
                        #variant_index,
                        #variant_id,
                        Some(#description),
                        #deprecated,
                        <#ty as _nexustack::openapi::Schema>::describe
                    )?;

            }
        }
        Style::Tuple => describe_tuple_variant(variant, &variant.fields, variant_index),
        Style::Struct => describe_struct_variant(variant, cattrs, &variant.fields, variant_index),
    }
}

fn describe_tuple_variant(variant: &Variant, fields: &[Field], variant_index: u32) -> Fragment {
    let describe_stmts = describe_tuple_struct_visitor(fields, &TupleTrait::TupleVariant);

    let mut non_skipped_fields = fields
        .iter()
        .enumerate()
        .filter(|(_, field)| !field.attrs.skip())
        .peekable();

    let let_mut = mut_if(non_skipped_fields.peek().is_some());

    let len = non_skipped_fields
        .map(|_| quote!(1))
        .fold(quote!(0), |sum, expr| quote!(#sum + #expr));

    let description = variant.attrs.description();
    let deprecated = variant.attrs.deprecated();

    let variant_name = variant.attrs.name().serialize_name();
    let variant_span = variant.original.span();
    let variant_callsite = callsite(&variant_span);
    let variant_id =
        quote! { _nexustack::openapi::SchemaId::new(#variant_name, #variant_callsite) };

    quote_block! {
        let #let_mut __builder = _nexustack::openapi::EnumSchemaBuilder::describe_tuple_variant(
            &mut __enum_builder,
            #variant_index,
            #variant_id,
            #len,
            Some(#description),
            #deprecated,
        )?;
        #(#describe_stmts)*
        _nexustack::openapi::TupleVariantSchemaBuilder::end(__builder)?;
    }
}

fn describe_struct_variant(
    variant: &Variant,
    cattrs: &attr::Container,
    fields: &[Field],
    variant_index: u32,
) -> Fragment {
    let fields = fields
        .iter()
        .filter(|field| !field.attrs.skip())
        .collect::<Vec<_>>();

    let describe_fields =
        describe_struct_visitor(fields.iter().copied(), cattrs, &StructTrait::StructVariant);
    let mut serialized_fields = fields.iter().peekable();
    let let_mut = mut_if(serialized_fields.peek().is_some());

    let len = serialized_fields
        .map(|_| quote!(1))
        .fold(quote!(0), |sum, expr| quote!(#sum + #expr));

    let description = variant.attrs.description();
    let deprecated = variant.attrs.deprecated();

    let variant_name = variant.attrs.name().serialize_name();
    let variant_span = variant.original.span();
    let variant_callsite = callsite(&variant_span);
    let variant_id =
        quote! { _nexustack::openapi::SchemaId::new(#variant_name, #variant_callsite) };

    quote_block! {
        let #let_mut __builder = _nexustack::openapi::EnumSchemaBuilder::describe_struct_variant(
            &mut __enum_builder,
            #variant_index,
            #variant_id,
            #len,
            Some(#description),
            #deprecated,
        )?;
        #(#describe_fields)*
        _nexustack::openapi::StructVariantSchemaBuilder::end(__builder)?;
    }
}
