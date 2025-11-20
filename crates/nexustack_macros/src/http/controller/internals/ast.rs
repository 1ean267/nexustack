/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/ast.rs
 */

use crate::{
    http::controller::internals::attr::{self, HttpMethod},
    internals::Ctxt,
};
use proc_macro2::TokenStream;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use syn::spanned::Spanned;

pub struct Controller {
    pub ty: syn::TypePath,
    pub attrs: attr::Controller,
    pub actions: Vec<Action>,
}

/// A action of an controller.
pub struct Action {
    pub attrs: attr::Action,
    pub args: Vec<ActionArg>,
    pub original: syn::ImplItemFn,
}

pub struct ActionArg {
    pub ident: syn::Ident,
    pub attrs: attr::ActionArg,
    pub original: syn::PatType,
}

impl<'a> Controller {
    /// Convert the raw Syn ast into a parsed container object, collecting errors in `cx`.
    pub fn from_ast(cx: &Ctxt, attr: TokenStream, item: &'a mut syn::ItemImpl) -> Option<Self> {
        if item.unsafety.is_some() {
            cx.error_spanned_by(item, "Unsafe impl blocks are not supported");
            return None;
        }

        if item.trait_.is_some() {
            cx.error_spanned_by(item, "Trait impl blocks are not supported");
            return None;
        }

        // TODO: Do we allow generics on controllers?

        let ty = match item.self_ty.as_ref() {
            syn::Type::Path(type_path) => type_path,
            _ => {
                cx.error_spanned_by(
                    &item.self_ty,
                    "Only type paths are supported for controller impl blocks",
                );
                return None;
            }
        };

        let controller_attrs = attr::Controller::from_ast(cx, attr, item);

        let actions = item
            .items
            .iter_mut()
            .filter_map(|impl_item| {
                if let syn::ImplItem::Fn(action) = impl_item {
                    map_action(cx, action, controller_attrs.api_skip())
                } else {
                    None
                }
            })
            .collect();

        let item = Controller {
            ty: ty.clone(),
            attrs: controller_attrs,
            actions,
        };
        // check::check(cx, &mut item);
        Some(item)
    }
}

