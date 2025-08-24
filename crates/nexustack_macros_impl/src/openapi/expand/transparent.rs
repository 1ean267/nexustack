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
    openapi::{
        expand::Parameters,
        internals::ast::{Container, Data},
    },
};
use proc_macro2::TokenStream;
use quote::quote;

pub fn expand_transparent(cont: &Container) -> TokenStream {
    let ident = &cont.ident;
    let params = Parameters::new(cont);
    let (impl_generics, ty_generics, where_clause) = params.generics.split_for_impl();
    let body = Stmts(describe(cont));
    let examples = examples_type(cont);

    quote! {
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

fn examples_type(cont: &Container) -> TokenStream {
    let fields = match &cont.data {
        Data::Struct(_, fields) => fields,
        Data::Enum(_) => unreachable!(),
    };

    let transparent_field = fields.iter().find(|f| f.attrs.transparent()).unwrap();
    let ty = transparent_field.ty;

    quote! { <#ty as _nexustack::openapi::Schema>::Examples }
}

fn describe(cont: &Container) -> Fragment {
    let fields = match &cont.data {
        Data::Struct(_, fields) => fields,
        Data::Enum(_) => unreachable!(),
    };

    let transparent_field = fields.iter().find(|f| f.attrs.transparent()).unwrap();
    let ty = transparent_field.ty;

    quote_block! { <#ty as _nexustack::openapi::Schema>::describe(__schema_builder) }
}
