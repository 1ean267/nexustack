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
            ExampleContainerIdentifier, Parameters, StructTrait, describe_struct_visitor, mut_if,
        },
        generics::{field_contains_generic_params, make_lifetimes_static},
        internals::{
            ast::{Container, Field},
            attr,
            name::Name,
        },
        serde::{build_example_field_attribute, build_example_struct_attribute},
    },
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::ops::Deref;
use syn::spanned::Spanned;

pub fn expand_struct(cont: &Container, fields: &[Field]) -> TokenStream {
    let ident = &cont.ident;
    let params = Parameters::new(cont);

    let mut non_generic_fields = vec![];
    let mut generic_fields = vec![];

    for field in fields {
        if field.attrs.skip() {
            continue;
        }

        if field_contains_generic_params(field, cont) {
            generic_fields.push(field);
        } else {
            non_generic_fields.push(field);
        }
    }

    let impl_block = if generic_fields.is_empty() {
        let type_name = cont.attrs.name().serialize_name();
        base_case(
            ident,
            &params,
            cont,
            non_generic_fields.as_slice(),
            Some(type_name),
        )
    } else if non_generic_fields.is_empty() {
        base_case(ident, &params, cont, generic_fields.as_slice(), None)
    } else {
        combined_case(
            ident,
            &params,
            cont,
            non_generic_fields.as_slice(),
            generic_fields.as_slice(),
        )
    };

    quote! {
        static __callsite: _nexustack::__private::utils::AtomicOnceCell<_nexustack::Callsite> =
            _nexustack::__private::utils::AtomicOnceCell::new();

        #impl_block
    }
}

