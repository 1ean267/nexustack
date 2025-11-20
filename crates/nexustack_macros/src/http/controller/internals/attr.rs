/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/attr.rs
 */

use crate::{
    http::controller::internals::route::Route,
    internals::{
        Ctxt,
        attr::{
            Attr, VecAttr, get_lit_str, get_lit_str2_expr, parse_lit_into_bool,
            parse_lit_into_expr_path, parse_lit_into_path, parse_lit_into_ty,
        },
        default::Default,
        name::Name,
        symbol::*,
    },
};
use docstrings::{DocSection, parse_md_docblock};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident};
use std::collections::{BTreeSet, HashMap};
use syn::{Ident, Token, parse::Parser as _};

fn unraw(ident: &Ident) -> Ident {
    Ident::new(ident.to_string().trim_start_matches("r#"), ident.span())
}

///////// Controller /////////

/// Represents struct or enum attribute information.
#[derive(Debug)]
pub struct Controller {
    api_skip: bool,
    crate_path: Option<syn::Path>,
    deprecated: bool,
    description: String,
    tags: Vec<String>,
}

impl Controller {
    pub fn from_ast(cx: &Ctxt, meta: TokenStream, item: &syn::ItemImpl) -> Self {
        let mut api_skip = Attr::none(cx, API_SKIP);
        let mut crate_path = Attr::none(cx, CRATE);
        let mut deprecated = Attr::none(cx, DEPRECATED);
        let mut description = Attr::none(cx, DESCRIPTION);
        let mut tags = Attr::none(cx, TAGS);

        if !meta.is_empty() {
            let parser = syn::meta::parser(|meta| {
                if meta.path == API_SKIP {
                    if meta.input.peek(Token![=]) {
                        // #[http_controller(api_skip = "...")]
                        if let Some(value) = parse_lit_into_bool(cx, API_SKIP, &meta)? {
                            api_skip.set(&meta.path, value)
                        }
                    } else {
                        // #[http_controller(api_skip)]
                        api_skip.set(&meta.path, true)
                    }
                } else if meta.path == CRATE {
                    // #[http_controller(crate = "foo")]
                    if let Some(path) = parse_lit_into_path(cx, CRATE, &meta)? {
                        crate_path.set(&meta.path, path);
                    }
                } else if meta.path == DEPRECATED {
                    if meta.input.peek(Token![=]) {
                        // #[http_controller(deprecated = "...")]
                        if let Some(value) = parse_lit_into_bool(cx, DEPRECATED, &meta)? {
                            deprecated.set(&meta.path, value)
                        }
                    } else {
                        // #[http_controller(deprecated)]
                        deprecated.set(&meta.path, true)
                    }
                } else if meta.path == DESCRIPTION {
                    // #[http_controller(description = "...")]
                    if let Some(s) = get_lit_str(cx, DESCRIPTION, &meta)? {
                        description.set(&meta.path, s.value());
                    }
                } else if meta.path == TAGS {
                    // #[http_controller(tags = "...")]
                    if let Some(s) = get_lit_str(cx, TAGS, &meta)? {
                        tags.set(
                            &meta.path,
                            s.value().split(',').map(|s| s.trim().to_string()).collect(),
                        );
                    }
                } else {
                    let path = meta.path.to_token_stream().to_string().replace(' ', "");
                    return Err(meta.error(format_args!("unknown controller attribute `{path}`")));
                }
                Ok(())
            });

            // Parse
            let parse_res = parser.parse2(meta);
            if let Err(err) = parse_res {
                cx.syn_error(err);
            }
        }

        for attr in &item.attrs {
            if matches!(&attr.meta, syn::Meta::Path(path) if path == DEPRECATED) {
                deprecated.set_if_none(true);
            }

            if let syn::Meta::NameValue(meta) = &attr.meta
                && meta.path == DOC
                && let Ok(Some(s)) = get_lit_str2_expr(cx, DOC, DOC, &meta.value)
            {
                description.set_if_none(s.value().trim().to_string());
            }
        }

        let api_skip = api_skip.get().unwrap_or(false);

        Self {
            api_skip,
            crate_path: crate_path.get(),
            deprecated: deprecated.get().unwrap_or(false),
            description: match description.get() {
                Some(description) => description,
                None => {
                    if !api_skip {
                        cx.error_spanned_by(item, "No description provided");
                    }
                    String::new()
                }
            },
            tags: tags.get().unwrap_or_default(),
        }
    }

    pub fn api_skip(&self) -> bool {
        self.api_skip
    }

