/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    dummy::wrap_in_const,
    http::response::internals::{
        ast::{Container, Data, Style, Variant},
        attr,
    },
    internals::{Ctxt, replace_receiver},
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::spanned::Spanned;

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
    // precondition(&ctxt, &cont);
    ctxt.check()?;

    let impl_block = match cont.data {
        Data::Struct(style) => {
            expand_struct(style, &cont.attrs, &cont.ident, &cont.original.generics)
        }
        Data::Enum(variants) => {
            expand_enum(variants, &cont.attrs, &cont.ident, &cont.original.generics)
        }
    };

    Ok(quote! {
        #input
        #impl_block
    })
}

// TODO: Stolen from openapi.bound.rs - refactor to common place

// Remove the default from every type parameter because in the generated impls
// they look like associated types: "error: associated type bindings are not
// allowed here".
fn without_defaults(generics: &syn::Generics) -> syn::Generics {
    syn::Generics {
        params: generics
            .params
            .iter()
            .map(|param| match param {
                syn::GenericParam::Type(param) => syn::GenericParam::Type(syn::TypeParam {
                    eq_token: None,
                    default: None,
                    ..param.clone()
                }),
                _ => param.clone(),
            })
            .collect(),
        ..generics.clone()
    }
}

fn expand_struct(
    style: Style,
    cont_attrs: &attr::Container,
    cont_ident: &syn::Ident,
    cont_generics: &syn::Generics,
) -> TokenStream {
    let encoder = cont_attrs.encoder();
    let encoder_span = encoder.span();
    let status_code = cont_attrs.status_code();
    let status_code = quote! {_nexustack::__private::axum::http::StatusCode::#status_code };
    let generics = without_defaults(cont_generics);
    #[cfg_attr(not(feature = "openapi"), allow(unused_variables))]
    let (api_impl_generics, ty_generics, _) = generics.split_for_impl();

    let state_gen_arg_ident = generics.params.iter().fold(
        syn::Ident::new("S", Span::call_site()),
        |s_gen_arg_ident, param| {
            let param_ident = match &param {
                syn::GenericParam::Lifetime(_) => {
                    // A lifetime parameter must start with a tick and thus cannot match the ident
                    return s_gen_arg_ident;
                }
                syn::GenericParam::Type(type_param) => &type_param.ident,
                syn::GenericParam::Const(const_param) => &const_param.ident,
            };

            if param_ident != &s_gen_arg_ident {
                s_gen_arg_ident
            } else {
                format_ident!("{}_", s_gen_arg_ident)
            }
        },
    );

    let mut impl_generics = generics.clone();
    impl_generics
        .params
        .push(syn::GenericParam::Type(syn::TypeParam {
            attrs: Vec::new(),
            ident: state_gen_arg_ident.clone(),
            colon_token: None,
            bounds: syn::punctuated::Punctuated::new(),
            eq_token: None,
            default: None,
        }));

    if let Style::Newtype(field) = &style {
        let field_ty = field.ty;
        let field_span = field.original.span();
        impl_generics
            .make_where_clause()
            .predicates
            .push(syn::parse_quote_spanned! {field_span=>
                #field_ty: _nexustack::__private::serde::Serialize
            });
    }

    impl_generics
        .make_where_clause()
        .predicates
        .push(syn::parse_quote_spanned! {encoder_span=>
            #encoder: _nexustack::http::encoding::Encoder
        });

    // TODO: Should this be part of the Encoder type definition?
    impl_generics.make_where_clause().predicates.push(
        syn::parse_quote_spanned! {encoder_span=>
            <#encoder as _nexustack::http::encoding::Encoder>::Context: _nexustack::__private::axum::extract::FromRequestParts<S> + _nexustack::__private::Send
        },
    );

    let (impl_generics, _, impl_where_clause) = impl_generics.split_for_impl();

    let into_response_impl = match style {
        Style::Newtype(_) => {
            quote! {
                _nexustack::http::encoding::Encoder::into_response(#encoder, #status_code, self.0, context)
            }
        }
        Style::Unit => {
            quote! {
                _nexustack::__private::axum::response::IntoResponse::into_response(#status_code)
            }
        }
    };

    #[cfg_attr(not(feature = "openapi"), allow(unused_mut))]
    let mut impl_block = quote! {
        impl #impl_generics _nexustack::http::response::IntoResponseWithContext<#state_gen_arg_ident> for #cont_ident #ty_generics #impl_where_clause {
            type Context = <#encoder as _nexustack::http::encoding::Encoder>::Context;

            fn into_response(self, context: Self::Context) -> _nexustack::__private::axum::http::Response<_nexustack::__private::axum::body::Body> {
                #into_response_impl
            }
        }
    };

    #[cfg(feature = "openapi")]
    {
        let description = cont_attrs.description();
        let deprecated = cont_attrs.deprecated();
        let describe_impl = if !cont_attrs.api_skip() {
            match &style {
                Style::Newtype(field) => {
                    let field_ty = field.ty;
                    quote! {
                        _nexustack::openapi::HttpResponseBuilder::collect_response(
                            &mut response_builder,
                            #status_code.as_u16(),
                            Some(#description),
                            #deprecated,
                            <#encoder as _nexustack::openapi::HttpContentType<#field_ty>>::describe,
                        )?;
                    }
                }
                Style::Unit => {
                    quote! {
                        _nexustack::openapi::HttpResponseBuilder::describe_empty_response(
                            &mut response_builder,
                            #status_code.as_u16(),
                            Some(#description),
                            #deprecated,
                        )?;
                    }
                }
            }
        } else {
            quote! {}
        };

        let mut api_generics = generics.clone();

        if !cont_attrs.api_skip()
            && let Style::Newtype(field) = &style
        {
            let field_ty = field.ty;
            let field_span = field.original.span();
            api_generics.make_where_clause().predicates.push(
                syn::parse_quote_spanned! {field_span=>
                    #field_ty: _nexustack::openapi::Schema
                },
            );
        }

        if let Style::Newtype(field) = &style {
            let field_ty = field.ty;
            api_generics.make_where_clause().predicates.push(
                syn::parse_quote_spanned! {encoder_span=>
                    #encoder: _nexustack::openapi::HttpContentType<#field_ty>
                },
            );
        }

        let api_where_clause = api_generics.where_clause;

        let api_doc_block = quote! {
            impl #api_impl_generics  _nexustack::openapi::HttpResponse for #cont_ident #ty_generics #api_where_clause {
                fn describe<B>(mut response_builder: B) -> Result<B::Ok, B::Error>
                where
                    B: _nexustack::openapi::HttpResponseBuilder,
                {
                    #describe_impl

                    _nexustack::openapi::HttpResponseBuilder::end(response_builder)
                }
            }
        };

        impl_block = quote! {
            #impl_block
            #api_doc_block
        };
    }

    wrap_in_const(cont_attrs.custom_crate_path(), impl_block)
}

fn expand_enum(
    variants: Vec<Variant>,
    cont_attrs: &attr::Container,
    cont_ident: &syn::Ident,
    cont_generics: &syn::Generics,
) -> TokenStream {
    let encoder = cont_attrs.encoder();
    let encoder_span = encoder.span();
    let status_code = cont_attrs.status_code();
    let status_code = quote! {_nexustack::__private::axum::http::StatusCode::#status_code };
    let generics = without_defaults(cont_generics);
    #[cfg_attr(not(feature = "openapi"), allow(unused_variables))]
    let (api_impl_generics, ty_generics, _) = generics.split_for_impl();

    let state_gen_arg_ident = generics.params.iter().fold(
        syn::Ident::new("S", Span::call_site()),
        |s_gen_arg_ident, param| {
            let param_ident = match &param {
                syn::GenericParam::Lifetime(_) => {
                    // A lifetime parameter must start with a tick and thus cannot match the ident
                    return s_gen_arg_ident;
                }
                syn::GenericParam::Type(type_param) => &type_param.ident,
                syn::GenericParam::Const(const_param) => &const_param.ident,
            };

            if param_ident != &s_gen_arg_ident {
                s_gen_arg_ident
            } else {
                format_ident!("{}_", s_gen_arg_ident)
            }
        },
    );

    let mut impl_generics = generics.clone();
    impl_generics
        .params
        .push(syn::GenericParam::Type(syn::TypeParam {
            attrs: Vec::new(),
            ident: state_gen_arg_ident.clone(),
            colon_token: None,
            bounds: syn::punctuated::Punctuated::new(),
            eq_token: None,
            default: None,
        }));

    for variant in &variants {
        let style = &variant.style;

        if let Style::Newtype(field) = &style {
            let field_ty = field.ty;
            let field_span = field.original.span();
            impl_generics.make_where_clause().predicates.push(
                syn::parse_quote_spanned! {field_span=>
                    #field_ty: _nexustack::__private::serde::Serialize
                },
            );
        }
    }

    impl_generics
        .make_where_clause()
        .predicates
        .push(syn::parse_quote_spanned! {encoder_span=>
            #encoder: _nexustack::http::encoding::Encoder
        });

    // TODO: Should this be part of the Encoder type definition?
    impl_generics.make_where_clause().predicates.push(
        syn::parse_quote_spanned! {encoder_span=>
            <#encoder as _nexustack::http::encoding::Encoder>::Context: _nexustack::__private::axum::extract::FromRequestParts<S> + _nexustack::__private::Send
        },
    );

    let (impl_generics, _, impl_where_clause) = impl_generics.split_for_impl();

    let into_response_impl = variants.iter().map(|variant| {
        let style = &variant.style;
        let variant_ident = &variant.ident;
        let status_code = variant.attrs.status_code().map(|status_code| quote! {_nexustack::__private::axum::http::StatusCode::#status_code }).unwrap_or(status_code.clone());
        match style {
            Style::Newtype(_) => {
                let encoder = variant.attrs.encoder().unwrap_or(&encoder);
                quote! {
                    Self:: #variant_ident(val) => {
                        _nexustack::http::encoding::Encoder::into_response(#encoder, #status_code, val, context)
                    }
                }
            }
            Style::Unit => {
                quote! {
                    Self:: #variant_ident => {
                        _nexustack::__private::axum::response::IntoResponse::into_response(#status_code)
                    }
                }
            }
        }
    });

    #[cfg_attr(not(feature = "openapi"), allow(unused_mut))]
    let mut impl_block = quote! {
        impl #impl_generics _nexustack::http::response::IntoResponseWithContext<#state_gen_arg_ident> for #cont_ident #ty_generics #impl_where_clause {
            type Context = <#encoder as _nexustack::http::encoding::Encoder>::Context;

            fn into_response(self, context: Self::Context) -> _nexustack::__private::axum::http::Response<_nexustack::__private::axum::body::Body> {
                match self {
                    #(
                        #into_response_impl
                    ),*
                }
            }
        }
    };

    #[cfg(feature = "openapi")]
    {
        let deprecated = cont_attrs.deprecated();

        let describe_impl = if !cont_attrs.api_skip() {
            either::Either::Left(variants.iter().filter(|variant| !variant.attrs.api_skip()).map(|variant| {
            let style = &variant.style;
            let status_code = variant.attrs.status_code().map(|status_code| quote! {_nexustack::__private::axum::http::StatusCode::#status_code }).unwrap_or(status_code.clone());
            let description = variant.attrs.description();
            let deprecated = variant.attrs.deprecated() || deprecated;

            match &style {
                Style::Newtype(field) => {
                    let field_ty = field.ty;
                    quote! {
                        _nexustack::openapi::HttpResponseBuilder::collect_response(
                            &mut response_builder,
                            #status_code.as_u16(),
                            Some(#description),
                            #deprecated,
                            <#encoder as _nexustack::openapi::HttpContentType<#field_ty>>::describe,
                        )?;
                    }
                }
                Style::Unit => {
                    quote! {
                        _nexustack::openapi::HttpResponseBuilder::describe_empty_response(
                            &mut response_builder,
                            #status_code.as_u16(),
                            Some(#description),
                            #deprecated,
                        )?;
                    }
                }
            }
        }))
        } else {
            either::Either::Right(std::iter::empty())
        };

        let mut api_generics = generics.clone();

        if !cont_attrs.api_skip() {
            for variant in &variants {
                let style = &variant.style;

                if variant.attrs.api_skip() {
                    continue;
                }

                if let Style::Newtype(field) = &style {
                    let field_ty = field.ty;
                    let field_span = field.original.span();
                    api_generics.make_where_clause().predicates.push(
                        syn::parse_quote_spanned! {field_span=>
                            #field_ty: _nexustack::openapi::Schema
                        },
                    );
                }
            }
        }

        for variant in variants.iter().filter(|variant| !variant.attrs.api_skip()) {
            let style = &variant.style;

            if let Style::Newtype(field) = &style {
                let field_ty = field.ty;

                api_generics.make_where_clause().predicates.push(
                    syn::parse_quote_spanned! {encoder_span=>
                        #encoder: _nexustack::openapi::HttpContentType<#field_ty>
                    },
                );
            }
        }

        let api_where_clause = api_generics.where_clause;

        let api_doc_block = quote! {
            impl #api_impl_generics  _nexustack::openapi::HttpResponse for #cont_ident #ty_generics #api_where_clause {
                fn describe<B>(mut response_builder: B) -> Result<B::Ok, B::Error>
                where
                    B: _nexustack::openapi::HttpResponseBuilder,
                {
                    #(
                        #describe_impl
                    )*

                    _nexustack::openapi::HttpResponseBuilder::end(response_builder)
                }
            }
        };

        impl_block = quote! {
            #impl_block
            #api_doc_block
        };
    }

    wrap_in_const(cont_attrs.custom_crate_path(), impl_block)
}
