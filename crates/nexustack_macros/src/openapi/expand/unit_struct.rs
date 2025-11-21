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
        internals::ast::Container,
        serde::build_example_struct_attribute,
    },
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;

pub fn expand_unit_struct(cont: &Container) -> TokenStream {
    let ident = &cont.ident;
    let params = Parameters::new(cont);
    let (impl_generics, ty_generics, where_clause) = params.generics.split_for_impl();
    let (example_cont, example_cont_id) = example_container(cont);
    let body = Stmts(describe(cont, &example_cont_id));
    let examples = examples_type(&example_cont_id);

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

fn example_container(cont: &Container) -> (TokenStream, ExampleContainerIdentifier) {
    let example_cont_ident = format_ident!("__{}_Example", cont.ident);
    let serde_attr = build_example_struct_attribute(cont);

    (
        quote! {
            #[derive(_serde::Serialize)]
            #serde_attr
            pub struct #example_cont_ident;
        },
        ExampleContainerIdentifier {
            ident: example_cont_ident,
            generic_args: vec![],
        },
    )
}

fn examples_type(example_cont: &ExampleContainerIdentifier) -> TokenStream {
    quote!(_nexustack::__private::Once<#example_cont>)
}

fn describe(cont: &Container, example_cont: &ExampleContainerIdentifier) -> Fragment {
    let cattrs = &cont.attrs;
    let example_cont_instantiation = example_cont.instantiation();
    let description = cattrs.description();
    let deprecated = cattrs.deprecated();
    let type_name = cattrs.name().serialize_name();
    let cont_span = cont.original.span();
    let cont_callsite = callsite(&cont_span);

    quote_expr! {
        _nexustack::openapi::SchemaBuilder::describe_unit_struct(
            __schema_builder,
            _nexustack::__private::Option::Some(_nexustack::openapi::SchemaId::new(#type_name, *__callsite.get_or_init(|| #cont_callsite))),
            _nexustack::__private::Option::Some(#description),
            || _nexustack::__private::Result::Ok(_nexustack::__private::once(#example_cont_instantiation)),
            #deprecated,
        )
    }
}