    pub fn custom_crate_path(&self) -> Option<&syn::Path> {
        self.crate_path.as_ref()
    }

    pub fn deprecated(&self) -> bool {
        self.deprecated
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }
}

///////// Action /////////

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HttpMethod {
    /// HTTP GET method.
    Get,
    /// HTTP POST method.
    Post,
    /// HTTP PUT method.
    Put,
    /// HTTP DELETE method.
    Delete,
    /// HTTP PATCH method.
    Patch,
    /// HTTP OPTIONS method.
    Options,
    /// HTTP HEAD method.
    Head,
    /// HTTP TRACE method.
    Trace,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "get",
            HttpMethod::Post => "post",
            HttpMethod::Put => "put",
            HttpMethod::Delete => "delete",
            HttpMethod::Patch => "patch",
            HttpMethod::Options => "options",
            HttpMethod::Head => "head",
            HttpMethod::Trace => "trace",
        }
    }

    fn from_attr(attr: &syn::Attribute) -> Option<Self> {
        let path = attr.path();

        if path.leading_colon.is_some() {
            return None;
        }

        if path.segments.len() > 4
            || path
                .segments
                .iter()
                .any(|segment| !segment.arguments.is_none())
        {
            return None;
        }

        let arg = match path.segments.last() {
            Some(segment) => {
                if segment.ident == Self::Get.as_str() {
                    Self::Get
                } else if segment.ident == Self::Post.as_str() {
                    Self::Post
                } else if segment.ident == Self::Put.as_str() {
                    Self::Put
                } else if segment.ident == Self::Delete.as_str() {
                    Self::Delete
                } else if segment.ident == Self::Patch.as_str() {
                    Self::Patch
                } else if segment.ident == Self::Options.as_str() {
                    Self::Options
                } else if segment.ident == Self::Head.as_str() {
                    Self::Head
                } else if segment.ident == Self::Trace.as_str() {
                    Self::Trace
                } else {
                    return None;
                }
            }
            None => return None,
        };

        if path.segments.len() > 1 && path.segments[path.segments.len() - 2].ident != "controller" {
            return None;
        }

        if path.segments.len() > 2 && path.segments[path.segments.len() - 3].ident != "http" {
            return None;
        }

        if path.segments.len() > 3 && path.segments[path.segments.len() - 4].ident != "nexustack" {
            return None;
        }

        Some(arg)
    }
}

pub struct Action {
    method: HttpMethod,
    route: Route,
    api_skip: bool,
    deprecated: bool,
    description: String,
    parameter_descriptions: HashMap<String, String>,
}

