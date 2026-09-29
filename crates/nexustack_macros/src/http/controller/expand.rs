/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    dummy::wrap_in_const,
    http::controller::internals::{
        ast::{Action, Controller},
        attr,
    },
    inject::expand_resolvable_type,
    internals::{Ctxt, callsite, default::Default, symbol::*},
};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::spanned::Spanned;

pub fn expand_http_controller(
    attr: TokenStream,
    input: &mut syn::ItemImpl,
) -> syn::Result<TokenStream> {
    let ctxt = Ctxt::new();
    let attr_span = attr.span();
    let cont = match Controller::from_ast(&ctxt, attr, input) {
        Some(cont) => cont,
        None => return Err(ctxt.check().unwrap_err()),
    };
    // precondition(&ctxt, &cont);
    let resolvable_impl_block = expand_resolvable_type(
        &ctxt,
        input,
        cont.attrs.custom_crate_path(),
        attr_span,
        &[NEXUSTACK, HTTP, CONTROLLER],
    );

    ctxt.check()?;

    let cont_path = &cont.ty.path;

    let endpoints_impls_and_context_constraints = cont
        .actions
        .iter()
        .map(|action| expand_http_endpoint(cont_path, &cont.attrs, action))
        .collect::<Vec<_>>();

    let endpoints_impls = endpoints_impls_and_context_constraints
        .iter()
        .map(|(endpoints_impls, _)| endpoints_impls);

    let context_constraints = endpoints_impls_and_context_constraints
        .iter()
        .flat_map(|(_, context_constraints)| context_constraints.into_iter())
        .collect::<Vec<_>>();

    let compiled_context_constraints = if context_constraints.is_empty() {
        quote!()
    } else {
        quote! {
            where #(#context_constraints)*
        }
    };

    let endpoints: Vec<_> = cont
        .actions
        .iter()
        .map(|action| format_ident!("__{}_Endpoint", action.original.sig.ident))
        .collect();

    let impl_block = quote! {
        impl<C: _nexustack::http::HttpEndpointContext> _nexustack::http::HttpController<C> for #cont_path #compiled_context_constraints {
            fn build_endpoints<B>(mut builder: B)
            where
                B: _nexustack::http::HttpEndpointsBuilder<Context = C>,
            {
                #(
                    builder.add_endpoint::<#endpoints>();
                )*
            }
        }

        #(
            #endpoints_impls
        )*
    };

    let impl_block = wrap_in_const(cont.attrs.custom_crate_path(), impl_block);

    Ok(quote! {
        #input
        #impl_block
        #resolvable_impl_block
    })
}

