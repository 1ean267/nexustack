/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    http::internals::ast::{Container, Data, Style, Variant},
    internals::{Ctxt, replace_receiver},
};
use proc_macro2::TokenStream;
use quote::quote;

pub fn expand_http_response(
    attr: TokenStream,
    input: &mut syn::DeriveInput,
) -> syn::Result<TokenStream> {
    replace_receiver(input);

    let ctxt = Ctxt::new();
    let cont = match Container::from_ast(&ctxt, attr, input) {
        Some(cont) => cont,
        None => return Err(ctxt.check().unwrap_err()),
    };
    precondition(&ctxt, &cont);
    ctxt.check()?;

    let impl_block = match cont.data {
        Data::Struct(style) => expand_struct(&cont, style),
        Data::Enum(variants) => expand_enum(&cont, variants),
    };

    Ok(quote! {
        #input
        #impl_block
    })
}

fn expand_struct(cont: &Container, style: Style) -> TokenStream {
    let ident = &cont.ident;
    let encoder = cont.attrs.encoder();
    let status_code = cont.attrs.status_code();
    let status_code = quote! {axum::http::StatusCode::#status_code };

    // TODO: Serde crate path
    // TODO: Axum crate path

    let into_response_impl = match style {
        Style::Newtype(_) => {
            quote! {
                _nexustack::http::encoding::Encoder::into_response(#encoder, #status_code, self.0, context)
            }
        }
        Style::Unit => {
            quote! {
                axum::response::IntoResponse::into_response(#status_code)
            }
        }
    };

    // TODO: Add S and constraint all to serde::Serialize to impl_generics
    let impl_block = quote! {
        impl #impl_generics _nexustack::http::response::IntoResponseWithContext<S> for #ident #ty_generics #where_clause {
            type Context = <#encoder as _nexustack::http::encoding::Encoder>::Context;

            fn into_response(self, context: Self::Context) -> _axum::http::Response<axum::body::Body> {
                #into_response_impl
            }
        }
    };

    if cont.attrs.api_skip() {
        return impl_block;
    }

    let description = cont.attrs.description();
    let deprecated = cont.attrs.deprecated();

    let describe_impl = match style {
        Style::Newtype(field) => {
            let field_ty = field.ty;
            quote! {
                // TODO: Use explicit function path
                response_builder.collect_response(
                    #status_code.as_u16(),
                    #description,
                    #deprecated,
                    <#encoder as openapi::HttpContentType>::describe::<<#field_ty>, _>,
                )?;
            }
        }
        Style::Unit => {
            quote! {
                // TODO: Use explicit function path
                response_builder.describe_empty_response(
                    #status_code.as_u16(),
                    #description,
                    #deprecated,
                )?;
            }
        }
    };

    // TODO: constraint all to serde::Serialize to _nexustack::openapi::Schema
    let api_doc_block = quote! {
        impl #impl_generics  openapi::HttpResponse for #ident #ty_generics #where_clause {
            fn describe<B>(mut response_builder: B) -> Result<B::Ok, B::Error>
            where
                B: _nexustack::openapi::HttpResponseBuilder,
            {
                #describe_impl

                // TODO: Use explicit function path
                response_builder.end()
            }
        }
    };

    quote! {
        #impl_block
        #api_doc_block
    }
}

fn expand_enum(cont: &Container, variants: Vec<Variant>) -> TokenStream {
    // TODO
    quote! {}
}