impl Action {
    pub fn from_ast(
        cx: &Ctxt,
        action: &mut syn::ImplItemFn,
        controller_api_skip: bool,
    ) -> Option<Self> {
        let method = match action.attrs.iter().find_map(HttpMethod::from_attr) {
            Some(method) => method,
            None => {
                return None;
            }
        };

        let mut api_skip = Attr::none(cx, API_SKIP);
        let mut description = Attr::none(cx, DESCRIPTION);
        let mut deprecated = Attr::none(cx, DESCRIPTION);
        let mut route = Attr::none(cx, ROUTE);

        for i in (0..action.attrs.len()).rev() {
            let attr = &action.attrs[i];

            if let Some(attr_method) = HttpMethod::from_attr(attr) {
                if method != attr_method {
                    cx.error_spanned_by(
                        attr,
                        format_args!(
                            "Mismatched action type. Expected `{}` but found `{}`",
                            method.as_str(),
                            attr_method.as_str()
                        ),
                    );
                    continue;
                }
            } else {
                continue;
            }

            if let syn::Meta::List(meta) = &attr.meta {
                if meta.tokens.is_empty() {
                    action.attrs.remove(i);
                    continue;
                }
            } else if let syn::Meta::Path(_) = &attr.meta {
                action.attrs.remove(i);
                continue;
            }

            if let Err(err) = attr.parse_nested_meta(|meta| {
                if meta.path == API_SKIP {
                    if meta.input.peek(Token![=]) {
                        // #[get(api_skip = "...")]
                        if let Some(value) = parse_lit_into_bool(cx, API_SKIP, &meta)? {
                            api_skip.set(&meta.path, value)
                        }
                    } else {
                        // #[get(api_skip)]
                        api_skip.set(&meta.path, true)
                    }
                } else if meta.path == DESCRIPTION {
                    // #[get(description = "...")]
                    if let Some(s) = get_lit_str(cx, DESCRIPTION, &meta)? {
                        description.set(&meta.path, s.value());
                    }
                } else if meta.path == DEPRECATED {
                    if meta.input.peek(Token![=]) {
                        // #[get(deprecated = "...")]
                        if let Some(value) = parse_lit_into_bool(cx, DEPRECATED, &meta)? {
                            deprecated.set(&meta.path, value)
                        }
                    } else {
                        // #[get(deprecated)]
                        deprecated.set(&meta.path, true)
                    }
                } else if meta.path == ROUTE {
                    // #[get(route = "...")]
                    if let Some(s) = get_lit_str(cx, ROUTE, &meta)? {
                        route.set(&meta.path, Route::from(&s));
                    }
                } else {
                    let path = meta.path.to_token_stream().to_string().replace(' ', "");
                    return Err(meta.error(format_args!("unknown action attribute `{path}`")));
                }
                Ok(())
            }) {
                cx.syn_error(err);
            }

            action.attrs.remove(i);
        }

        let mut doc_comment: Option<String> = None;

        for attr in &action.attrs {
            if matches!(&attr.meta, syn::Meta::Path(path) if path == DEPRECATED) {
                deprecated.set_if_none(true);
            }

            if let syn::Meta::NameValue(meta) = &attr.meta
                && meta.path == DOC
                && let Ok(Some(s)) = get_lit_str2_expr(cx, DOC, DOC, &meta.value)
            {
                if let Some(doc_comment) = &mut doc_comment {
                    doc_comment.push('\n');
                    doc_comment.push_str(s.value().trim());
                } else {
                    doc_comment = Some(s.value().trim().to_string());
                }
            }
        }

        let parameter_descriptions = if let Some(doc_comment) = doc_comment {
            match parse_md_docblock(&doc_comment) {
                Ok(docblock) => {
                    description.set_if_none(docblock.teaser);
                    docblock
                        .sections
                        .iter()
                        .find_map(|section| match section {
                            DocSection::Parameters(parameters) => Some(parameters),
                            _ => None,
                        })
                        .cloned()
                }
                Err(err) => {
                    cx.error_spanned_by(
                        &action,
                        format_args!("Failed to parse docstring for action description: {}", err),
                    );
                    None
                }
            }
        } else {
            None
        };

        let api_skip = api_skip.get().unwrap_or(false);

        Some(Self {
            method,
            api_skip,
            deprecated: deprecated.get().unwrap_or(false),
            route: match route.get() {
                Some(route) => route,
                None => {
                    cx.error_spanned_by(&action, "No route provided");
                    Route {
                        value: String::new(),
                        span: Span::call_site(),
                    }
                }
            },
            description: match description.get() {
                Some(description) => description,
                None => {
                    if !api_skip && !controller_api_skip {
                        cx.error_spanned_by(&action, "No description provided");
                    }
                    String::new()
                }
            },
            parameter_descriptions: parameter_descriptions
                .unwrap_or_default()
                .into_iter()
                .collect(),
        })
    }

    pub fn method(&self) -> HttpMethod {
        self.method
    }

    pub fn api_skip(&self) -> bool {
        self.api_skip
    }

