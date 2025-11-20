/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    dummy,
    internals::{Ctxt, attr::*, symbol::*},
};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens as _, format_ident, quote};
use syn::parse::Parser as _;

pub fn expand_resolvable_type(
    ctxt: &Ctxt,
    item_impl: &mut syn::ItemImpl,
    crate_path: Option<&syn::Path>,
    attr_span: Span,
    attr_path: &[Symbol],
) -> TokenStream {
    match item_impl.self_ty.as_ref() {
        syn::Type::Path(..) => {}
        _ => {
            ctxt.error(
                attr_span,
                "The type must be a type a without lifetime parameters.",
            );
        }
    };

    if item_impl.generics.lifetimes().any(|_| true) {
        ctxt.error(
            attr_span,
            "The type must be a a type without lifetime parameters.",
        );
    }

    let fns = item_impl.items.iter_mut().filter_map(|item| match item {
        syn::ImplItem::Fn(func) => Some(func),
        _ => None,
    });

    let ctor_fn = match find_injectable_ctor(ctxt, fns, attr_path) {
        Some(ctor_fn) => ctor_fn,
        _ => {
            ctxt.error(attr_span, "No viable constructor function found.");
            // Will error anyway
            return TokenStream::new();
        }
    };

    let input_types = &ctor_fn
        .sig
        .inputs
        .iter()
        .map(|input| match input {
            syn::FnArg::Typed(input_type) => input_type,
            _ => {
                unreachable!()
            }
        })
        .collect::<Vec<&syn::PatType>>();

    // let {#parameter_name} = injector.resolve::<{#parameter_type}>()?;
    let arguments = input_types.iter().map(|input_type| {
        let parameter_type = input_type.ty.as_ref();
        let parameter_name = match input_type.pat.as_ref() {
            syn::Pat::Ident(parameter_name) => &parameter_name.ident,
            _ => panic!("TODO: When does this happen??"),
        };

        quote! {
            let #parameter_name = injector.resolve::<#parameter_type>()?;
        }
    });

    let ctor_name = &ctor_fn.sig.ident;
    let ctor_parameter_names = input_types
        .iter()
        .map(|input_type| match input_type.pat.as_ref() {
            syn::Pat::Ident(parameter_name) => &parameter_name.ident,
            _ => panic!("TODO: When does this happen??"),
        });

    let ident = item_impl.self_ty.as_ref();
    let generics = &item_impl.generics.params;
    let where_clause = &item_impl.generics.where_clause;

    let impl_block = quote! {
        #[automatically_derived]
        impl <#generics> _nexustack::inject::FromInjector for #ident #where_clause  {
            fn from_injector(
                injector: &_nexustack::inject::Injector,
            ) -> _nexustack::inject::ConstructionResult<Self> {
                #(#arguments)*

                _nexustack::inject::IntoConstructionResult::into_construction_result(Self::#ctor_name(#(#ctor_parameter_names),*))
            }
        }

        #[automatically_derived]
        impl <#generics> _nexustack::inject::IntoConstructionResult for #ident #where_clause {
            type Service = #ident;

            fn into_construction_result(self) -> _nexustack::inject::ConstructionResult<Self::Service> {
                _nexustack::inject::ConstructionResult::Ok(self)
            }
        }
    };

    let impl_block = dummy::wrap_in_const(crate_path, impl_block);

    quote! {
        #impl_block
    }
}

pub fn expand_injectable(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    // TODO: Replace receiver
    let ctxt = Ctxt::new();

    match syn::parse2::<syn::Item>(item) {
        Ok(syn::Item::Impl(item_impl)) => {
            let result = process_item_impl(&ctxt, attr, item_impl);
            ctxt.check()?;
            Ok(result)
        }
        Ok(syn::Item::Struct(item_struct)) => match &item_struct.fields {
            _fields @ syn::Fields::Named(_) => {
                let result = process_item_struct(&ctxt, attr, item_struct);
                ctxt.check()?;
                Ok(result)
            }
            _fields @ syn::Fields::Unnamed(_) => {
                let result = process_item_tuple_struct(&ctxt, attr, item_struct);
                ctxt.check()?;
                Ok(result)
            }
            _fields @ syn::Fields::Unit => {
                let result = process_item_unit_struct(&ctxt, attr, item_struct);
                ctxt.check()?;
                Ok(result)
            }
        },
        _ => {
            ctxt.error_spanned_by(
                attr,
                "The #[injectable] attribute must be placed on an impl definition or struct definition.",
            );

            // Will error anyway
            ctxt.check()?;
            Ok(TokenStream::new())
        }
    }
}