fn combined_case(
    ident: &syn::Ident,
    params: &Parameters,
    cont: &Container,
    non_generic_fields: &[&Field],
    generic_fields: &[&Field],
) -> TokenStream {
    let cattrs = &cont.attrs;
    let type_name = cattrs.name().serialize_name();

    let all_fields = non_generic_fields
        .iter()
        .chain(generic_fields)
        .map(Deref::deref)
        .collect::<Vec<_>>();

    let (impl_generics, ty_generics, where_clause) = params.generics.split_for_impl();
    let (example_cont, example_cont_id) = example_container(ident, cont, all_fields.as_slice());
    let examples_types = examples_type(all_fields.as_slice(), &example_cont_id);
    let description = cattrs.description();
    let deprecated = cattrs.deprecated();
    let examples = examples(all_fields.as_slice(), &example_cont_id);

    let ty_inner_non_generic = format_ident!("{}__Non_Generic", ident);
    let ty_inner_generic = format_ident!("{}__Generic", ident);

    let ty_inner_non_generic_impl = base_case(
        &ty_inner_non_generic,
        params,
        cont,
        non_generic_fields,
        Some(type_name),
    );
    let ty_inner_generic_impl = base_case(&ty_inner_generic, params, cont, generic_fields, None);

    let type_params_phantom_fields = cont
        .generics
        .type_params()
        .map(|type_param| {
            let ident = &type_param.ident;
            quote! { #ident: _nexustack::__private::PhantomData<fn() -> #ident> }
        })
        .collect::<Vec<_>>();

    quote! {
        #[automatically_derived]
        #example_cont

        #[automatically_derived]
        impl #impl_generics _nexustack::openapi::Schema for #ident #ty_generics #where_clause {
            type Example = <Self::Examples as Iterator>::Item;
            type Examples = #examples_types;

            fn describe<__B>(__schema_builder: __B) -> _nexustack::__private::Result<__B::Ok, __B::Error>
            where
                __B: _nexustack::openapi::SchemaBuilder<Self::Examples>,
            {
                let is_human_readable = _nexustack::openapi::SchemaBuilder::is_human_readable(&__schema_builder);
                let mut __one_of_schema_builder = _nexustack::openapi::SchemaBuilder::describe_all_of(
                    __schema_builder,
                    2,
                    _nexustack::__private::Option::Some(#description),
                    || _nexustack::__private::Result::Ok(#examples),
                    #deprecated,
                )?;

                _nexustack::openapi::CombinatorSchemaBuilder::collect_subschema(
                    &mut __one_of_schema_builder,
                    _nexustack::__private::Option::Some(#description),
                    #deprecated,
                    <#ty_inner_generic #ty_generics as _nexustack::openapi::Schema>::describe,
                )?;

                _nexustack::openapi::CombinatorSchemaBuilder::collect_subschema(
                    &mut __one_of_schema_builder,
                    _nexustack::__private::Option::Some(#description),
                    #deprecated,
                    <#ty_inner_non_generic #ty_generics as _nexustack::openapi::Schema>::describe,
                )?;

                _nexustack::openapi::CombinatorSchemaBuilder::end(__one_of_schema_builder)
            }
        }

        #[automatically_derived]
        struct #ty_inner_non_generic #ty_generics #where_clause {
            #(#type_params_phantom_fields),*
        }

        #ty_inner_non_generic_impl

        #[automatically_derived]
        struct #ty_inner_generic #ty_generics #where_clause {
            #(#type_params_phantom_fields),*
        }

        #ty_inner_generic_impl
    }
}

fn base_case(
    ident: &syn::Ident,
    params: &Parameters,
    cont: &Container,
    fields: &[&Field],
    name: Option<&Name>,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = params.generics.split_for_impl();
    let (example_cont, example_cont_id) = example_container(ident, cont, fields);
    let body = Stmts(describe(fields, cont, &example_cont_id, name));
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
    ident: &syn::Ident,
    cont: &Container,
    fields: &[&Field],
) -> (TokenStream, ExampleContainerIdentifier) {
    let example_cont_ident = format_ident!("__{}_Example", ident);
    let serde_attr = build_example_struct_attribute(cont);

    let mut where_clause = vec![];
    let mut generic_params = vec![];
    let mut generic_args: Vec<TokenStream> = vec![];
    let mut g_fields = Vec::with_capacity(fields.len());

    for field in fields {
        let mut ty = field.ty.clone();
        make_lifetimes_static(&mut ty);
        let ident = field
            .original
            .ident
            .as_ref()
            .expect("Brace structs have named fields");
        let serde_field_attr = build_example_field_attribute(field);

        if field_contains_generic_params(field, cont) {
            let generic_param = format_ident!("T__{}_Example", ident);
            where_clause.push(quote! { #generic_param: _serde::Serialize + 'static });
            generic_args.push(quote! { <#ty as _nexustack::openapi::Schema>::Example });
            g_fields.push(quote! { #serde_field_attr #ident: #generic_param });
            generic_params.push(generic_param);
        } else {
            g_fields.push(
                quote! { #serde_field_attr #ident: <#ty as _nexustack::openapi::Schema>::Example },
            );
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
            pub struct #example_cont_ident #generic_params #where_clause { #(#g_fields),* }
        },
        ExampleContainerIdentifier {
            ident: example_cont_ident,
            generic_args,
        },
    )
}

fn examples_type(fields: &[&Field], example_cont: &ExampleContainerIdentifier) -> TokenStream {
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

fn examples(fields: &[&Field], example_cont: &ExampleContainerIdentifier) -> TokenStream {
    let example_cont_instantiation = example_cont.instantiation();

    if fields.is_empty() {
        return quote!(_nexustack::__private::once(#example_cont_instantiation { }));
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
                (|e: <#ty as _nexustack::openapi::Schema>::Example| #example_cont_instantiation { #ident: e }) as _,
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
            (|f: (#(#example_ty_list),*)| #example_cont_instantiation { #(#f_entries),* }) as _,
        )
    }
}

fn describe(
    fields: &[&Field],
    cont: &Container,
    example_cont: &ExampleContainerIdentifier,
    name: Option<&Name>,
) -> Fragment {
    let cattrs = &cont.attrs;
    assert!(
        fields.len() as u64 <= u64::from(u32::MAX),
        "too many fields in {}: {}, maximum supported count is {}",
        cattrs.name().serialize_name(),
        fields.len(),
        u32::MAX,
    );

    let has_flatten = fields.iter().any(|field| field.attrs.flatten());

    if has_flatten {
        describe_struct_as_map(fields, cont, example_cont, name)
    } else {
        describe_struct_as_struct(fields, cont, example_cont, name)
    }
}

fn describe_struct_tag_field(cattrs: &attr::Container, struct_trait: &StructTrait) -> TokenStream {
    match cattrs.tag() {
        attr::TagType::Internal { tag } => {
            let func = struct_trait.describe_field(Span::call_site());

            quote! {
                #func(&mut __builder, #tag, _nexustack::openapi::FieldMod::ReadWrite, <str as _nexustack::openapi::Schema>::describe)?;
            }
        }
        _ => quote! {},
    }
}

fn describe_struct_as_struct(
    fields: &[&Field],
    cont: &Container,
    example_cont: &ExampleContainerIdentifier,
    name: Option<&Name>,
) -> Fragment {
    let cattrs = &cont.attrs;
    let describe_fields =
        describe_struct_visitor(fields.iter().copied(), cattrs, &StructTrait::Struct);

    // TODO: Does this work with the generic/non-generic combined case?
    let tag_field = describe_struct_tag_field(cattrs, &StructTrait::Struct);
    let tag_field_exists = !tag_field.is_empty();

    let mut serialized_fields = fields.iter().peekable();

    let let_mut = mut_if(serialized_fields.peek().is_some() || tag_field_exists);

    let len = serialized_fields.map(|_| quote!(1)).fold(
        quote!(#tag_field_exists as usize),
        |sum, expr| quote!(#sum + #expr),
    );

    let description = cattrs.description();
    let examples = examples(fields, example_cont);
    let deprecated = cattrs.deprecated();

    let id = if let Some(name) = name {
        let cont_span = cont.original.span();
        let cont_callsite = callsite(&cont_span);
        quote! { _nexustack::__private::Option::Some(_nexustack::openapi::SchemaId::new(#name, *__callsite.get_or_init(|| #cont_callsite))) }
    } else {
        quote! { _nexustack::__private::Option::None }
    };

    quote_block! {
        let is_human_readable = _nexustack::openapi::SchemaBuilder::is_human_readable(&__schema_builder);
        let #let_mut __builder = _nexustack::openapi::SchemaBuilder::describe_struct(
            __schema_builder,
            #id,
            #len,
            _nexustack::__private::Option::Some(#description),
            || _nexustack::__private::Result::Ok(#examples),
            #deprecated,
        )?;

        #tag_field
        #(#describe_fields)*

        _nexustack::openapi::StructSchemaBuilder::end(__builder)
    }
}

fn describe_struct_as_map(
    fields: &[&Field],
    cont: &Container,
    example_cont: &ExampleContainerIdentifier,
    name: Option<&Name>,
) -> Fragment {
    let cattrs = &cont.attrs;
    let describe_fields =
        describe_struct_visitor(fields.iter().copied(), cattrs, &StructTrait::Map);

    // TODO: Does this work with the generic/non-generic combined case?
    let tag_field = describe_struct_tag_field(cattrs, &StructTrait::Map);
    let tag_field_exists = !tag_field.is_empty();

    let mut serialized_fields = fields.iter().peekable();

    let let_mut = mut_if(serialized_fields.peek().is_some() || tag_field_exists);

    let description = cattrs.description();
    let examples = examples(fields, example_cont);
    let deprecated = cattrs.deprecated();

    let id = if let Some(name) = name {
        let cont_span = cont.original.span();
        let cont_callsite = callsite(&cont_span);
        quote! { _nexustack::__private::Option::Some(_nexustack::openapi::SchemaId::new(#name, *__callsite.get_or_init(|| #cont_callsite))) }
    } else {
        quote! { _nexustack::__private::Option::None }
    };

    quote_block! {
        let is_human_readable = _nexustack::openapi::SchemaBuilder::is_human_readable(&__schema_builder);
        let #let_mut __builder = _nexustack::openapi::SchemaBuilder::describe_map(
            __schema_builder,
            #id,
            _nexustack::__private::Option::Some(#description),
            || _nexustack::__private::Result::Ok(#examples),
            #deprecated,
        )?;

        #tag_field
        #(#describe_fields)*

        _nexustack::openapi::MapSchemaBuilder::end(__builder)
    }
}