    pub fn deprecated(&self) -> bool {
        self.deprecated
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn route(&self) -> &Route {
        &self.route
    }

    pub fn parameter_descriptions(&self) -> &HashMap<String, String> {
        &self.parameter_descriptions
    }
}

///////// Action args /////////

#[derive(Debug, PartialEq, Eq)]
pub enum ActionArgType {
    Body,
    Session,
    Param,
    Query,
    Header,
    User,
    IpAddress,
    Service,
    Cookie,
    Request,
}

impl ActionArgType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionArgType::Body => "body",
            ActionArgType::Session => "session",
            ActionArgType::Param => "param",
            ActionArgType::Query => "query",
            ActionArgType::Header => "header",
            ActionArgType::User => "user",
            ActionArgType::IpAddress => "ip_address",
            ActionArgType::Service => "service",
            ActionArgType::Cookie => "cookie",
            ActionArgType::Request => "request",
        }
    }

    fn from_attr(attr: &syn::Attribute, method: &HttpMethod) -> Option<Self> {
        let path = attr.path();

        if path.leading_colon.is_some() {
            return None;
        }

        if path.segments.len() > 5
            || path
                .segments
                .iter()
                .any(|segment| !segment.arguments.is_none())
        {
            return None;
        }

        let arg = match path.segments.last() {
            Some(segment) => {
                if segment.ident == ActionArgType::Body.as_str() {
                    ActionArgType::Body
                } else if segment.ident == ActionArgType::Session.as_str() {
                    ActionArgType::Session
                } else if segment.ident == ActionArgType::Param.as_str() {
                    ActionArgType::Param
                } else if segment.ident == ActionArgType::Query.as_str() {
                    ActionArgType::Query
                } else if segment.ident == ActionArgType::Header.as_str() {
                    ActionArgType::Header
                } else if segment.ident == ActionArgType::User.as_str() {
                    ActionArgType::User
                } else if segment.ident == ActionArgType::IpAddress.as_str() {
                    ActionArgType::IpAddress
                } else if segment.ident == ActionArgType::Service.as_str() {
                    ActionArgType::Service
                } else if segment.ident == ActionArgType::Cookie.as_str() {
                    ActionArgType::Cookie
                } else {
                    return None;
                }
            }
            None => return None,
        };

        let method = method.as_str();

        if path.segments.len() > 1 && path.segments[path.segments.len() - 2].ident != method {
            return None;
        }

        if path.segments.len() > 2 && path.segments[path.segments.len() - 3].ident != "controller" {
            return None;
        }

        if path.segments.len() > 3 && path.segments[path.segments.len() - 4].ident != "http" {
            return None;
        }

        if path.segments.len() > 4 && path.segments[path.segments.len() - 5].ident != "nexustack" {
            return None;
        }

        Some(arg)
    }

    pub fn arg_from_ast(
        &self,
        cx: &Ctxt,
        arg: &mut syn::PatType,
        method: HttpMethod,
        description: Option<String>,
        action_api_skip: bool,
    ) -> ActionArg {
        match self {
            ActionArgType::Body => ActionArg::Body(Body::from_ast(
                cx,
                arg,
                method,
                description,
                action_api_skip,
            )),
            ActionArgType::Session => ActionArg::Session(Session::from_ast(
                cx,
                arg,
                method,
                description,
                action_api_skip,
            )),
            ActionArgType::Param => ActionArg::Param(Param::from_ast(
                cx,
                arg,
                method,
                description,
                action_api_skip,
            )),
            ActionArgType::Query => ActionArg::Query(Query::from_ast(
                cx,
                arg,
                method,
                description,
                action_api_skip,
            )),
            ActionArgType::Header => ActionArg::Header(Header::from_ast(
                cx,
                arg,
                method,
                description,
                action_api_skip,
            )),
            ActionArgType::User => ActionArg::User(User::from_ast(
                cx,
                arg,
                method,
                description,
                action_api_skip,
            )),
            ActionArgType::IpAddress => ActionArg::IpAddress(IpAddress::from_ast(
                cx,
                arg,
                method,
                description,
                action_api_skip,
            )),
            ActionArgType::Service => ActionArg::Service(Service::from_ast(
                cx,
                arg,
                method,
                description,
                action_api_skip,
            )),
            ActionArgType::Cookie => ActionArg::Cookie(Cookie::from_ast(
                cx,
                arg,
                method,
                description,
                action_api_skip,
            )),
            ActionArgType::Request => ActionArg::Request(Request::from_ast(
                cx,
                arg,
                method,
                description,
                action_api_skip,
            )),
        }
    }
}

pub enum ActionArg {
    Body(Body),
    Session(Session),
    Param(Param),
    Query(Query),
    Header(Header),
    User(User),
    IpAddress(IpAddress),
    Service(Service),
    Cookie(Cookie),
    Request(Request),
}

impl ActionArg {
    pub fn from_ast(
        cx: &Ctxt,
        arg: &mut syn::PatType,
        method: HttpMethod,
        description: Option<String>,
        action_api_skip: bool,
    ) -> Option<Self> {
        let attrs = &arg.attrs;

        let action_arg_type = attrs
            .iter()
            .find_map(|attr| ActionArgType::from_attr(attr, &method));

        if let Some(action_arg_type) = action_arg_type {
            Some(action_arg_type.arg_from_ast(cx, arg, method, description, action_api_skip))
        } else {
            cx.error_spanned_by(
                arg,
                format_args!(
                    "No valid action argument attribute found for HTTP method `{}`",
                    method.as_str()
                ),
            );
            None
        }
    }
}

