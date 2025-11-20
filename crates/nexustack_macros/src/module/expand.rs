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
use proc_macro2::TokenStream;
use quote::{ToTokens as _, format_ident, quote, quote_spanned};
use syn::{parse::Parser as _, spanned::Spanned as _};

pub fn expand_module(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let ctxt = Ctxt::new();

    match syn::parse2::<syn::Item>(item) {
        Ok(syn::Item::Trait(item_trait)) => {
            let result = process_item_trait(&ctxt, attr, item_trait);
            ctxt.check()?;
            Ok(result)
        }
        _ => {
            ctxt.error_spanned_by(
                attr,
                "The #[module] attribute must be placed on a trait definition.",
            );

            // Will error anyway
            ctxt.check()?;
            Ok(TokenStream::new())
        }
    }
}

fn process_item_trait(ctxt: &Ctxt, attr: TokenStream, item_trait: syn::ItemTrait) -> TokenStream {
    item_trait.generics.params.iter().any(|_| true).then(|| {
        ctxt.error_spanned_by(
            &item_trait.generics,
            "The #[module] attribute does not support generics on the trait.",
        );
    });

    let mut crate_path = Attr::none(ctxt, CRATE);
    let mut features = Attr::none(ctxt, FEATURES);

    if !attr.is_empty() {
        let parser = syn::meta::parser(|meta| {
            if meta.path == CRATE {
                // #[module(crate = "foo")]
                if let Some(path) = parse_lit_into_path(ctxt, CRATE, &meta)? {
                    crate_path.set(&meta.path, path);
                }
            } else if meta.path == FEATURES {
                // #[module(features(...))]
                let feats = parse_lit_into_ty_list(ctxt, FEATURES, &meta)?;
                features.set(&meta.path, feats);
            } else {
                let path = meta.path.to_token_stream().to_string().replace(' ', "");
                return Err(meta.error(format_args!("unknown container attribute `{path}`")));
            }

            Ok(())
        });

        let parse_res = parser.parse2(attr);
        if let Err(err) = parse_res {
            ctxt.syn_error(err);
        }
    }

    let crate_path = crate_path.get();
    let features = features.get().unwrap_or_default();

    let trait_vis = &item_trait.vis;
    let trait_ident = &item_trait.ident;
    let trait_attrs = &item_trait.attrs;
    let trait_base = item_trait
        .supertraits
        .iter()
        .map(|base| base.to_token_stream())
        .collect::<Vec<_>>();

    let fn_items = item_trait
        .items
        .iter()
        .map(|item| match item {
            syn::TraitItem::Fn(trait_item_fn) => {
                if trait_item_fn.sig.constness.is_some() {
                    ctxt.error_spanned_by(
                        trait_item_fn.sig.constness,
                        "The #[module] attribute does not support const functions inside the trait.",
                    );
                }

                if trait_item_fn.sig.asyncness.is_some() {
                    ctxt.error_spanned_by(
                        trait_item_fn.sig.asyncness,
                        "The #[module] attribute does not support async functions inside the trait.",
                    );
                }

                if trait_item_fn.sig.unsafety.is_some() {
                    ctxt.error_spanned_by(
                        trait_item_fn.sig.unsafety,
                        "The #[module] attribute does not support unsafe functions inside the trait.",
                    );
                }

                if trait_item_fn.sig.abi.is_some() {
                    ctxt.error_spanned_by(
                        &trait_item_fn.sig.abi,
                        "The #[module] attribute does not support functions with an ABI inside the trait.",
                    );
                }

                if trait_item_fn.sig.generics.where_clause.is_some() {
                    ctxt.error_spanned_by(
                        &trait_item_fn.sig.generics.where_clause,
                        "The #[module] attribute does not support where clauses on trait functions.",
                    );
                }

                if trait_item_fn.sig.variadic.is_some() {
                    ctxt.error_spanned_by(
                        &trait_item_fn.sig.variadic,
                        "The #[module] attribute does not support variadic functions inside the trait.",
                    );
                }

                if trait_item_fn.sig.generics.params.iter().any(|_| true) {
                    ctxt.error_spanned_by(
                        &trait_item_fn.sig.generics,
                        "The #[module] attribute does not support generics on trait functions.",
                    );
                }

                if trait_item_fn.sig.output == syn::ReturnType::Default {
                    ctxt.error_spanned_by(
                        &trait_item_fn.sig.ident,
                        "Functions inside a #[module] trait must have a return type.",
                    );
                }

                let fn_attrs = &trait_item_fn.attrs;
                let fn_ident = &trait_item_fn.sig.ident;
                let fn_input = &trait_item_fn.sig.inputs;
                let fn_output = &trait_item_fn.sig.output;

                quote! {
                    #(#fn_attrs)*
                    fn #fn_ident(#fn_input) #fn_output<Chain = Self::Chain>;
                }
            }
            item => {
                ctxt.error_spanned_by(
                    item,
                    "Only function definitions are allowed inside a #[module] trait.",
                );

                item.to_token_stream()
            }
        })
        .collect::<Vec<_>>();

    let impl_items = item_trait
        .items
        .iter()
        .map(|item| match item {
            syn::TraitItem::Fn(trait_item_fn) => {
                let fn_impl = &trait_item_fn.default;

                if let Some(fn_iml) = fn_impl {
                    let fn_ident = &trait_item_fn.sig.ident;
                    let fn_input = &trait_item_fn.sig.inputs;
                    let fn_output = &trait_item_fn.sig.output;
                    let output_type_guard = quote_spanned! {fn_output.span()=>
                        is_application_builder(&result);
                    };

                    quote! {
                        fn #fn_ident(#fn_input) #fn_output<Chain = Self::Chain> {
                            const fn is_application_builder<T: _nexustack::ApplicationBuilder> (_t: &T) {}
                            let result = {
                                #fn_iml
                            };

                            #output_type_guard

                            result
                        }
                    }
                } else {
                    ctxt.error_spanned_by(
                        &trait_item_fn.sig.ident,
                        "Function definitions inside a #[module] trait must have an implementation.",
                    );
                    quote! { }
                }
            },
            // Errors already handled above
            item => item.to_token_stream(),
        })
        .collect::<Vec<_>>();

    let indices = features.iter().map(|feature| {
        let feature_ident = match feature {
            syn::Type::Path(type_path) => &type_path.path.segments.last().unwrap().ident,
            _ => {
                ctxt.error_spanned_by(feature, "Expected a type path for the feature.");

                // Will error anyway
                &format_ident!("UnknownFeature")
            }
        };

        let index_ident = format_ident!("_{}__Index", feature_ident);

        quote! { , #index_ident: _nexustack::Index }
    });

    let where_clause = if features.is_empty() {
        quote! {}
    } else {
        let clauses = features.iter().map(|feature| {
            let feature_ident = match feature {
                syn::Type::Path(type_path) => &type_path.path.segments.last().unwrap().ident,
                _ => {
                    // Error already handled above
                    &format_ident!("UnknownFeature")
                }
            };

            let index_ident = format_ident!("_{}__Index", feature_ident);
            quote! {
                T::Chain: #feature<#index_ident>
            }
        });

        quote! { where #( #clauses ),*}
    };

    let item_gen_args = if features.is_empty() {
        quote! {}
    } else {
        let gen_arg = features.iter().map(|feature| {
            let feature_ident = match feature {
                syn::Type::Path(type_path) => &type_path.path.segments.last().unwrap().ident,
                _ => {
                    // Error already handled above
                    &format_ident!("UnknownFeature")
                }
            };

            format_ident!("_{}__Index", feature_ident)
        });

        quote! { < #( #gen_arg ),* >}
    };

    let impl_block = quote! {
        impl<T: _nexustack::ApplicationBuilder #( + #trait_base )* #(#indices)*> #trait_ident #item_gen_args for T
        #where_clause {
            #(#impl_items)*
        }
    };

    let impl_block = dummy::wrap_in_const(crate_path.as_ref(), impl_block);
    let crate_path = crate_path.unwrap_or(format_ident!("nexustack").into());

    quote! {
        #[allow(nonstandard_style)]
        #(#trait_attrs)*
        #trait_vis trait #trait_ident #item_gen_args: #crate_path::ApplicationBuilder #( + #trait_base )* {
            #(#fn_items)*
        }

        #impl_block
    }
}