fn process_item_impl(ctxt: &Ctxt, attr: TokenStream, mut item_impl: syn::ItemImpl) -> TokenStream {
    match item_impl.self_ty.as_ref() {
        syn::Type::Path(..) => {}
        _ => {
            ctxt.error_spanned_by(
                &attr,
                "The injectable type must be a type a without lifetime parameters.",
            );
        }
    };

    if item_impl.generics.lifetimes().any(|_| true) {
        ctxt.error_spanned_by(
            &attr,
            "The injectable type must be a a type without lifetime parameters.",
        );
    }

    let fns = item_impl.items.iter_mut().filter_map(|item| match item {
        syn::ImplItem::Fn(func) => Some(func),
        _ => None,
    });

    let ctor_fn = match find_injectable_ctor(ctxt, fns, &[NEXUSTACK, INJECT, INJECTABLE]) {
        Some(ctor_fn) => ctor_fn,
        _ => {
            ctxt.error_spanned_by(&attr, "No viable constructor function found.");
            // Will error anyway
            return TokenStream::new();
        }
    };

    let input_types = &ctor_fn
        .sig
        .inputs
        .iter()
        .map(|input| match input {
            syn::FnArg::Typed(input_type) => input_type,
            _ => {
                unreachable!()
            }
        })
        .collect::<Vec<&syn::PatType>>();

    // let {#parameter_name} = injector.resolve::<{#parameter_type}>()?;
    let arguments = input_types.iter().map(|input_type| {
        let parameter_type = input_type.ty.as_ref();
        let parameter_name = match input_type.pat.as_ref() {
            syn::Pat::Ident(parameter_name) => &parameter_name.ident,
            _ => panic!("TODO: When does this happen??"),
        };

        quote! {
            let #parameter_name = injector.resolve::<#parameter_type>()?;
        }
    });

    let ctor_name = &ctor_fn.sig.ident;
    let ctor_parameter_names = input_types
        .iter()
        .map(|input_type| match input_type.pat.as_ref() {
            syn::Pat::Ident(parameter_name) => &parameter_name.ident,
            _ => panic!("TODO: When does this happen??"),
        });

    let ident = item_impl.self_ty.as_ref();
    let generics = &item_impl.generics.params;
    let where_clause = &item_impl.generics.where_clause;

    let impl_block = quote! {
        #[automatically_derived]
        impl <#generics> _nexustack::inject::FromInjector for #ident #where_clause  {
            fn from_injector(
                injector: &_nexustack::inject::Injector,
            ) -> _nexustack::inject::ConstructionResult<Self> {
                #(#arguments)*

                _nexustack::inject::IntoConstructionResult::into_construction_result(Self::#ctor_name(#(#ctor_parameter_names),*))
            }
        }

        #[automatically_derived]
        impl <#generics> _nexustack::inject::Injectable for #ident #where_clause { }

        #[automatically_derived]
        impl <#generics> _nexustack::inject::IntoConstructionResult for #ident #where_clause {
            type Service = #ident;

            fn into_construction_result(self) -> _nexustack::inject::ConstructionResult<Self::Service> {
                _nexustack::inject::ConstructionResult::Ok(self)
            }
        }
    };

    let crate_path = get_crate_path(ctxt, attr);
    let impl_block = dummy::wrap_in_const(crate_path.as_ref(), impl_block);

    quote! {
        #item_impl
        #impl_block
    }
}

fn get_crate_path(ctxt: &Ctxt, attr: TokenStream) -> Option<syn::Path> {
    let mut crate_path = None;

    if !attr.is_empty() {
        let parser = syn::meta::parser(|meta| {
            if meta.path == CRATE {
                // #[inject(crate = "foo")]
                if let Some(path) = parse_lit_into_path(ctxt, CRATE, &meta)? {
                    crate_path = Some(path);
                }
            } else {
                let path = meta.path.to_token_stream().to_string().replace(' ', "");
                return Err(meta.error(format_args!("unknown attribute `{path}`")));
            }
            Ok(())
        });

        let parse_res = parser.parse2(attr);
        if let Err(err) = parse_res {
            ctxt.syn_error(err);
        }
    }

    crate_path
}