macro_rules! action_args {
    ( $($ident:ident),* $(,)? ) => {
        $(
            pub struct $ident {
                api_skip: bool,
                deprecated: bool,
                description: String,
            }

            impl $ident {
                pub fn from_ast(
                    cx: &Ctxt,
                    arg: &mut syn::PatType,
                    method: HttpMethod,
                    description: Option<String>,
                    action_api_skip: bool,
                ) -> Self {
                    let attrs = &mut arg.attrs;
                    let mut api_skip = Attr::none(cx, API_SKIP);
                    let mut description = if let Some(description) = description {
                        Attr::some(cx, DESCRIPTION, description)
                    } else {
                        Attr::none(cx, DESCRIPTION)
                    };
                    let mut deprecated = Attr::none(cx, DEPRECATED);

                    for i in (0..attrs.len()).rev() {
                        let attr = &attrs[i];

                        let arg_type = Self::arg_type();

                        if let Some(attr_arg_type) = ActionArgType::from_attr(attr, &method) {
                            if arg_type != attr_arg_type {
                                cx.error_spanned_by(
                                    attr,
                                    format_args!(
                                        "Mismatched action argument type. Expected `{}` but found `{}`",
                                        arg_type.as_str(),
                                        attr_arg_type.as_str()
                                    ),
                                );
                                continue;
                            }
                        } else {
                            continue;
                        }

                        if let syn::Meta::List(meta) = &attr.meta {
                            if meta.tokens.is_empty() {
                                attrs.remove(i);
                                continue;
                            }
                        } else if let syn::Meta::Path(_) = &attr.meta {
                            attrs.remove(i);
                            continue;
                        }

                        if let Err(err) = attr.parse_nested_meta(|meta| {
                            if meta.path == API_SKIP {
                                if meta.input.peek(Token![=]) {
                                    // #[param(api_skip = "...")]
                                    if let Some(value) = parse_lit_into_bool(cx, API_SKIP, &meta)? {
                                        api_skip.set(&meta.path, value)
                                    }
                                } else {
                                    // #[param(api_skip)]
                                    api_skip.set(&meta.path, true)
                                }
                            } else if meta.path == DESCRIPTION {
                                // #[param(description = "...")]
                                if let Some(s) = get_lit_str(cx, DESCRIPTION, &meta)? {
                                    description.set(&meta.path, s.value());
                                }
                            } else if meta.path == DEPRECATED {
                                if meta.input.peek(Token![=]) {
                                    // #[param(deprecated = "...")]
                                    if let Some(value) = parse_lit_into_bool(cx, DEPRECATED, &meta)? {
                                        deprecated.set(&meta.path, value)
                                    }
                                } else {
                                    // #[param(deprecated)]
                                    deprecated.set(&meta.path, true)
                                }
                            } else {
                                let path = meta.path.to_token_stream().to_string().replace(' ', "");
                                return Err(meta.error(format_args!("unknown action argument attribute `{path}`")));
                            }
                            Ok(())
                        }) {
                            cx.syn_error(err);
                        }

                        attrs.remove(i);
                    }

                    for attr in attrs {
                        if matches!(&attr.meta, syn::Meta::Path(path) if path == DEPRECATED) {
                            deprecated.set_if_none(true);
                        }
                    }

                    let api_skip = api_skip.get().unwrap_or(false);

                    Self {
                        api_skip,
                        deprecated: deprecated.get().unwrap_or(false),
                        description: match description.get() {
                            Some(description) => description,
                            None => {
                                if !action_api_skip && !api_skip {
                                    cx.error_spanned_by(arg, "No description provided");
                                }
                                String::new()
                            }
                        },
                    }
                }

                pub fn api_skip(&self) -> bool {
                    self.api_skip
                }

                pub fn deprecated(&self) -> bool {
                    self.deprecated
                }

                pub fn description(&self) -> &str {
                    &self.description
                }

                fn arg_type() -> ActionArgType {
                    ActionArgType::$ident
                }
            }

        )*
    };
}

action_args! {
    Session,
    Header,
    User,
    IpAddress,
    Service,
    Cookie,
    Request,
}

pub struct ActionArgName {
    name: Name,
    renamed: bool,
    aliases: BTreeSet<Name>,
}

impl ActionArgName {
    pub(crate) fn from_attrs(
        source_name: Name,
        name: Attr<Name>,
        aliases: Option<VecAttr<Name>>,
    ) -> Self {
        let mut alias_set = BTreeSet::new();
        if let Some(aliases) = aliases {
            for alias_name in aliases.get() {
                alias_set.insert(alias_name);
            }
        }

        let de_name = name.get();
        let de_renamed = de_name.is_some();
        Self {
            name: de_name.unwrap_or(source_name),
            renamed: de_renamed,
            aliases: alias_set,
        }
    }

    /// Return the container name for the container when serializing.
    pub fn name(&self) -> &Name {
        &self.name
    }

