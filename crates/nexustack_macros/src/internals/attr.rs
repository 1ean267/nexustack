/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/attr.rs
 */

use crate::internals::{Ctxt, symbol::Symbol};
use syn::meta::ParseNestedMeta;

#[cfg(any(feature = "openapi", feature = "cron"))]
use proc_macro2::TokenStream;
#[cfg(any(feature = "openapi", feature = "cron"))]
use quote::ToTokens;

#[cfg(feature = "openapi")]
use syn::{Token, punctuated::Punctuated};

#[cfg(any(feature = "openapi", feature = "cron"))]
pub(crate) struct Attr<'c, T> {
    cx: &'c Ctxt,
    name: Symbol,
    tokens: TokenStream,
    pub(crate) value: Option<T>,
}

#[cfg(any(feature = "openapi", feature = "cron"))]
impl<'c, T> Attr<'c, T> {
    pub(crate) fn none(cx: &'c Ctxt, name: Symbol) -> Self {
        Attr {
            cx,
            name,
            tokens: TokenStream::new(),
            value: None,
        }
    }

    pub(crate) fn set<A: ToTokens>(&mut self, obj: A, value: T) {
        let tokens = obj.into_token_stream();

        if self.value.is_some() {
            let msg = format!("duplicate attribute `{}`", self.name);
            self.cx.error_spanned_by(tokens, msg);
        } else {
            self.tokens = tokens;
            self.value = Some(value);
        }
    }

    #[cfg(feature = "openapi")]
    pub(crate) fn set_opt<A: ToTokens>(&mut self, obj: A, value: Option<T>) {
        if let Some(value) = value {
            self.set(obj, value);
        }
    }

    #[cfg(feature = "openapi")]
    pub(crate) fn set_if_none(&mut self, value: T) {
        if self.value.is_none() {
            self.value = Some(value);
        }
    }

    pub(crate) fn get(self) -> Option<T> {
        self.value
    }

    #[cfg(feature = "openapi")]
    pub(crate) fn get_with_tokens(self) -> Option<(TokenStream, T)> {
        match self.value {
            Some(v) => Some((self.tokens, v)),
            None => None,
        }
    }
}

#[cfg(feature = "openapi")]
pub(crate) struct BoolAttr<'c>(pub(crate) Attr<'c, ()>);

#[cfg(feature = "openapi")]
impl<'c> BoolAttr<'c> {
    pub(crate) fn none(cx: &'c Ctxt, name: Symbol) -> Self {
        BoolAttr(Attr::none(cx, name))
    }

    pub(crate) fn set_true<A: ToTokens>(&mut self, obj: A) {
        self.0.set(obj, ());
    }

    pub(crate) fn get(&self) -> bool {
        self.0.value.is_some()
    }
}

#[cfg(feature = "openapi")]
pub(crate) struct VecAttr<'c, T> {
    cx: &'c Ctxt,
    name: Symbol,
    first_dup_tokens: TokenStream,
    values: Vec<T>,
}

#[cfg(feature = "openapi")]
impl<'c, T> VecAttr<'c, T> {
    pub(crate) fn none(cx: &'c Ctxt, name: Symbol) -> Self {
        VecAttr {
            cx,
            name,
            first_dup_tokens: TokenStream::new(),
            values: Vec::new(),
        }
    }

    pub(crate) fn insert<A: ToTokens>(&mut self, obj: A, value: T) {
        if self.values.len() == 1 {
            self.first_dup_tokens = obj.into_token_stream();
        }
        self.values.push(value);
    }

    pub(crate) fn at_most_one(mut self) -> Option<T> {
        if self.values.len() > 1 {
            let dup_token = self.first_dup_tokens;
            let msg = format!("duplicate attribute `{}`", self.name);
            self.cx.error_spanned_by(dup_token, msg);
            None
        } else {
            self.values.pop()
        }
    }

    pub(crate) fn get(self) -> Vec<T> {
        self.values
    }
}

pub(crate) fn get_lit_str(
    cx: &Ctxt,
    attr_name: Symbol,
    meta: &ParseNestedMeta,
) -> syn::Result<Option<syn::LitStr>> {
    get_lit_str2(cx, attr_name, attr_name, meta)
}