fn map_action(
    cx: &Ctxt,
    action: &mut syn::ImplItemFn,
    controller_api_skip: bool,
) -> Option<Action> {
    let original = action.clone();
    let attrs = attr::Action::from_ast(cx, action, controller_api_skip)?;

    if let syn::Visibility::Public(_) = action.vis {
    } else {
        cx.error_spanned_by(&action.vis, "Action must be public");
        return None;
    }

    if action.sig.abi.is_some() {
        cx.error_spanned_by(&action.sig.abi, "Action must not have a custom ABI");
        return None;
    }

    if !action.sig.generics.params.is_empty() {
        cx.error_spanned_by(
            &action.sig.generics,
            "Action must not have generic parameters",
        );
        return None;
    }

    if action.sig.generics.where_clause.is_some() {
        cx.error_spanned_by(
            &action.sig.generics.where_clause,
            "Action must not have a where clause",
        );
        return None;
    }

    if action.sig.variadic.is_some() {
        cx.error_spanned_by(&action.sig.variadic, "Action must not be variadic");
        return None;
    }

    if action.sig.unsafety.is_some() {
        cx.error_spanned_by(action.sig.unsafety, "Action must not be unsafe");
        return None;
    }

    // TODO: This should not be the case. This is for simplicity in the first iteration.
    if action.sig.asyncness.is_none() {
        cx.error_spanned_by(action.sig.asyncness, "Action must be async");
        return None;
    }

    if let Some(receiver) = action.sig.receiver() {
        if receiver.reference.is_none() {
            cx.error_spanned_by(
                receiver,
                "Action receiver must be either &self or &mut self",
            );
            return None;
        }

        if receiver.colon_token.is_some() {
            cx.error_spanned_by(
                receiver,
                "Action receiver must be either &self or &mut self",
            );
            return None;
        }
    }

    let args = action
        .sig
        .inputs
        .iter_mut()
        .filter_map(|input| {
            map_action_arg(
                cx,
                input,
                attrs.method(),
                attrs.parameter_descriptions(),
                attrs.api_skip() || controller_api_skip,
            )
        })
        .collect::<Vec<_>>();

    let mut has_body_arg = false;
    let mut has_session_arg = false;
    let mut has_request_arg = false;
    let mut has_user_arg = false;
    let mut has_ip_address_arg = false;

    for arg in &args {
        if let attr::ActionArg::Body(_) = arg.attrs {
            if has_body_arg {
                cx.error_spanned_by(&arg.original, "Duplicate body argument");
                return None;
            }

            if attrs.method() == HttpMethod::Get {
                cx.error_spanned_by(&arg.original, "GET actions must not have a body argument");
                return None;
            }

            if has_request_arg {
                cx.error_spanned_by(&arg.original, "Cannot have both request and body arguments");
                return None;
            }

            has_body_arg = true;
        }

        if let attr::ActionArg::Session(_) = arg.attrs {
            if has_session_arg {
                cx.error_spanned_by(&arg.original, "Duplicate session argument");
                return None;
            }
            has_session_arg = true;
        }

        if let attr::ActionArg::Request(_) = arg.attrs {
            if has_request_arg {
                cx.error_spanned_by(&arg.original, "Duplicate request argument");
                return None;
            }

            if has_body_arg {
                cx.error_spanned_by(&arg.original, "Cannot have both request and body arguments");
                return None;
            }

            has_request_arg = true;
        }

        if let attr::ActionArg::User(_) = arg.attrs {
            if has_user_arg {
                cx.error_spanned_by(&arg.original, "Duplicate user argument");
                return None;
            }
            has_user_arg = true;
        }

        if let attr::ActionArg::IpAddress(_) = arg.attrs {
            if has_ip_address_arg {
                cx.error_spanned_by(&arg.original, "Duplicate ip-address argument");
                return None;
            }
            has_ip_address_arg = true;
        }
    }

    let action_args = args
        .iter()
        .filter_map(|arg| {
            if let attr::ActionArg::Param(param) = &arg.attrs {
                Some(param.name().value.clone())
            } else {
                None
            }
        })
        .collect::<HashSet<_>>();

    let route_parameters = extract_path_parameters(attrs.route().value.as_str());

    let missing_params = route_parameters
        .difference(&action_args)
        .collect::<Vec<_>>();
    let extra_params = action_args
        .difference(&route_parameters)
        .collect::<Vec<_>>();

    if !missing_params.is_empty() {
        for param in missing_params {
            cx.error_spanned_by(
                attrs.route(),
                format!(
                    "Missing #[param] argument for path parameter '{{{}}}'",
                    param
                ),
            );
        }
    }

    // TODO: Use the correct span here
    if !extra_params.is_empty() {
        for param in extra_params {
            let arg = args.iter().find(|arg| {
                if let attr::ActionArg::Param(p) = &arg.attrs {
                    p.name().value == *param
                } else {
                    false
                }
            });
            cx.error(
                arg.map(|arg| arg.original.span())
                    .unwrap_or(action.sig.span()),
                format!(
                    "Path parameter '{{{}}}' not found in route '{}'",
                    param,
                    attrs.route()
                ),
            );
        }
    }

    Some(Action {
        attrs,
        args,
        original,
    })
}

fn map_action_arg(
    cx: &Ctxt,
    arg: &mut syn::FnArg,
    method: HttpMethod,
    parameter_descriptions: &HashMap<String, String>,
    action_api_skip: bool,
) -> Option<ActionArg> {
    let pat_type = match arg {
        syn::FnArg::Receiver(_) => {
            return None;
        }
        syn::FnArg::Typed(pat_type) => pat_type,
    };
    let original = pat_type.clone();

    let ident = match pat_type.pat.as_ref() {
        syn::Pat::Ident(pat_ident) => Some(pat_ident.ident.clone()),
        _ => None,
    };

    let description = if let Some(ident) = &ident {
        parameter_descriptions.get(&ident.to_string()).cloned()
    } else {
        None
    };

    let attrs = attr::ActionArg::from_ast(cx, pat_type, method, description, action_api_skip)?;

    let ident = match ident {
        Some(ident) => ident,
        _ => {
            cx.error_spanned_by(
                &pat_type.pat,
                "Only identifier patterns are supported for action arguments",
            );
            return None;
        }
    };

    Some(ActionArg {
        attrs,
        ident,
        original,
    })
}

/// Extracts all path parameters from a URI template.
///
/// # Arguments
///
/// * `uri_template` - A string slice that holds the URI template.
///
/// # Returns
///
/// A vector of strings representing the path parameters.
///
/// # Example
///
/// ```
/// let params = extract_path_parameters("/api/client_info/{a}/test/{b}");
/// assert_eq!(params, vec!["a", "b"]);
/// ```
pub fn extract_path_parameters(uri_template: &str) -> HashSet<String> {
    // Regular expression to match path parameters in curly braces
    let re = Regex::new(r"\{([^}]+)\}").unwrap();

    // Collect all matches into a vector
    re.captures_iter(uri_template)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}