    pub(crate) fn aliases(&self) -> &BTreeSet<Name> {
        &self.aliases
    }
}

pub struct Query {
    api_skip: bool,
    deprecated: bool,
    description: String,
    name: ActionArgName,
    default: Default,
}

impl Query {
    pub fn from_ast(
        cx: &Ctxt,
        arg: &mut syn::PatType,
        method: HttpMethod,
        description: Option<String>,
        action_api_skip: bool,
    ) -> Self {
        let attrs = &mut arg.attrs;

        let mut api_skip = Attr::none(cx, API_SKIP);
        let mut description = if let Some(description) = description {
            Attr::some(cx, DESCRIPTION, description)
        } else {
            Attr::none(cx, DESCRIPTION)
        };
        let mut deprecated = Attr::none(cx, DEPRECATED);
        let mut name = Attr::none(cx, RENAME);
        let mut aliases = VecAttr::none(cx, ALIAS);
        let mut default = Attr::none(cx, DEFAULT);

        for i in (0..attrs.len()).rev() {
            let attr = &attrs[i];

            let arg_type = Self::arg_type();

            if let Some(attr_arg_type) = ActionArgType::from_attr(attr, &method) {
                if arg_type != attr_arg_type {
                    cx.error_spanned_by(
                        attr,
                        format_args!(
                            "Mismatched action argument type. Expected `{}` but found `{}`",
                            arg_type.as_str(),
                            attr_arg_type.as_str()
                        ),
                    );
                    continue;
                }
            } else {
                continue;
            }

            if let syn::Meta::List(meta) = &attr.meta {
                if meta.tokens.is_empty() {
                    attrs.remove(i);
                    continue;
                }
            } else if let syn::Meta::Path(_) = &attr.meta {
                attrs.remove(i);
                continue;
            }

            if let Err(err) = attr.parse_nested_meta(|meta| {
                if meta.path == API_SKIP {
                    if meta.input.peek(Token![=]) {
                        // #[param(api_skip = "...")]
                        if let Some(value) = parse_lit_into_bool(cx, API_SKIP, &meta)? {
                            api_skip.set(&meta.path, value)
                        }
                    } else {
                        // #[param(api_skip)]
                        api_skip.set(&meta.path, true)
                    }
                } else if meta.path == DESCRIPTION {
                    // #[param(description = "...")]
                    if let Some(s) = get_lit_str(cx, DESCRIPTION, &meta)? {
                        description.set(&meta.path, s.value());
                    }
                } else if meta.path == DEPRECATED {
                    if meta.input.peek(Token![=]) {
                        // #[param(deprecated = "...")]
                        if let Some(value) = parse_lit_into_bool(cx, DEPRECATED, &meta)? {
                            deprecated.set(&meta.path, value)
                        }
                    } else {
                        // #[param(deprecated)]
                        deprecated.set(&meta.path, true)
                    }
                } else if meta.path == RENAME {
                    // #[param(rename = "foo")]
                    if let Some(s) = get_lit_str(cx, ALIAS, &meta)? {
                        name.set_if_none(Name::from(&s));
                        aliases.insert(&meta.path, Name::from(&s));
                    }
                } else if meta.path == ALIAS {
                    // #[param(alias = "foo")]
                    if let Some(s) = get_lit_str(cx, ALIAS, &meta)? {
                        aliases.insert(&meta.path, Name::from(&s));
                    }
                } else if meta.path == DEFAULT {
                    if meta.input.peek(Token![=]) {
                        // #[param(default = "...")]
                        if let Some(path) = parse_lit_into_expr_path(cx, DEFAULT, &meta)? {
                            default.set(&meta.path, Default::Path(path));
                        }
                    } else {
                        // #[param(default)]
                        default.set(&meta.path, Default::Default);
                    }
                } else {
                    let path = meta.path.to_token_stream().to_string().replace(' ', "");
                    return Err(
                        meta.error(format_args!("unknown action argument attribute `{path}`"))
                    );
                }
                Ok(())
            }) {
                cx.syn_error(err);
            }

            attrs.remove(i);
        }

        for attr in attrs {
            if matches!(&attr.meta, syn::Meta::Path(path) if path == DEPRECATED) {
                deprecated.set_if_none(true);
            }
        }

        let api_skip = api_skip.get().unwrap_or(false);

        let ident = match arg.pat.as_ref() {
            syn::Pat::Ident(pat_ident) => Some(pat_ident.ident.clone()),
            _ => None,
        }
        .unwrap_or(format_ident!("unreachable"));

        Self {
            api_skip,
            deprecated: deprecated.get().unwrap_or(false),
            description: match description.get() {
                Some(description) => description,
                None => {
                    if !action_api_skip && !api_skip {
                        cx.error_spanned_by(arg, "No description provided");
                    }
                    String::new()
                }
            },
            name: ActionArgName::from_attrs(Name::from(&unraw(&ident)), name, Some(aliases)),
            default: default.get().unwrap_or(Default::None),
        }
    }