fn is_injectable_ctor_attr(attr: &syn::Attribute, attr_path: &[Symbol]) -> bool {
    match &attr.meta {
        syn::Meta::Path(path) => {
            if path.leading_colon.is_some() {
                return false;
            }

            if path
                .segments
                .iter()
                .any(|segment| !segment.arguments.is_none())
            {
                return false;
            }

            if path.segments.last().is_none_or(|last| last.ident != CTOR) {
                return false;
            }

            if path.segments.len() - 1 > attr_path.len() {
                return false;
            }

            for (segment, attr_path_segment) in
                std::iter::zip(path.segments.iter().rev().skip(1), attr_path.iter().rev())
            {
                if attr_path_segment != &segment.ident {
                    return false;
                }
            }

            true
        }
        _ => false,
    }
}

fn get_injectable_ctor_attr(
    fun: &mut syn::ImplItemFn,
    attr_path: &[Symbol],
) -> Option<syn::Attribute> {
    for (i, attr) in fun.attrs.iter().enumerate() {
        if is_injectable_ctor_attr(attr, attr_path) {
            return Some(fun.attrs.remove(i));
        }
    }

    None
}

fn is_static_func(fun: &syn::ImplItemFn) -> bool {
    !matches!(fun.sig.inputs.first(), Some(syn::FnArg::Receiver(_)))
}

fn find_injectable_ctor<'a>(
    ctxt: &Ctxt,
    fns: impl Iterator<Item = &'a mut syn::ImplItemFn>,
    attr_path: &[Symbol],
) -> Option<&'a syn::ImplItemFn> {
    let mut default_ctor: Option<&'a syn::ImplItemFn> = None;
    let mut decorated_ctor: Option<&'a syn::ImplItemFn> = None;

    for fun in fns {
        let injectable_ctor_attr = get_injectable_ctor_attr(fun, attr_path);

        if let Some(injectable_ctor_attr) = &injectable_ctor_attr
            && decorated_ctor.is_some()
        {
            let error_msg = format!(
                "Found multiple viable type constructors decorated with #[{}].",
                attr_path
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join("::")
            );

            ctxt.error_spanned_by(injectable_ctor_attr, error_msg);
        }

        if !is_static_func(fun) {
            if let Some(injectable_ctor_attr) = &injectable_ctor_attr {
                ctxt.error_spanned_by(injectable_ctor_attr, "Type constructor has self parameter.");
            }

            continue;
        }

        if !fun.sig.generics.params.is_empty() {
            if let Some(injectable_ctor_attr) = injectable_ctor_attr {
                ctxt.error_spanned_by(
                    injectable_ctor_attr,
                    "Type constructor has generic parameters.",
                );
            }

            continue;
        }

        if injectable_ctor_attr.is_some() {
            decorated_ctor = Some(fun);
        } else if fun.sig.ident == "new"
            && let syn::Visibility::Public(_) = fun.vis
        {
            default_ctor = Some(fun);
        }
    }

    decorated_ctor.or(default_ctor)
}

fn process_item_unit_struct(
    ctxt: &Ctxt,
    attr: TokenStream,
    struct_impl: syn::ItemStruct,
) -> TokenStream {
    if struct_impl.generics.lifetimes().any(|_| true) {
        ctxt.error_spanned_by(
            &attr,
            "The injectable type must be a type without lifetime parameters.",
        );
    }

    let ident = &struct_impl.ident;
    let generics = &struct_impl.generics.params;
    let where_clause = &struct_impl.generics.where_clause;

    let impl_block = quote! {
        #[automatically_derived]
        impl <#generics> _nexustack::inject::FromInjector for #ident #where_clause  {
            fn from_injector(
                injector: &_nexustack::inject::Injector,
            ) -> _nexustack::inject::ConstructionResult<Self> {
                _nexustack::inject::IntoConstructionResult::into_construction_result(Self)
            }
        }

        #[automatically_derived]
        impl <#generics> _nexustack::inject::Injectable for #ident #where_clause { }

        #[automatically_derived]
        impl <#generics> _nexustack::inject::IntoConstructionResult for #ident #where_clause {
            type Service = #ident;

            fn into_construction_result(self) -> _nexustack::inject::ConstructionResult<Self::Service> {
                _nexustack::inject::ConstructionResult::Ok(self)
            }
        }
    };

    let crate_path = get_crate_path(ctxt, attr);
    let impl_block = dummy::wrap_in_const(crate_path.as_ref(), impl_block);

    quote! {
        #struct_impl
        #impl_block
    }
}

