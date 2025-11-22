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
        expand::{ExampleContainerIdentifier, Parameters},
        generics::{field_contains_generic_params, make_lifetimes_static},
        internals::ast::{Container, Field},
        serde::{build_example_field_attribute, build_example_struct_attribute},
    },
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;

pub fn expand_newtype_struct(cont: &Container, field: &Field<'_>) -> TokenStream {
    let ident = &cont.ident;
    let params = Parameters::new(cont);
    let (impl_generics, ty_generics, where_clause) = params.generics.split_for_impl();
    let (example_cont, example_cont_id) = example_container(cont, field);
    let body = Stmts(describe(field, cont, &example_cont_id));
    let examples = examples_type(field, &example_cont_id);

    quote! {
        static __callsite: _nexustack::__private::utils::AtomicOnceCell<_nexustack::Callsite> =
            _nexustack::__private::utils::AtomicOnceCell::new();

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
    cont: &Container,
    field: &Field<'_>,
) -> (TokenStream, ExampleContainerIdentifier) {
    let example_cont_ident = format_ident!("__{}_Example", cont.ident);
    let serde_attr = build_example_struct_attribute(cont);

    if field_contains_generic_params(field, cont) {
        let mut ty = field.ty.clone();
        make_lifetimes_static(&mut ty);
        let generic_param = syn::Ident::new("T__Example", Span::call_site());
        let serde_field_attr = build_example_field_attribute(field);

        (
            quote! {
                #[derive(_serde::Serialize)]
                #serde_attr
                pub struct #example_cont_ident <#generic_param>(
                    #serde_field_attr
                    #generic_param,
                ) where #generic_param: _serde::Serialize + 'static;
            },
            ExampleContainerIdentifier {
                ident: example_cont_ident,
                generic_args: vec![quote! { <#ty as _nexustack::openapi::Schema>::Example }],
            },
        )
    } else {
        let ty = field.ty;
        let serde_field_attr = build_example_field_attribute(field);

        (
            quote! {
                #[derive(_serde::Serialize)]
                #serde_attr
                pub struct #example_cont_ident(
                    #serde_field_attr
                    <#ty as _nexustack::openapi::Schema>::Example,
                );
            },
            ExampleContainerIdentifier {
                ident: example_cont_ident,
                generic_args: vec![],
            },
        )
    }
}

fn examples_type(field: &Field, example_cont: &ExampleContainerIdentifier) -> TokenStream {
    let ty = field.ty;

    quote! {
        _nexustack::__private::Map<<#ty as _nexustack::openapi::Schema>::Examples, fn(<#ty as _nexustack::openapi::Schema>::Example) -> #example_cont>
    }
}

fn examples(field: &Field, example_cont: &ExampleContainerIdentifier) -> TokenStream {
    let example_cont_instantiation = example_cont.instantiation();

    let ty = field.ty;
    quote! {
        _nexustack::__private::Iterator::map(
            <#ty as _nexustack::openapi::SchemaExamples>::examples(is_human_readable)?,
            (|e: <#ty as _nexustack::openapi::Schema>::Example| #example_cont_instantiation(e)) as _,
        )
    }
}

fn describe(
    field: &Field,
    cont: &Container,
    example_cont: &ExampleContainerIdentifier,
) -> Fragment {
    let cattrs = &cont.attrs;
    let type_name = cattrs.name().serialize_name();
    let span = field.original.span();

    let description = cattrs.description();
    let examples = examples(field, example_cont);
    let deprecated = cattrs.deprecated();
    let func = quote_spanned!(span => _nexustack::openapi::SchemaBuilder::collect_newtype_struct);
    let ty = field.ty;

    // If the field contains one of the containers generic parameters,
    // we cannot provide a unique type name.
    let id = if field_contains_generic_params(field, cont) {
        quote! { _nexustack::__private::Option::None }
    } else {
        let cont_span = cont.original.span();
        let cont_callsite = callsite(&cont_span);
        quote! { _nexustack::__private::Option::Some(_nexustack::openapi::SchemaId::new(#type_name, *__callsite.get_or_init(|| #cont_callsite))) }
    };

    quote_expr! {
        let is_human_readable = _nexustack::openapi::SchemaBuilder::is_human_readable(&__schema_builder);
        #func(
            __schema_builder,
            #id,
            _nexustack::__private::Option::Some(#description),
            || _nexustack::__private::Result::Ok(#examples),
            #deprecated,
            <#ty as _nexustack::openapi::Schema>::describe
        )
    }
}