    pub fn api_skip(&self) -> bool {
        self.api_skip
    }

    pub fn deprecated(&self) -> bool {
        self.deprecated
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn name(&self) -> &ActionArgName {
        &self.name
    }

    pub fn default(&self) -> &Default {
        &self.default
    }

    fn arg_type() -> ActionArgType {
        ActionArgType::Query
    }
}

pub struct Param {
    api_skip: bool,
    deprecated: bool,
    description: String,
    name: Name,
}

impl Param {
    pub fn from_ast(
        cx: &Ctxt,
        arg: &mut syn::PatType,
        method: HttpMethod,
        description: Option<String>,
        action_api_skip: bool,
    ) -> Self {
        let attrs = &mut arg.attrs;

        let mut api_skip = Attr::none(cx, API_SKIP);
        let mut description = if let Some(description) = description {
            Attr::some(cx, DESCRIPTION, description)
        } else {
            Attr::none(cx, DESCRIPTION)
        };
        let mut deprecated = Attr::none(cx, DEPRECATED);
        let mut name = Attr::none(cx, RENAME);

        for i in (0..attrs.len()).rev() {
            let attr = &attrs[i];

            let arg_type = Self::arg_type();

            if let Some(attr_arg_type) = ActionArgType::from_attr(attr, &method) {
                if arg_type != attr_arg_type {
                    cx.error_spanned_by(
                        attr,
                        format_args!(
                            "Mismatched action argument type. Expected `{}` but found `{}`",
                            arg_type.as_str(),
                            attr_arg_type.as_str()
                        ),
                    );
                    continue;
                }
            } else {
                continue;
            }

            if let syn::Meta::List(meta) = &attr.meta {
                if meta.tokens.is_empty() {
                    attrs.remove(i);
                    continue;
                }
            } else if let syn::Meta::Path(_) = &attr.meta {
                attrs.remove(i);
                continue;
            }

            if let Err(err) = attr.parse_nested_meta(|meta| {
                if meta.path == API_SKIP {
                    if meta.input.peek(Token![=]) {
                        // #[param(api_skip = "...")]
                        if let Some(value) = parse_lit_into_bool(cx, API_SKIP, &meta)? {
                            api_skip.set(&meta.path, value)
                        }
                    } else {
                        // #[param(api_skip)]
                        api_skip.set(&meta.path, true)
                    }
                } else if meta.path == DESCRIPTION {
                    // #[param(description = "...")]
                    if let Some(s) = get_lit_str(cx, DESCRIPTION, &meta)? {
                        description.set(&meta.path, s.value());
                    }
                } else if meta.path == DEPRECATED {
                    if meta.input.peek(Token![=]) {
                        // #[param(deprecated = "...")]
                        if let Some(value) = parse_lit_into_bool(cx, DEPRECATED, &meta)? {
                            deprecated.set(&meta.path, value)
                        }
                    } else {
                        // #[param(deprecated)]
                        deprecated.set(&meta.path, true)
                    }
                } else if meta.path == RENAME {
                    // #[param(rename = "foo")]
                    if let Some(s) = get_lit_str(cx, ALIAS, &meta)? {
                        name.set_if_none(Name::from(&s));
                    }
                } else {
                    let path = meta.path.to_token_stream().to_string().replace(' ', "");
                    return Err(
                        meta.error(format_args!("unknown action argument attribute `{path}`"))
                    );
                }
                Ok(())
            }) {
                cx.syn_error(err);
            }

            attrs.remove(i);
        }

        for attr in attrs {
            if matches!(&attr.meta, syn::Meta::Path(path) if path == DEPRECATED) {
                deprecated.set_if_none(true);
            }
        }

        let api_skip = api_skip.get().unwrap_or(false);

        let ident = match arg.pat.as_ref() {
            syn::Pat::Ident(pat_ident) => Some(pat_ident.ident.clone()),
            _ => None,
        }
        .unwrap_or(format_ident!("unreachable"));

        Self {
            api_skip,
            deprecated: deprecated.get().unwrap_or(false),
            description: match description.get() {
                Some(description) => description,
                None => {
                    if !action_api_skip && !api_skip {
                        cx.error_spanned_by(arg, "No description provided");
                    }
                    String::new()
                }
            },
            name: name.get().unwrap_or(Name::from(&unraw(&ident))),
        }
    }

