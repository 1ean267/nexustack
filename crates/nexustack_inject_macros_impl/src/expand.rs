/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{dummy, internals::Ctxt};
use proc_macro2::TokenStream;
use quote::quote;

pub fn expand_injectable(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    // TODO: Replace receiver
    let ctxt = Ctxt::new();

    let item_impl = match syn::parse2::<syn::ItemImpl>(item) {
        Ok(item_impl) => item_impl,
        Err(_) => {
            ctxt.error_spanned_by(
                attr,
                "The #[injectable] attribute must be placed on an impl definition.",
            );

            // Will error anyway
            ctxt.check()?;
            return Ok(TokenStream::new());
        }
    };

    let result = process_item_impl(&ctxt, attr, item_impl);
    ctxt.check()?;
    Ok(result)
}

fn process_item_impl(ctxt: &Ctxt, attr: TokenStream, item_impl: syn::ItemImpl) -> TokenStream {
    match item_impl.self_ty.as_ref() {
        syn::Type::Path(..) => {}
        _ => {
            ctxt.error_spanned_by(
                &attr,
                "The injectable type must be type a without lifetime parameters.",
            );
        }
    };

    if item_impl.generics.lifetimes().any(|_| true) {
        ctxt.error_spanned_by(
            &attr,
            "The injectable type must be a type without lifetime parameters.",
        );
    }

    let fns = item_impl.items.iter().filter_map(|item| match item {
        syn::ImplItem::Fn(func) => Some(func),
        _ => None,
    });

    let ctor_fn = match find_injectable_ctor(ctxt, fns) {
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

    let mut transformed_item_impl = item_impl.clone();

    for fn_item in transformed_item_impl
        .items
        .iter_mut()
        .filter_map(|item| match item {
            syn::ImplItem::Fn(func) => Some(func),
            _ => None,
        })
    {
        if let Some(index) = fn_item.attrs.iter().position(|attr| match &attr.meta {
            syn::Meta::Path(attr_path) => attr_path.is_ident("injectable_ctor"),
            _ => false,
        }) {
            fn_item.attrs.swap_remove(index);
        }
    }

    let ident = item_impl.self_ty.as_ref();
    let generics = item_impl.generics.params;
    let where_clause = item_impl.generics.where_clause;

    let impl_block = quote! {
        #[automatically_derived]
        impl <#generics> _nexustack_inject::FromInjector for #ident #where_clause  {
            fn from_injector(
                injector: &_nexustack_inject::Injector,
            ) -> _nexustack_inject::ConstructionResult<Self> {
                #(#arguments)*

                _nexustack_inject::IntoConstructionResult::into_construction_result(Self::#ctor_name(#(#ctor_parameter_names),*))
            }
        }

        #[automatically_derived]
        impl <#generics> _nexustack_inject::Injectable for #ident #where_clause { }

        #[automatically_derived]
        impl <#generics> _nexustack_inject::IntoConstructionResult for #ident #where_clause {
            type Value = #ident;

            fn into_construction_result(self) -> _nexustack_inject::ConstructionResult<Self::Value> {
                _nexustack_inject::ConstructionResult::Ok(self)
            }
        }
    };

    let impl_block = dummy::wrap_in_const(None, impl_block);

    quote! {
        #transformed_item_impl
        #impl_block
    }
}

fn get_injectable_ctor_attr(fun: &syn::ImplItemFn) -> Option<&syn::Attribute> {
    fun.attrs.iter().find(|attr| match &attr.meta {
        syn::Meta::Path(attr_path) => attr_path.is_ident("injectable_ctor"),
        _ => false,
    })
}

fn is_static_func(fun: &syn::ImplItemFn) -> bool {
    !matches!(fun.sig.inputs.first(), Some(syn::FnArg::Receiver(_)))
}

fn find_injectable_ctor<'a>(
    ctxt: &Ctxt,
    fns: impl Iterator<Item = &'a syn::ImplItemFn>,
) -> Option<&'a syn::ImplItemFn> {
    let mut default_ctor: Option<&'a syn::ImplItemFn> = None;
    let mut decorated_ctor: Option<&'a syn::ImplItemFn> = None;

    for fun in fns {
        let injectable_ctor_attr = get_injectable_ctor_attr(fun);

        if let Some(injectable_ctor_attr) = injectable_ctor_attr
            && decorated_ctor.is_some()
        {
            ctxt.error_spanned_by(
                injectable_ctor_attr,
                "Found multiple viable type constructors decorated with #[injectable_ctor].",
            );
        }

        if !is_static_func(fun) {
            if let Some(injectable_ctor_attr) = injectable_ctor_attr {
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