pub(crate) fn get_lit_str2(
    cx: &Ctxt,
    attr_name: Symbol,
    meta_item_name: Symbol,
    meta: &ParseNestedMeta,
) -> syn::Result<Option<syn::LitStr>> {
    let expr: syn::Expr = meta.value()?.parse()?;
    get_lit_str2_expr(cx, attr_name, meta_item_name, &expr)
}

pub(crate) fn get_lit_str2_expr(
    cx: &Ctxt,
    attr_name: Symbol,
    meta_item_name: Symbol,
    expr: &syn::Expr,
) -> syn::Result<Option<syn::LitStr>> {
    let mut value = expr;
    while let syn::Expr::Group(e) = value {
        value = &e.expr;
    }
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(lit),
        ..
    }) = value
    {
        let suffix = lit.suffix();
        if !suffix.is_empty() {
            cx.error_spanned_by(
                lit,
                format!("unexpected suffix `{suffix}` on string literal"),
            );
        }
        Ok(Some(lit.clone()))
    } else {
        cx.error_spanned_by(
            expr,
            format!("expected {attr_name} attribute to be a string: `{meta_item_name} = \"...\"`"),
        );
        Ok(None)
    }
}

#[cfg(feature = "openapi")]
pub(crate) fn parse_lit_into_bool(
    cx: &Ctxt,
    attr_name: Symbol,
    meta: &ParseNestedMeta,
) -> syn::Result<Option<bool>> {
    let string = match get_lit_str(cx, attr_name, meta)? {
        Some(string) => string,
        None => return Ok(None),
    };

    if string.value().eq("true") {
        return Ok(Some(true));
    }

    if string.value().eq(&"false") {
        return Ok(Some(false));
    }

    cx.error_spanned_by(
        &string,
        format!("failed to parse path: {:?}", string.value()),
    );

    Ok(None)
}

pub(crate) fn parse_lit_into_path(
    cx: &Ctxt,
    attr_name: Symbol,
    meta: &ParseNestedMeta,
) -> syn::Result<Option<syn::Path>> {
    let string = match get_lit_str(cx, attr_name, meta)? {
        Some(string) => string,
        None => return Ok(None),
    };

    Ok(match string.parse() {
        Ok(path) => Some(path),
        Err(_) => {
            cx.error_spanned_by(
                &string,
                format!("failed to parse path: {:?}", string.value()),
            );
            None
        }
    })
}

#[cfg(any(feature = "openapi", feature = "cron"))]
pub(crate) fn parse_lit_into_expr_path(
    cx: &Ctxt,
    attr_name: Symbol,
    meta: &ParseNestedMeta,
) -> syn::Result<Option<syn::ExprPath>> {
    let string = match get_lit_str(cx, attr_name, meta)? {
        Some(string) => string,
        None => return Ok(None),
    };

    Ok(match string.parse() {
        Ok(expr) => Some(expr),
        Err(_) => {
            cx.error_spanned_by(
                &string,
                format!("failed to parse path: {:?}", string.value()),
            );
            None
        }
    })
}

#[cfg(feature = "openapi")]
pub(crate) fn parse_lit_into_where(
    cx: &Ctxt,
    attr_name: Symbol,
    meta_item_name: Symbol,
    meta: &ParseNestedMeta,
) -> syn::Result<Vec<syn::WherePredicate>> {
    let string = match get_lit_str2(cx, attr_name, meta_item_name, meta)? {
        Some(string) => string,
        None => return Ok(Vec::new()),
    };

    Ok(
        match string.parse_with(Punctuated::<syn::WherePredicate, Token![,]>::parse_terminated) {
            Ok(predicates) => Vec::from_iter(predicates),
            Err(err) => {
                cx.error_spanned_by(string, err);
                Vec::new()
            }
        },
    )
}

#[cfg(feature = "openapi")]
pub(crate) fn parse_lit_into_ty(
    cx: &Ctxt,
    attr_name: Symbol,
    meta: &ParseNestedMeta,
) -> syn::Result<Option<syn::Type>> {
    let string = match get_lit_str(cx, attr_name, meta)? {
        Some(string) => string,
        None => return Ok(None),
    };

    Ok(match string.parse() {
        Ok(ty) => Some(ty),
        Err(_) => {
            cx.error_spanned_by(
                &string,
                format!("failed to parse type: {} = {:?}", attr_name, string.value()),
            );
            None
        }
    })
}