fn process_item_tuple_struct(
    ctxt: &Ctxt,
    attr: TokenStream,
    struct_impl: syn::ItemStruct,
) -> TokenStream {
    if struct_impl.generics.lifetimes().any(|_| true) {
        ctxt.error_spanned_by(
            &attr,
            "The injectable type must be a type without lifetime parameters.",
        );
    }

    // let {#parameter_name} = injector.resolve::<{#parameter_type}>()?;
    let arguments = struct_impl.fields.iter().enumerate().map(|(index, field)| {
        let field_type = &field.ty;
        let var_name = format_ident!("arg_{index}");

        quote! {
            let #var_name = injector.resolve::<#field_type>()?;
        }
    });

    let field_names = (0usize..struct_impl.fields.len()).map(|index| format_ident!("arg_{index}"));

    let ident = &struct_impl.ident;
    let generics = &struct_impl.generics.params;
    let where_clause = &struct_impl.generics.where_clause;

    let impl_block = quote! {
        #[automatically_derived]
        impl <#generics> _nexustack::inject::FromInjector for #ident #where_clause  {
            fn from_injector(
                injector: &_nexustack::inject::Injector,
            ) -> _nexustack::inject::ConstructionResult<Self> {
                #(#arguments)*

                _nexustack::inject::IntoConstructionResult::into_construction_result(Self ( #(#field_names),* ))
            }
        }

        #[automatically_derived]
        impl <#generics> _nexustack::inject::Injectable for #ident #where_clause { }

        #[automatically_derived]
        impl <#generics> _nexustack::inject::IntoConstructionResult for #ident #where_clause {
            type Service = #ident;

            fn into_construction_result(self) -> _nexustack::inject::ConstructionResult<Self::Service> {
                _nexustack::inject::ConstructionResult::Ok(self)
            }
        }
    };

    let crate_path = get_crate_path(ctxt, attr);
    let impl_block = dummy::wrap_in_const(crate_path.as_ref(), impl_block);

    quote! {
        #struct_impl
        #impl_block
    }
}

fn process_item_struct(
    ctxt: &Ctxt,
    attr: TokenStream,
    struct_impl: syn::ItemStruct,
) -> TokenStream {
    if struct_impl.generics.lifetimes().any(|_| true) {
        ctxt.error_spanned_by(
            &attr,
            "The injectable type must be a type without lifetime parameters.",
        );
    }

    // let {#parameter_name} = injector.resolve::<{#parameter_type}>()?;
    let arguments = struct_impl.fields.iter().map(|field| {
        let field_type = &field.ty;
        let field_name = match &field.ident {
            Some(ident) => ident,
            _ => unreachable!("Fields of braced structs are always named"),
        };

        quote! {
            let #field_name = injector.resolve::<#field_type>()?;
        }
    });

    let field_names = struct_impl.fields.iter().map(|field| match &field.ident {
        Some(ident) => ident,
        _ => unreachable!("Fields of braced structs are always named"),
    });

    let ident = &struct_impl.ident;
    let generics = &struct_impl.generics.params;
    let where_clause = &struct_impl.generics.where_clause;

    let impl_block = quote! {
        #[automatically_derived]
        impl <#generics> _nexustack::inject::FromInjector for #ident #where_clause  {
            fn from_injector(
                injector: &_nexustack::inject::Injector,
            ) -> _nexustack::inject::ConstructionResult<Self> {
                #(#arguments)*

                _nexustack::inject::IntoConstructionResult::into_construction_result(Self { #(#field_names),* })
            }
        }

        #[automatically_derived]
        impl <#generics> _nexustack::inject::Injectable for #ident #where_clause { }

        #[automatically_derived]
        impl <#generics> _nexustack::inject::IntoConstructionResult for #ident #where_clause {
            type Service = #ident;

            fn into_construction_result(self) -> _nexustack::inject::ConstructionResult<Self::Service> {
                _nexustack::inject::ConstructionResult::Ok(self)
            }
        }
    };

    let crate_path = get_crate_path(ctxt, attr);
    let impl_block = dummy::wrap_in_const(crate_path.as_ref(), impl_block);

    quote! {
        #struct_impl
        #impl_block
    }
}
