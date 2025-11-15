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
use syn::{parse::Parser as _, spanned::Spanned};

pub fn expand_cron(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    // TODO: Replace receiver
    let ctxt = Ctxt::new();

    match syn::parse2::<syn::Item>(item) {
        Ok(syn::Item::Fn(func)) => {
            let result = expand_function_cron(&ctxt, attr, func);
            ctxt.check()?;
            Ok(result)
        }
        _ => {
            ctxt.error_spanned_by(
                attr,
                "The #[cron] attribute must be placed on a function definition.",
            );

            // Will error anyway
            ctxt.check()?;
            Ok(TokenStream::new())
        }
    }
}

fn expand_function_cron(ctxt: &Ctxt, attr: TokenStream, mut item_fn: syn::ItemFn) -> TokenStream {
    if !item_fn.sig.generics.params.is_empty() {
        ctxt.syn_error(syn::Error::new_spanned(
            &attr,
            "Cron functions cannot have generic parameters.",
        ));
    }

    if item_fn.sig.variadic.is_some() {
        ctxt.syn_error(syn::Error::new_spanned(
            &attr,
            "Cron functions cannot be variadic.",
        ));
    }

    for input in item_fn.sig.inputs.iter_mut() {
        match input {
            syn::FnArg::Receiver(_) => {
                // TODO: This is unreachable, as we only accept freestanding functions
                ctxt.syn_error(syn::Error::new_spanned(
                    input,
                    "Cron functions cannot have a receiver.",
                ));
            }
            syn::FnArg::Typed(pat_type) => {
                let len = pat_type.attrs.len();
                pat_type.attrs.retain(|attr| !is_cron_service_attr(attr));

                if len == pat_type.attrs.len() {
                    ctxt.syn_error(syn::Error::new_spanned(
                        &pat_type,
                        "Unknown parameter. To inject a service decorate it with the #[cron::service] attribute.",
                    ));
                }
            }
        }
    }

    let mut crate_path = Attr::none(ctxt, CRATE);
    let mut schedule = Attr::none(ctxt, SCHEDULE);
    let mut schedule_with = Attr::none(ctxt, SCHEDULE_WITH);

    let span = attr.span();

    if !attr.is_empty() {
        let parser = syn::meta::parser(|meta| {
            if meta.path == CRATE {
                // #[cron(crate = "foo")]
                if let Some(path) = parse_lit_into_path(ctxt, CRATE, &meta)? {
                    crate_path.set(&meta.path, path);
                }
            } else if meta.path == SCHEDULE {
                // #[cron(schedule = "...")]
                if let Some(lit_str) = get_lit_str(ctxt, SCHEDULE, &meta)? {
                    schedule.set(&meta.path, lit_str.value());
                }
            } else if meta.path == SCHEDULE_WITH {
                // #[cron(schedule_with = "...")]
                if let Some(path) = parse_lit_into_expr_path(ctxt, SCHEDULE_WITH, &meta)? {
                    schedule_with.set(&meta.path, path);
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

    let schedule = schedule.get();
    let schedule_with = schedule_with.get();

    let schedule_with = if let Some(schedule) = &schedule {
        if schedule_with.is_some() {
            ctxt.syn_error(syn::Error::new(
                span,
                "Conflicting attributes: Only one of `schedule` or `schedule_with` can be specified.",
            ));
        }

        if let Err(err) = schedule.parse::<cron::Schedule>() {
            ctxt.syn_error(syn::Error::new(
                span,
                format!("Invalid cron schedule expression: {}", err),
            ));
        }

        // TODO: cron path hygiene
        quote! {
            |_service_provider: _nexustack::inject::ServiceProvider| async {
                #schedule.parse::<_nexustack::__private::cron::Schedule>().map_err(|err| _nexustack::cron::CronError::ScheduleError(err.into()))
            }
        }
    } else if let Some(schedule_with) = &schedule_with {
        quote_spanned! { schedule_with.span()=>
           |service_provider: _nexustack::inject::ServiceProvider| async {
               #schedule_with(service_provider).await.map_err(|err| _nexustack::cron::CronError::ScheduleError(err.into()))
           }
        }
    } else {
        ctxt.syn_error(syn::Error::new(
            span,
            "Missing required attribute: Either `schedule` or `schedule_with` must be specified.",
        ));

        // Errors anyway
        quote!()
    };

    let service_inits = item_fn
        .sig
        .inputs
        .iter()
        .enumerate()
        .map(|(index, input)| match input {
            syn::FnArg::Receiver(_) => unreachable!(),
            syn::FnArg::Typed(pat_type) => {
                let ty = &pat_type.ty;
                let pat_name = match &*pat_type.pat {
                    syn::Pat::Ident(pat_ident) => &pat_ident.ident,
                    _ => &format_ident!("arg_{}", index),
                };

                quote_spanned! { pat_type.span()=>
                    let #pat_name = service_provider
                        .resolve::<#ty>()
                        .map_err(|err| _nexustack::cron::CronError::RunError(err.into()))?;
                }
            }
        });

    let cron_args = item_fn.sig.inputs.iter().map(|input| match input {
        syn::FnArg::Receiver(_) => unreachable!(),
        syn::FnArg::Typed(pat_type) => {
            let pat_name = &pat_type.pat;
            quote! { #pat_name }
        }
    });

    let map_cron_err = quote_spanned! { item_fn.sig.output.span()=>
        |err| _nexustack::cron::CronError::RunError(err.into())
    };

    let cron_item_name = item_fn.sig.ident.clone();
    let cron_item_vis = item_fn.vis.clone();

    item_fn.sig.ident = format_ident!("{}_cron_impl", item_fn.sig.ident);

    let cron_fn_name = &item_fn.sig.ident;

    let cron_item_name_str = cron_item_name.to_string();

    let impl_block = quote! {
        #item_fn

        // TODO: Impl AsyncFn for cron_item_name to restore the option to call the function directly --- FUTURE WORK ---

        #[allow(clippy::used_underscore_binding)]
        impl _nexustack::cron::CronJob for #cron_item_name {
            async fn schedule(
                service_provider: _nexustack::inject::ServiceProvider,
            ) -> _nexustack::cron::CronResult<_nexustack::__private::cron::Schedule> {
                let schedule_with = #schedule_with;

                schedule_with(service_provider).await
            }

            async fn run(
                service_provider: _nexustack::inject::ServiceProvider,
            ) -> _nexustack::cron::CronResult {
                #(#service_inits)*

                #cron_fn_name(#(#cron_args),*).await.map_err(#map_cron_err)?;

                _nexustack::__private::Ok(())
            }

            fn name() -> _nexustack::__private::Cow<'static, str> {
                _nexustack::__private::Cow::Borrowed(#cron_item_name_str)
            }
        }
    };
    let crate_path = crate_path.get();
    let impl_block = dummy::wrap_in_const(crate_path.as_ref(), impl_block);

    quote! {
        #[allow(nonstandard_style)]
        #cron_item_vis struct #cron_item_name;

        #impl_block
    }
}

fn is_cron_service_attr(attr: &syn::Attribute) -> bool {
    match &attr.meta {
        syn::Meta::Path(attr_path) => {
            is_path(attr_path, &["cron", "service"])
                || is_path(attr_path, &["cron", "cron", "service"])
                || is_path(attr_path, &["nexustack", "cron", "cron", "service"])
        }
        _ => false,
    }
}

// TODO: Stolen from inject expand, refactor later
fn is_path(path: &syn::Path, segments: &[&str]) -> bool {
    if path.leading_colon.is_some() {
        return false;
    }

    if path.segments.len() != segments.len() {
        return false;
    }

    for (i, segment) in path.segments.iter().enumerate() {
        if !segment.arguments.is_none() {
            return false;
        }

        if segment.ident != segments[i] {
            return false;
        }
    }

    true
}
