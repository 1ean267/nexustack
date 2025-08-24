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
            ExampleContainerIdentifier, Parameters, TupleTrait, describe_tuple_struct_visitor,
            mut_if,
        },
        generics::field_contains_generic_params,
        internals::ast::{Container, Field},
        serde::{build_example_field_attribute, build_example_struct_attribute},
    },
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;

pub fn expand_tuple_struct(cont: &Container, fields: &[Field]) -> TokenStream {
    let ident = &cont.ident;
    let params = Parameters::new(cont);
    let (impl_generics, ty_generics, where_clause) = params.generics.split_for_impl();
    let (example_cont, example_cont_id) = example_container(cont, fields);
    let body = Stmts(describe(fields, cont, &example_cont_id));
    let examples = examples_type(fields, &example_cont_id);

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
    cont: &Container,
    fields: &[Field],
) -> (TokenStream, ExampleContainerIdentifier) {
    let fields = fields
        .iter()
        .filter(|field| !field.attrs.skip())
        .collect::<Vec<_>>();

    let example_cont_ident = format_ident!("__{}_Example", cont.ident);
    let serde_attr = build_example_struct_attribute(cont);

    let mut where_clause = vec![];
    let mut generic_params = vec![];
    let mut generic_args: Vec<TokenStream> = vec![];
    let mut g_fields = Vec::with_capacity(fields.len());

    for (index, field) in fields.iter().enumerate() {
        let ty = field.ty;
        let serde_field_attr = build_example_field_attribute(field);

        if field_contains_generic_params(field, cont) {
            let generic_param = format_ident!("T__{}_Example", syn::Index::from(index));
            where_clause.push(quote! { #generic_param: _serde::Serialize + 'static });
            generic_args.push(quote! { <#ty as _nexustack::openapi::Schema>::Example });
            g_fields.push(quote! { #serde_field_attr #generic_param });
            generic_params.push(generic_param);
        } else {
            g_fields
                .push(quote! { #serde_field_attr <#ty as _nexustack::openapi::Schema>::Example });
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
            pub struct #example_cont_ident #generic_params(#(#g_fields),*) #where_clause;
        },
        ExampleContainerIdentifier {
            ident: example_cont_ident,
            generic_args,
        },
    )
}

fn examples_type(fields: &[Field], example_cont: &ExampleContainerIdentifier) -> TokenStream {
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

fn examples(fields: &[Field], example_cont: &ExampleContainerIdentifier) -> TokenStream {
    let fields = fields
        .iter()
        .filter(|field| !field.attrs.skip())
        .collect::<Vec<_>>();

    let example_cont_instantiation = example_cont.instantiation();

    // This should never occur
    if fields.is_empty() {
        return quote!(_nexustack::__private::once(#example_cont_instantiation()));
    }

    // This should never occur
    if fields.len() == 1 {
        let ty = fields[0].ty;

        return quote! {
            _nexustack::__private::Iterator::map(
                <#ty as _nexustack::openapi::SchemaExamples>::examples(is_human_readable)?,
                (|e: <#ty as _nexustack::openapi::Schema>::Example| #example_cont_instantiation(e)) as _,
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
            (|f: (#(#example_ty_list),*)| #example_cont_instantiation(#(#f_entries),*)) as _,
        )
    }
}

fn describe(
    fields: &[Field],
    cont: &Container,
    example_cont: &ExampleContainerIdentifier,
) -> Fragment {
    let cattrs = &cont.attrs;
    let describe_stmts = describe_tuple_struct_visitor(fields, &TupleTrait::TupleStruct);

    let type_name = cattrs.name().serialize_name();

    let mut serialized_fields = fields.iter().enumerate().peekable();

    let let_mut = mut_if(serialized_fields.peek().is_some());

    let len = serialized_fields
        .map(|_| quote!(1))
        .fold(quote!(0), |sum, expr| quote!(#sum + #expr));

    let description = cattrs.description();
    let deprecated = cattrs.deprecated();
    let examples = examples(fields, example_cont);

    // If any of the fields contains one of the containers generic parameters,
    // we cannot provide a unique type name.
    let id = if fields
        .iter()
        .any(|field| field_contains_generic_params(field, cont))
    {
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
        let #let_mut __builder = _nexustack::openapi::SchemaBuilder::describe_tuple_struct(
            __schema_builder,
            #id,
            #len,
            _nexustack::__private::Option::Some(#description),
            || _nexustack::__private::Result::Ok(#examples),
            #deprecated,
        )?;

        #(#describe_stmts)*

        _nexustack::openapi::TupleStructSchemaBuilder::end(__builder)
    }
}