fn expand_http_endpoint(
    cont_path: &syn::Path,
    cont_attrs: &attr::Controller,
    action: &Action,
) -> (TokenStream, Vec<TokenStream>) {
    let endpoint_ident = format_ident!("__{}_Endpoint", action.original.sig.ident);
    let method = match action.attrs.method() {
        attr::HttpMethod::Get => quote! { _nexustack::http::HttpMethod::Get },
        attr::HttpMethod::Post => quote! { _nexustack::http::HttpMethod::Post },
        attr::HttpMethod::Put => quote! { _nexustack::http::HttpMethod::Put },
        attr::HttpMethod::Delete => quote! { _nexustack::http::HttpMethod::Delete },
        attr::HttpMethod::Patch => quote! { _nexustack::http::HttpMethod::Patch },
        attr::HttpMethod::Options => quote! { _nexustack::http::HttpMethod::Options },
        attr::HttpMethod::Head => quote! { _nexustack::http::HttpMethod::Head },
        attr::HttpMethod::Trace => quote! { _nexustack::http::HttpMethod::Trace },
    };
    let openapi_method = action.attrs.method().as_str().to_ascii_uppercase();
    let openapi_operation_name = format!(
        "{}.{}",
        cont_path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap(), // TODO: No unwrap
        action.original.sig.ident
    );
    let route = action.attrs.route();
    let response = match &action.original.sig.output {
        syn::ReturnType::Type(_, ty) => ty.to_token_stream(),
        syn::ReturnType::Default => {
            quote! { () }
        }
    };
    let action_ident = &action.original.sig.ident;
    let receiver = action.original.sig.receiver().map(|receiver| {
        if receiver.mutability.is_some() {
            quote! { &mut self.0, }
        } else {
            quote! { &self.0, }
        }
    });
    let action_description = action.attrs.description();
    let action_span = action.original.span();
    let action_callsite = callsite(&action_span);
    let action_deprecated = action.attrs.deprecated() | cont_attrs.deprecated();

    let context_constraints = action
        .args
        .iter()
        .filter_map(|arg| {
            let arg_span = arg.original.span();
            let arg_ty = &arg.original.ty;
            match &arg.attrs {
                attr::ActionArg::Session(session) => Some(quote_spanned! {arg_span=>
                    C::Session: _nexustack::__private::Into<#arg_ty>,
                }),
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    let arg_impls = action.args.iter().map(|arg| {
        let arg_span = arg.original.span();
        let arg_ident = &arg.ident;
        let arg_ty = &arg.original.ty;
        match &arg.attrs {
            attr::ActionArg::Param(_) => {
                quote_spanned! {arg_span=>
                    let #arg_ident = __param_container.#arg_ident;
                }
            },
            attr::ActionArg::IpAddress(_) => {
                #[cfg(feature = "axum-client-ip")]
                quote_spanned! {arg_span=>
                    let #arg_ident: #arg_ty = <
                        _nexustack::__private::axum_client_ip::ClientIp::< < #arg_ty as _nexustack::__private::utils::Optional > :: Inner > as _nexustack::__private::axum::extract::FromRequestParts<_>
                    >::from_request_parts(&mut parts, &()).await.ok().map(|client_ip| client_ip.0);
                }
                #[cfg(not(feature = "axum-client-ip"))]
                quote_spanned! {arg_span=>
                    let #arg_ident: #arg_ty = <
                        _nexustack::__private::axum::extract::ConnectInfo::<_nexustack::__private::std::net::SocketAddr> as _nexustack::__private::axum::extract::FromRequestParts<_>
                    >::from_request_parts(&mut parts, &()).await.ok().map(|connect_info| connect_info.0.ip());
                }
            },
            attr::ActionArg::Query(_) => {
                quote_spanned! {arg_span=>
                    let #arg_ident = __query_container.#arg_ident;
                }
            },
            attr::ActionArg::Header(header) => {
                let header_rename = header.name().map(|name| name.to_string());
                let header_default = header.default();

                if let Some (header_rename) = header_rename {
                    let header_fallback = match header_default {
                        Default::None => quote! {
                            return Err(_nexustack::http::headers::ExtractHeaderError::new(
                                _nexustack::__private::axum::http::HeaderName::from_static(#header_rename),
                                _nexustack::http::headers::ExtractHeaderErrorReason::Missing,
                            ).into());
                        },
                        Default::Default => quote! {
                            _nexustack::__private::Default::default()
                        },
                        Default::Path(path) => quote! {
                            #path
                        },
                    };

                    quote_spanned! {arg_span=>
                        let #arg_ident: #arg_ty = if let Some(content_type_header) = parts.headers.get(#header_rename) {
                            let deserializer = _nexustack::http::headers::HeaderValueDeserializer {
                                value: content_type_header,
                            };

                            <#arg_ty as _nexustack::__private::serde::Deserialize>::deserialize(deserializer).map_err(|err| nexustack::http::headers::ExtractHeaderError::new(
                                _nexustack::__private::axum::http::HeaderName::from_static(#header_rename),
                                _nexustack::http::headers::ExtractHeaderErrorReason::Error(err.into()),
                            ))?
                        } else {
                            #header_fallback
                        };
                    }
                } else {
                    match header_default {
                        Default::None => quote_spanned! {arg_span=>
                            let #arg_ident: #arg_ty =
                            <#arg_ty as _nexustack::__private::From<<#arg_ty as _nexustack::http::headers::OptionalHeader>::Header>>::from(
                                <
                                    _nexustack::__private::axum_extra::TypedHeader::<
                                        <#arg_ty as _nexustack::http::headers::OptionalHeader>::Header
                                    > as _nexustack::__private::axum::extract::FromRequestParts<_>
                                    // TODO: Macro_hygiene
                                >::from_request_parts(
                                    &mut parts,
                                    &()
                                ).await.map_err(
                                    |err| <_nexustack::http::headers::ExtractHeaderError as From<_nexustack::__private::axum_extra::typed_header::TypedHeaderRejection>>::from(err)
                                )?.0
                            );
                        },
                        Default::Default => quote_spanned! {arg_span=>
                            // TODO: Macro_hygiene
                            let #arg_ident: #arg_ty = match <
                                _nexustack::__private::axum_extra::TypedHeader::<
                                    <#arg_ty as _nexustack::http::headers::OptionalHeader>::Header
                                > as _nexustack::__private::axum::extract::OptionalFromRequestParts<_>
                            >::from_request_parts(
                                &mut parts,
                                &()
                            ).await.map_err(
                                |err| <_nexustack::http::headers::ExtractHeaderError as From<_nexustack::__private::axum_extra::typed_header::TypedHeaderRejection>>::from(err)
                            )? {
                                Some(_nexustack::__private::axum_extra::TypedHeader(#arg_ident)) =>  <#arg_ty as _nexustack::__private::From<<#arg_ty as _nexustack::http::headers::OptionalHeader>::Header>>::from(#arg_ident),
                                None => <#arg_ty as _nexustack::__private::Default>::default(),
                            };
                        },
                        Default::Path(path) =>quote_spanned! {arg_span=>
                            // TODO: Mostly copied from Default::Default above
                            // TODO: Macro_hygiene
                            let #arg_ident: #arg_ty = match <
                                _nexustack::__private::axum_extra::TypedHeader::<
                                    <#arg_ty as _nexustack::http::headers::OptionalHeader>::Header
                                > as _nexustack::__private::axum::extract::OptionalFromRequestParts<_>
                            >::from_request_parts(
                                &mut parts,
                                &()
                            ).await.map_err(
                                |err| <_nexustack::http::headers::ExtractHeaderError as From<_nexustack::__private::axum_extra::typed_header::TypedHeaderRejection>>::from(err)
                            )? {
                                Some(_nexustack::__private::axum_extra::TypedHeader(#arg_ident)) =>  <#arg_ty as _nexustack::__private::From<<#arg_ty as _nexustack::http::headers::OptionalHeader>::Header>>::from(#arg_ident),
                                None => #path,
                            };
                        },
                    }
                }
            },
            attr::ActionArg::Cookie(cookie) => {
                // TODO: Always load a complete cookie jar
                todo!()
            },
            attr::ActionArg::User(user) => {
                // TODO: Get the current user information from request extensions / DI ?
                todo!()
            },
            attr::ActionArg::Session(session) => {
                quote_spanned! {arg_span=>
                    let #arg_ident = <
                        <
                            C::SessionExtractor as _nexustack::http::session::SessionExtractor
                        >::Session<()> as _nexustack::__private::axum::extract::FromRequestParts<()>
                    >::from_request_parts(&mut parts, &()).await?.into();
                }
            },
            attr::ActionArg::Service(service) => {
                let service_fallback = match service.default() {
                    Default::None => quote! {
                        err
                    },
                    Default::Default => quote! {
                        Ok(_nexustack::__private::Default::default())
                    },
                    Default::Path(path) => quote! {
                        Ok(#path)
                    },
                };

                // TODO: Allow Option<#arg_ty> to be used as a way to inject optional services
                quote_spanned! {arg_span=>
                    let #arg_ident: #arg_ty = match parts
                        .extensions
                        .get::<_nexustack::inject::ServiceScope>()
                        .expect("ServiceScope is registered in a middleware for each request")
                        .service_provider()
                        .resolve::<#arg_ty>() {
                            #[allow(unused_variables)]
                            err @ Err(_nexustack::inject::InjectionError::ServiceNotFound {..}) => #service_fallback,
                            result => result,
                        }?;
                }
            },
            // Processed below
            attr::ActionArg::Body(_)
            | attr::ActionArg::Request(_) => {
                quote!()
            },
        }
    });

    let arg_impls = arg_impls.chain(action.args.iter().map(|arg| {
        let arg_span = arg.original.span();
        let arg_ident = &arg.ident;
        let arg_ty = &arg.original.ty;
        match &arg.attrs {
            // Already processed above
            attr::ActionArg::Param(_)
            | attr::ActionArg::IpAddress(_)
            | attr::ActionArg::Query(_)
            | attr::ActionArg::Header(_)
            | attr::ActionArg::Cookie(_)
            | attr::ActionArg::User(_)
            | attr::ActionArg::Session(_)
            | attr::ActionArg::Service(_) => {
                quote!()
            },
            attr::ActionArg::Body(body) => {
                let decoder = body
                    .decoder()
                    .map(|ty| ty.into_token_stream())
                    .unwrap_or_else(|| quote!(_nexustack::http::decoding::DefaultDecoder));
                quote_spanned! {arg_span=>
                    let req = _nexustack::__private::axum::http::Request::<_nexustack::__private::axum::body::Body>::from_parts(parts, body);

                    let #arg_ident: #arg_ty = <
                        #decoder as _nexustack::http::decoding::Decoder
                    >::decode_request :: < #arg_ty >(req).await?;
                }
            },
            attr::ActionArg::Request(_) => {
                quote_spanned! {arg_span=>
                    let #arg_ident: #arg_ty = _nexustack::__private::axum::http::Request::<_nexustack::__private::axum::body::Body>::from_parts(parts, body);
                }
            },
        }
    }));

    let param_cont = build_param_container(action);
    let query_cont = build_query_container(action);

    let param_extract = if action
        .args
        .iter()
        .any(|arg| matches!(&arg.attrs, attr::ActionArg::Param(_)))
    {
        quote_spanned! {action_span=>
            let _nexustack::__private::axum::extract::Path(__param_container) = <
                _nexustack::__private::axum::extract::Path::<__ParamContainer> as _nexustack::__private::axum::extract::FromRequestParts<_>
            >::from_request_parts(&mut parts, &()).await?;
        }
    } else {
        quote!()
    };

    let query_extract = if action
        .args
        .iter()
        .any(|arg| matches!(&arg.attrs, attr::ActionArg::Query(_)))
    {
        quote_spanned! {action_span=>
            let _nexustack::__private::axum_extra::extract::Query(__query_container) = <
                _nexustack::__private::axum_extra::extract::Query::<__QueryContainer> as _nexustack::__private::axum::extract::FromRequestParts<_>
            >::from_request_parts(&mut parts, &()).await?;
        }
    } else {
        quote!()
    };

    let openapi_args = action.args.iter().map(|arg| {
        let arg_span = arg.original.span();
        let arg_ident = &arg.ident;
        let arg_ty = &arg.original.ty;
        let arg_ident_str = arg_ident.to_string();
        match &arg.attrs {
            attr::ActionArg::Param(param) => {
                if param.api_skip() {
                    quote!()
                } else {
                    let param_description = param.description();
                    let param_deprecated = param.deprecated();
                    let param_rename = param.name().to_string();
                    quote_spanned! {arg_span=>
                        _nexustack::openapi::http::HttpOperationBuilder::collect_path_parameter(
                            &mut operation_builder,
                            #param_rename,
                            Some(#param_description),
                            #param_deprecated,
                            <#arg_ty as _nexustack::openapi::Schema>::describe,
                        )?;
                    }
                }
            }
            attr::ActionArg::Body(body) => {
                let body_description = body.description();
                let body_deprecated = body.deprecated();
                let decoder = body
                    .decoder()
                    .map(|ty| ty.into_token_stream())
                    .unwrap_or_else(|| quote!(_nexustack::http::decoding::DefaultDecoder));
                quote_spanned! {arg_span=>
                    _nexustack::openapi::http::HttpOperationBuilder::collect_request_body(
                        &mut operation_builder,
                        Some(#body_description),
                        #body_deprecated,
                        None,
                        <#decoder as _nexustack::openapi::http::HttpContentType< #arg_ty >>::describe,
                    )?;
                }
            }
            attr::ActionArg::Query(query) => {
                if query.api_skip() {
                    quote!()
                } else {
                    let query_description = query.description();
                    let query_deprecated = query.deprecated();
                    let query_rename = query.name().name().to_string();
                    let query_required = query.default().is_none();
                    quote_spanned! {arg_span=>
                        _nexustack::openapi::http::HttpOperationBuilder::collect_query_parameter(
                            &mut operation_builder,
                            #query_rename,
                            Some(#query_description),
                            #query_deprecated,
                            Some(#query_required),
                            <#arg_ty as _nexustack::openapi::Schema>::describe,
                        )?;
                    }
                }
            }
            attr::ActionArg::Header(header) => {
                if header.api_skip() {
                    quote!()
                } else {
                    let header_description = header.description();
                    let header_deprecated = header.deprecated();
                    let header_rename = header.name().map(|name| name.to_string());
                    let header_required = header.default().is_none();

                    if let Some(header_rename) = header_rename {
                        let header_rename = header_rename.to_lowercase();
                        quote_spanned! {arg_span=>
                            _nexustack::openapi::http::HttpOperationBuilder::collect_header_parameter(
                                &mut operation_builder,
                                #header_rename,
                                Some(#header_description),
                                #header_deprecated,
                                Some(#header_required),
                                <#arg_ty as _nexustack::openapi::Schema>::describe,
                            )?;
                        }
                    } else {
                        quote_spanned! {arg_span=>
                            _nexustack::openapi::http::HttpOperationBuilder::collect_header_parameter(
                                &mut operation_builder,
                                // TODO: Optional header
                                <<#arg_ty as _nexustack::http::headers::OptionalHeader>::Header as _nexustack::http::headers::Header>::name().as_str(),
                                Some(#header_description),
                                #header_deprecated,
                                Some(#header_required),
                                <#arg_ty as _nexustack::openapi::Schema>::describe,
                            )?;
                        }
                    }
                }
            }
            attr::ActionArg::Cookie(cookie) => {
                let cookie_description = cookie.description();
                let cookie_deprecated = cookie.deprecated();
                quote_spanned! {arg_span=>
                    _nexustack::openapi::http::HttpOperationBuilder::collect_cookie_parameter(
                        &mut operation_builder,
                        #arg_ident_str,
                        Some(#cookie_description),
                        #cookie_deprecated,
                        None,
                        <#arg_ty as _nexustack::openapi::Schema>::describe,
                    )?;
                }
            }
            attr::ActionArg::Session(_)
            | attr::ActionArg::User(_)
            | attr::ActionArg::IpAddress(_)
            | attr::ActionArg::Service(_)
            | attr::ActionArg::Request(_) => quote!(),
        }
    });

    let args = action.args.iter().map(|arg| &arg.ident).collect::<Vec<_>>();

    let response_bound = quote_spanned! {response.span()=>
        #response: _nexustack::openapi::http::HttpResponse,
    };

    let cont_tags = cont_attrs.tags();

    let compiled_context_constraints = if context_constraints.is_empty() {
        quote!()
    } else {
        quote! {
            where #(#context_constraints)*
        }
    };

    (
        quote_spanned! {action_span=>
            struct #endpoint_ident(#cont_path);

            const _: () = {

                static __callsite: _nexustack::__private::utils::AtomicOnceCell<_nexustack::Callsite> =
                    _nexustack::__private::utils::AtomicOnceCell::new();

                #param_cont
                #query_cont

                impl _nexustack::inject::FromInjector for #endpoint_ident {
                    #[inline]
                    fn from_injector(injector: &_nexustack::inject::Injector) -> _nexustack::inject::ConstructionResult<Self>
                    where
                        Self: Sized,
                    {
                        let service_provider = injector.resolve::<_nexustack::inject::ServiceProvider>()?;
                        let controller = service_provider.construct()?;
                        Ok(Self(controller))
                    }
                }

                impl<C: _nexustack::http::HttpEndpointContext> _nexustack::http::HttpEndpoint<C> for #endpoint_ident #compiled_context_constraints {
                    type Request = _nexustack::__private::axum::extract::Request<_nexustack::__private::axum::body::Body>;
                    type Response = #response;
                    type Routes = [&'static str; 1];

                    #[inline]
                    fn method() -> _nexustack::http::HttpMethod {
                        #method
                    }

                    #[inline]
                    fn routes() -> Self::Routes {
                        [#route]
                    }

                    #[inline]
                    #[allow(unused, unused_mut)]
                    async fn handle(&mut self, request: Self::Request, context: C) -> Self::Response {
                        #[allow(unused_variables)]
                        let (mut parts, body) = request.into_parts();

                        #param_extract
                        #query_extract

                        #(
                            #arg_impls
                        )*

                        #cont_path :: #action_ident ( #receiver #(#args),*).await
                    }
                }

                impl _nexustack::openapi::http::HttpOperation for #endpoint_ident {
                    type Context = ();

                    #[inline]
                    #[allow(unused_mut)]
                    fn describe<B>(mut operation_builder: B, _context: ()) -> Result<B::Ok, B::Error>
                    where
                        B: _nexustack::openapi::http::HttpOperationBuilder,
                        #response_bound
                    {

                        #(
                            #openapi_args
                        )*

                        _nexustack::openapi::http::HttpOperationBuilder::collect_response(
                            operation_builder,
                            _nexustack::openapi::http::HttpOperationId::new(#openapi_operation_name, *__callsite.get_or_init(|| #action_callsite)),
                            #openapi_method,
                            #route,
                            Some([#(#cont_tags),*]),
                            Some(#action_description),
                            #action_deprecated,
                            <#response as _nexustack::openapi::http::HttpResponse>::describe
                        )
                    }
                }
            };
        },
        context_constraints,
    )
}

fn build_param_container(action: &Action) -> TokenStream {
    let fields = action
        .args
        .iter()
        .filter_map(|arg| {
            let arg_span = arg.original.span();
            let arg_ident = &arg.ident;
            let arg_ty = &arg.original.ty;
            match &arg.attrs {
                attr::ActionArg::Param(param) => {
                    let name = param.name();
                    let serde_rename_attr = quote! { #[serde(rename = #name)] };

                    Some(quote_spanned! {arg_span=>
                        #serde_rename_attr
                        #arg_ident: #arg_ty,
                    })
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    let action_span = action.original.span();

    if !fields.is_empty() {
        quote_spanned! {action_span=>
            // TODO: Hygiene
            #[derive(Debug, PartialEq, _nexustack::__private::serde::Deserialize)]
            struct __ParamContainer {
                #(#fields)*
            }
        }
    } else {
        quote!()
    }
}

fn build_query_container(action: &Action) -> TokenStream {
    let fields = action
        .args
        .iter()
        .filter_map(|arg| {
            let arg_span = arg.original.span();
            let arg_ident = &arg.ident;
            let arg_ty = &arg.original.ty;
            match &arg.attrs {
                attr::ActionArg::Query(query) => {
                    let name = query.name().name();
                    let serde_rename_attr = quote! { #[serde(rename = #name)] };

                    let aliases = query
                        .name()
                        .aliases()
                        .iter()
                        .map(|alias| quote! { alias = #alias});
                    let serde_aliases_attr = quote! {
                        #[serde( #( #aliases),* )]
                    };

                    let serde_default_attr = match query.default() {
                        Default::None => quote! {},
                        Default::Default => quote! { #[serde(default)] },
                        Default::Path(path) => quote! { #[serde(default = #path)] },
                    };

                    Some(quote_spanned! {arg_span=>
                        #serde_rename_attr
                        #serde_aliases_attr
                        #serde_default_attr
                        #arg_ident: #arg_ty,
                    })
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    let action_span = action.original.span();

    if !fields.is_empty() {
        quote_spanned! {action_span=>
            // TODO: Hygiene
            #[derive(Debug, PartialEq, _nexustack::__private::serde::Deserialize)]
            struct __QueryContainer {
                #(#fields)*
            }
        }
    } else {
        quote!()
    }
}