    pub fn api_skip(&self) -> bool {
        self.api_skip
    }

    pub fn deprecated(&self) -> bool {
        self.deprecated
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    fn arg_type() -> ActionArgType {
        ActionArgType::Param
    }
}

pub struct Body {
    api_skip: bool,
    deprecated: bool,
    description: String,
    decoder: Option<syn::Type>,
}

impl Body {
    pub fn from_ast(
        cx: &Ctxt,
        arg: &mut syn::PatType,
        method: HttpMethod,
        description: Option<String>,
        action_api_skip: bool,
    ) -> Self {
        let attrs = &mut arg.attrs;

        let mut api_skip = Attr::none(cx, API_SKIP);
        let mut description = if let Some(description) = description {
            Attr::some(cx, DESCRIPTION, description)
        } else {
            Attr::none(cx, DESCRIPTION)
        };
        let mut deprecated = Attr::none(cx, DEPRECATED);
        let mut decoder = Attr::none(cx, DECODER);

        for i in (0..attrs.len()).rev() {
            let attr = &attrs[i];

            let arg_type = Self::arg_type();

            if let Some(attr_arg_type) = ActionArgType::from_attr(attr, &method) {
                if arg_type != attr_arg_type {
                    cx.error_spanned_by(
                        attr,
                        format_args!(
                            "Mismatched action argument type. Expected `{}` but found `{}`",
                            arg_type.as_str(),
                            attr_arg_type.as_str()
                        ),
                    );
                    continue;
                }
            } else {
                continue;
            }

            if let syn::Meta::List(meta) = &attr.meta {
                if meta.tokens.is_empty() {
                    attrs.remove(i);
                    continue;
                }
            } else if let syn::Meta::Path(_) = &attr.meta {
                attrs.remove(i);
                continue;
            }

            if let Err(err) = attr.parse_nested_meta(|meta| {
                if meta.path == API_SKIP {
                    if meta.input.peek(Token![=]) {
                        // #[param(api_skip = "...")]
                        if let Some(value) = parse_lit_into_bool(cx, API_SKIP, &meta)? {
                            api_skip.set(&meta.path, value)
                        }
                    } else {
                        // #[param(api_skip)]
                        api_skip.set(&meta.path, true)
                    }
                } else if meta.path == DESCRIPTION {
                    // #[param(description = "...")]
                    if let Some(s) = get_lit_str(cx, DESCRIPTION, &meta)? {
                        description.set(&meta.path, s.value());
                    }
                } else if meta.path == DEPRECATED {
                    if meta.input.peek(Token![=]) {
                        // #[param(deprecated = "...")]
                        if let Some(value) = parse_lit_into_bool(cx, DEPRECATED, &meta)? {
                            deprecated.set(&meta.path, value)
                        }
                    } else {
                        // #[param(deprecated)]
                        deprecated.set(&meta.path, true)
                    }
                } else if meta.path == DECODER {
                    // #[param(decoder = "Type")]
                    if let Some(ty) = parse_lit_into_ty(cx, DECODER, &meta)? {
                        decoder.set_opt(&meta.path, Some(ty));
                    }
                } else {
                    let path = meta.path.to_token_stream().to_string().replace(' ', "");
                    return Err(
                        meta.error(format_args!("unknown action argument attribute `{path}`"))
                    );
                }
                Ok(())
            }) {
                cx.syn_error(err);
            }

            attrs.remove(i);
        }

        for attr in attrs {
            if matches!(&attr.meta, syn::Meta::Path(path) if path == DEPRECATED) {
                deprecated.set_if_none(true);
            }
        }

        let api_skip = api_skip.get().unwrap_or(false);

        Self {
            api_skip,
            deprecated: deprecated.get().unwrap_or(false),
            description: match description.get() {
                Some(description) => description,
                None => {
                    if !action_api_skip && !api_skip {
                        cx.error_spanned_by(arg, "No description provided");
                    }
                    String::new()
                }
            },
            decoder: decoder.get(),
        }
    }

    pub fn api_skip(&self) -> bool {
        self.api_skip
    }

    pub fn deprecated(&self) -> bool {
        self.deprecated
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn decoder(&self) -> Option<&syn::Type> {
        self.decoder.as_ref()
    }

    fn arg_type() -> ActionArgType {
        ActionArgType::Body
    }
}
