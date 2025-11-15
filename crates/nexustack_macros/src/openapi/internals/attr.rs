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
    internals::{Ctxt, attr::*, symbol::*},
    openapi::internals::name::{MultiName, Name},
};
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use std::{borrow::Cow, collections::BTreeSet};
use syn::{
    Ident, Token, meta::ParseNestedMeta, parse::Parser, parse_quote, spanned::Spanned, token,
};

// This module handles parsing of attributes. The entrypoints
// are `attr::Container::from_ast`, `attr::Variant::from_ast`, and
// `attr::Field::from_ast`. Each returns an instance of the corresponding
// struct. Note that none of them return a Result. Unrecognized, malformed, or
// duplicated attributes result in a span_err but otherwise are ignored. The
// user will see errors simultaneously for all bad attributes in the crate
// rather than just the first.

pub use crate::internals::case::RenameRule;

use super::Derive;

fn unraw(ident: &Ident) -> Ident {
    Ident::new(ident.to_string().trim_start_matches("r#"), ident.span())
}

#[derive(Debug, Copy, Clone)]
pub struct RenameAllRules {
    pub serialize: RenameRule,
    pub deserialize: RenameRule,
}

impl RenameAllRules {
    /// Returns a new `RenameAllRules` with the individual rules of `self` and
    /// `other_rules` joined by `RenameRules::or`.
    pub fn or(self, other_rules: Self) -> Self {
        Self {
            serialize: self.serialize.or(other_rules.serialize),
            deserialize: self.deserialize.or(other_rules.deserialize),
        }
    }
}

/// Represents struct or enum attribute information.
#[derive(Debug)]
pub struct Container {
    name: MultiName,
    transparent: bool,
    deny_unknown_fields: bool,
    default: Default,
    rename_all_rules: RenameAllRules,
    rename_all_fields_rules: RenameAllRules,
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
    tag: TagType,
    type_from: Option<syn::Type>,
    type_try_from: Option<syn::Type>,
    type_into: Option<syn::Type>,
    identifier: Identifier,
    serde_path: Option<syn::Path>,
    crate_path: Option<syn::Path>,
    /// Error message generated when type can't be deserialized
    expecting: Option<String>,
    non_exhaustive: bool,
    deprecated: bool,
    description: String,
    // TODO: rename
    derive: Derive,
}

/// Styles of representing an enum.
#[derive(Debug)]
pub enum TagType {
    /// The default.
    ///
    /// ```json
    /// {"variant1": {"key1": "value1", "key2": "value2"}}
    /// ```
    External,

    /// `#[api_variant(tag = "type")]`
    ///
    /// ```json
    /// {"type": "variant1", "key1": "value1", "key2": "value2"}
    /// ```
    Internal { tag: String },

    /// `#[api_variant(tag = "t", content = "c")]`
    ///
    /// ```json
    /// {"t": "variant1", "c": {"key1": "value1", "key2": "value2"}}
    /// ```
    Adjacent { tag: String, content: String },

    /// `#[api_variant(untagged)]`
    ///
    /// ```json
    /// {"key1": "value1", "key2": "value2"}
    /// ```
    None,
}

/// Whether this enum represents the fields of a struct or the variants of an
/// enum.
#[derive(Debug, Copy, Clone)]
pub enum Identifier {
    /// It does not.
    No,

    /// This enum represents the fields of a struct. All of the variants must be
    /// unit variants, except possibly one which is annotated with
    /// `#[api_variant(other)]` and is a newtype variant.
    Field,

    /// This enum represents the variants of an enum. All of the variants must
    /// be unit variants.
    Variant,
}

impl Container {
    /// Extract out the `#[api_property(...)]` attributes from an item.
    pub fn from_ast(cx: &Ctxt, meta: TokenStream, item: &syn::DeriveInput) -> Self {
        let mut ser_name = Attr::none(cx, RENAME);
        let mut de_name = Attr::none(cx, RENAME);
        let mut transparent = BoolAttr::none(cx, TRANSPARENT);
        let mut deny_unknown_fields = BoolAttr::none(cx, DENY_UNKNOWN_FIELDS);
        let mut default = Attr::none(cx, DEFAULT);
        let mut rename_all_ser_rule = Attr::none(cx, RENAME_ALL);
        let mut rename_all_de_rule = Attr::none(cx, RENAME_ALL);
        let mut rename_all_fields_ser_rule = Attr::none(cx, RENAME_ALL_FIELDS);
        let mut rename_all_fields_de_rule = Attr::none(cx, RENAME_ALL_FIELDS);
        let mut ser_bound = Attr::none(cx, BOUND);
        let mut de_bound = Attr::none(cx, BOUND);
        let mut untagged = BoolAttr::none(cx, UNTAGGED);
        let mut internal_tag = Attr::none(cx, TAG);
        let mut content = Attr::none(cx, CONTENT);
        let mut type_from = Attr::none(cx, FROM);
        let mut type_try_from = Attr::none(cx, TRY_FROM);
        let mut type_into = Attr::none(cx, INTO);
        let mut field_identifier = BoolAttr::none(cx, FIELD_IDENTIFIER);
        let mut variant_identifier = BoolAttr::none(cx, VARIANT_IDENTIFIER);
        let mut serde_path = Attr::none(cx, SERDE);
        let mut crate_path = Attr::none(cx, CRATE);
        let mut expecting = Attr::none(cx, EXPECTING);
        let mut read = Attr::none(cx, READ);
        let mut write = Attr::none(cx, WRITE);
        let mut description = Attr::none(cx, DESCRIPTION);
        let mut deprecated = Attr::none(cx, DESCRIPTION);
        let mut non_exhaustive = Attr::none(cx, DESCRIPTION);

        if !meta.is_empty() {
            let parser = syn::meta::parser(|meta| {
                if meta.path == RENAME {
                    // #[api_schema(rename = "foo")]
                    // #[api_schema(rename(serialize = "foo", deserialize = "bar"))]
                    let (ser, de) = get_renames(cx, RENAME, &meta)?;
                    ser_name.set_opt(&meta.path, ser.as_ref().map(Name::from));
                    de_name.set_opt(&meta.path, de.as_ref().map(Name::from));
                } else if meta.path == RENAME_ALL {
                    // #[api_schema(rename_all = "foo")]
                    // #[api_schema(rename_all(serialize = "foo", deserialize = "bar"))]
                    let one_name = meta.input.peek(Token![=]);
                    let (ser, de) = get_renames(cx, RENAME_ALL, &meta)?;
                    if let Some(ser) = ser {
                        match RenameRule::from_str(&ser.value()) {
                            Ok(rename_rule) => rename_all_ser_rule.set(&meta.path, rename_rule),
                            Err(err) => cx.error_spanned_by(ser, err),
                        }
                    }
                    if let Some(de) = de {
                        match RenameRule::from_str(&de.value()) {
                            Ok(rename_rule) => rename_all_de_rule.set(&meta.path, rename_rule),
                            Err(err) => {
                                if !one_name {
                                    cx.error_spanned_by(de, err);
                                }
                            }
                        }
                    }
                } else if meta.path == RENAME_ALL_FIELDS {
                    // #[api_schema(rename_all_fields = "foo")]
                    // #[api_schema(rename_all_fields(serialize = "foo", deserialize = "bar"))]
                    let one_name = meta.input.peek(Token![=]);
                    let (ser, de) = get_renames(cx, RENAME_ALL_FIELDS, &meta)?;

                    match item.data {
                        syn::Data::Enum(_) => {
                            if let Some(ser) = ser {
                                match RenameRule::from_str(&ser.value()) {
                                    Ok(rename_rule) => {
                                        rename_all_fields_ser_rule.set(&meta.path, rename_rule);
                                    }
                                    Err(err) => cx.error_spanned_by(ser, err),
                                }
                            }
                            if let Some(de) = de {
                                match RenameRule::from_str(&de.value()) {
                                    Ok(rename_rule) => {
                                        rename_all_fields_de_rule.set(&meta.path, rename_rule);
                                    }
                                    Err(err) => {
                                        if !one_name {
                                            cx.error_spanned_by(de, err);
                                        }
                                    }
                                }
                            }
                        }
                        syn::Data::Struct(_) => {
                            let msg = "#[api_schema(rename_all_fields)] can only be used on enums";
                            cx.syn_error(meta.error(msg));
                        }
                        syn::Data::Union(_) => {
                            let msg = "#[api_schema(rename_all_fields)] can only be used on enums";
                            cx.syn_error(meta.error(msg));
                        }
                    }
                } else if meta.path == TRANSPARENT {
                    // #[api_schema(transparent)]
                    transparent.set_true(meta.path);
                } else if meta.path == DENY_UNKNOWN_FIELDS {
                    // #[api_schema(deny_unknown_fields)]
                    deny_unknown_fields.set_true(meta.path);
                } else if meta.path == DEFAULT {
                    if meta.input.peek(Token![=]) {
                        // #[api_schema(default = "...")]
                        if let Some(path) = parse_lit_into_expr_path(cx, DEFAULT, &meta)? {
                            match &item.data {
                                syn::Data::Struct(syn::DataStruct { fields, .. }) => match fields {
                                    syn::Fields::Named(_) | syn::Fields::Unnamed(_) => {
                                        default.set(&meta.path, Default::Path(path));
                                    }
                                    syn::Fields::Unit => {
                                        let msg = "#[api_schema(default = \"...\")] can only be used on structs that have fields";
                                        cx.syn_error(meta.error(msg));
                                    }
                                },
                                syn::Data::Enum(_) => {
                                    let msg = "#[v(default = \"...\")] can only be used on structs";
                                    cx.syn_error(meta.error(msg));
                                }
                                syn::Data::Union(_) => {
                                    let msg = "#[api_schema(default = \"...\")] can only be used on structs";
                                    cx.syn_error(meta.error(msg));
                                }
                            }
                        }
                    } else {
                        // #[api_schema(default)]
                        match &item.data {
                            syn::Data::Struct(syn::DataStruct { fields, .. }) => match fields {
                                syn::Fields::Named(_) | syn::Fields::Unnamed(_) => {
                                    default.set(meta.path, Default::Default);
                                }
                                syn::Fields::Unit => {
                                    let msg = "#[api_schema(default)] can only be used on structs that have fields";
                                    cx.error_spanned_by(fields, msg);
                                }
                            },
                            syn::Data::Enum(_) => {
                                let msg = "#[api_schema(default)] can only be used on structs";
                                cx.syn_error(meta.error(msg));
                            }
                            syn::Data::Union(_) => {
                                let msg = "#[api_schema(default)] can only be used on structs";
                                cx.syn_error(meta.error(msg));
                            }
                        }
                    }
                } else if meta.path == BOUND {
                    // #[api_schema(bound = "T: SomeBound")]
                    // #[api_schema(bound(serialize = "...", deserialize = "..."))]
                    let (ser, de) = get_where_predicates(cx, &meta)?;
                    ser_bound.set_opt(&meta.path, ser);
                    de_bound.set_opt(&meta.path, de);
                } else if meta.path == UNTAGGED {
                    // #[api_schema(untagged)]
                    match item.data {
                        syn::Data::Enum(_) => {
                            untagged.set_true(&meta.path);
                        }
                        syn::Data::Struct(_) => {
                            let msg = "#[api_schema(untagged)] can only be used on enums";
                            cx.syn_error(meta.error(msg));
                        }
                        syn::Data::Union(_) => {
                            let msg = "#[api_schema(untagged)] can only be used on enums";
                            cx.syn_error(meta.error(msg));
                        }
                    }
                } else if meta.path == TAG {
                    // #[api_schema(tag = "type")]
                    if let Some(s) = get_lit_str(cx, TAG, &meta)? {
                        match &item.data {
                            syn::Data::Enum(_) => {
                                internal_tag.set(&meta.path, s.value());
                            }
                            syn::Data::Struct(syn::DataStruct { fields, .. }) => match fields {
                                syn::Fields::Named(_) => {
                                    internal_tag.set(&meta.path, s.value());
                                }
                                syn::Fields::Unnamed(_) | syn::Fields::Unit => {
                                    let msg = "#[api_schema(tag = \"...\")] can only be used on enums and structs with named fields";
                                    cx.syn_error(meta.error(msg));
                                }
                            },
                            syn::Data::Union(_) => {
                                let msg = "#[api_schema(tag = \"...\")] can only be used on enums and structs with named fields";
                                cx.syn_error(meta.error(msg));
                            }
                        }
                    }
                } else if meta.path == CONTENT {
                    // #[api_schema(content = "c")]
                    if let Some(s) = get_lit_str(cx, CONTENT, &meta)? {
                        match &item.data {
                            syn::Data::Enum(_) => {
                                content.set(&meta.path, s.value());
                            }
                            syn::Data::Struct(_) => {
                                let msg =
                                    "#[api_schema(content = \"...\")] can only be used on enums";
                                cx.syn_error(meta.error(msg));
                            }
                            syn::Data::Union(_) => {
                                let msg =
                                    "#[api_schema(content = \"...\")] can only be used on enums";
                                cx.syn_error(meta.error(msg));
                            }
                        }
                    }
                } else if meta.path == FROM {
                    // #[api_schema(from = "Type")]
                    if let Some(from_ty) = parse_lit_into_ty(cx, FROM, &meta)? {
                        type_from.set_opt(&meta.path, Some(from_ty));
                    }
                } else if meta.path == TRY_FROM {
                    // #[api_schema(try_from = "Type")]
                    if let Some(try_from_ty) = parse_lit_into_ty(cx, TRY_FROM, &meta)? {
                        type_try_from.set_opt(&meta.path, Some(try_from_ty));
                    }
                } else if meta.path == INTO {
                    // #[api_schema(into = "Type")]
                    if let Some(into_ty) = parse_lit_into_ty(cx, INTO, &meta)? {
                        type_into.set_opt(&meta.path, Some(into_ty));
                    }
                } else if meta.path == REMOTE {
                    // #[api_schema(remote = "...")]
                    cx.syn_error(syn::Error::new(
                        meta.path.span(),
                        "Custom (de)serializers for foreign types are disallowed. Use a dedicated type for custom (de)serialization.",
                    ));
                } else if meta.path == FIELD_IDENTIFIER {
                    // #[api_schema(field_identifier)]
                    field_identifier.set_true(&meta.path);
                } else if meta.path == VARIANT_IDENTIFIER {
                    // #[api_schema(variant_identifier)]
                    variant_identifier.set_true(&meta.path);
                } else if meta.path == CRATE {
                    // #[api_schema(crate = "foo")]
                    if let Some(path) = parse_lit_into_path(cx, CRATE, &meta)? {
                        crate_path.set(&meta.path, path);
                    }
                } else if meta.path == SERDE {
                    // #[api_schema(serde = "foo")]
                    if let Some(path) = parse_lit_into_path(cx, SERDE, &meta)? {
                        serde_path.set(&meta.path, path);
                    }
                } else if meta.path == EXPECTING {
                    // #[api_schema(expecting = "a message")]
                    if let Some(s) = get_lit_str(cx, EXPECTING, &meta)? {
                        expecting.set(&meta.path, s.value());
                    }
                } else if meta.path == READ {
                    // #[api_schema(read)]
                    read.set(&meta.path, true);
                } else if meta.path == WRITE {
                    // #[api_schema(write)]
                    write.set(&meta.path, true);
                } else if meta.path == DESCRIPTION {
                    // #[api_schema(description = "...")]
                    if let Some(s) = get_lit_str(cx, DESCRIPTION, &meta)? {
                        description.set(&meta.path, s.value());
                    }
                } else if meta.path == DEPRECATED {
                    if meta.input.peek(Token![=]) {
                        // #[api_schema(deprecated = "...")]
                        if let Some(value) = parse_lit_into_bool(cx, DEFAULT, &meta)? {
                            deprecated.set(&meta.path, value)
                        }
                    } else {
                        // #[api_schema(deprecated)]
                        deprecated.set(&meta.path, true)
                    }
                } else if meta.path == NON_EXHAUSTIVE {
                    if meta.input.peek(Token![=]) {
                        // #[api_schema(non_exhaustive = "...")]
                        if let Some(value) = parse_lit_into_bool(cx, NON_EXHAUSTIVE, &meta)? {
                            non_exhaustive.set(&meta.path, value)
                        }
                    } else {
                        // #[api_schema(non_exhaustive)]
                        non_exhaustive.set(&meta.path, true)
                    }
                } else {
                    let path = meta.path.to_token_stream().to_string().replace(' ', "");
                    return Err(meta.error(format_args!("unknown container attribute `{path}`")));
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
            if attr.path() != SERDE {
                if matches!(&attr.meta, syn::Meta::Path(path) if path == NON_EXHAUSTIVE) {
                    non_exhaustive.set_if_none(true);
                }

                if matches!(&attr.meta, syn::Meta::Path(path) if path == DEPRECATED) {
                    deprecated.set_if_none(true);
                }

                if let syn::Meta::NameValue(meta) = &attr.meta
                    && meta.path == DOC
                    && let Ok(Some(s)) = get_lit_str2_expr(cx, DOC, DOC, &meta.value)
                {
                    description.set_if_none(s.value().trim().to_string());
                }
            } else {
                cx.syn_error(syn::Error::new(
                    attr.span(),
                    "Custom serde attributes are disallowed.",
                ));
            }
        }

        Container {
            name: MultiName::from_attrs(Name::from(&unraw(&item.ident)), ser_name, de_name, None),
            transparent: transparent.get(),
            deny_unknown_fields: deny_unknown_fields.get(),
            default: default.get().unwrap_or(Default::None),
            rename_all_rules: RenameAllRules {
                serialize: rename_all_ser_rule.get().unwrap_or(RenameRule::None),
                deserialize: rename_all_de_rule.get().unwrap_or(RenameRule::None),
            },
            rename_all_fields_rules: RenameAllRules {
                serialize: rename_all_fields_ser_rule.get().unwrap_or(RenameRule::None),
                deserialize: rename_all_fields_de_rule.get().unwrap_or(RenameRule::None),
            },
            ser_bound: ser_bound.get(),
            de_bound: de_bound.get(),
            tag: decide_tag(cx, item, untagged, internal_tag, content),
            type_from: type_from.get(),
            type_try_from: type_try_from.get(),
            type_into: type_into.get(),
            identifier: decide_identifier(cx, item, field_identifier, variant_identifier),
            serde_path: serde_path.get(),
            crate_path: crate_path.get(),
            expecting: expecting.get(),
            derive: match (read.get().unwrap_or(false), write.get().unwrap_or(false)) {
                (true, false) => Derive::Read,
                (false, true) => Derive::Write,
                _ => Derive::ReadWrite,
            },
            non_exhaustive: non_exhaustive.get().unwrap_or(false),
            deprecated: deprecated.get().unwrap_or(false),
            description: match description.get() {
                Some(description) => description,
                None => {
                    cx.error_spanned_by(item, "No description provided");
                    String::new()
                }
            },
        }
    }

    pub fn name(&self) -> &MultiName {
        &self.name
    }

    pub fn rename_all_rules(&self) -> RenameAllRules {
        self.rename_all_rules
    }

    pub fn rename_all_fields_rules(&self) -> RenameAllRules {
        self.rename_all_fields_rules
    }

    pub fn transparent(&self) -> bool {
        self.transparent
    }

    pub fn deny_unknown_fields(&self) -> bool {
        self.deny_unknown_fields
    }

    pub fn default(&self) -> &Default {
        &self.default
    }

    pub fn ser_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.ser_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn de_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.de_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn tag(&self) -> &TagType {
        &self.tag
    }

    pub fn type_from(&self) -> Option<&syn::Type> {
        self.type_from.as_ref()
    }

    pub fn type_try_from(&self) -> Option<&syn::Type> {
        self.type_try_from.as_ref()
    }

    pub fn type_into(&self) -> Option<&syn::Type> {
        self.type_into.as_ref()
    }

    pub fn identifier(&self) -> Identifier {
        self.identifier
    }

    pub fn custom_serde_path(&self) -> Option<&syn::Path> {
        self.serde_path.as_ref()
    }

    pub fn serde_path(&self) -> Cow<'_, syn::Path> {
        self.custom_serde_path()
            .map_or_else(|| Cow::Owned(parse_quote!(::serde)), Cow::Borrowed)
    }

    pub fn custom_crate_path(&self) -> Option<&syn::Path> {
        self.crate_path.as_ref()
    }

    /// Error message generated when type can't be deserialized.
    /// If `None`, default message will be used
    pub fn expecting(&self) -> Option<&str> {
        self.expecting.as_ref().map(String::as_ref)
    }

    pub fn non_exhaustive(&self) -> bool {
        self.non_exhaustive
    }

    pub fn deprecated(&self) -> bool {
        self.deprecated
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn derive(&self) -> Derive {
        self.derive
    }
}

fn decide_tag(
    cx: &Ctxt,
    item: &syn::DeriveInput,
    untagged: BoolAttr,
    internal_tag: Attr<String>,
    content: Attr<String>,
) -> TagType {
    match (
        untagged.0.get_with_tokens(),
        internal_tag.get_with_tokens(),
        content.get_with_tokens(),
    ) {
        (None, None, None) => TagType::External,
        (Some(_), None, None) => TagType::None,
        (None, Some((_, tag)), None) => {
            // Check that there are no tuple variants.
            if let syn::Data::Enum(data) = &item.data {
                for variant in &data.variants {
                    match &variant.fields {
                        syn::Fields::Named(_) | syn::Fields::Unit => {}
                        syn::Fields::Unnamed(fields) => {
                            if fields.unnamed.len() != 1 {
                                let msg = "#[api_schema(tag = \"...\")] cannot be used with tuple variants";
                                cx.error_spanned_by(variant, msg);
                                break;
                            }
                        }
                    }
                }
            }
            TagType::Internal { tag }
        }
        (Some((untagged_tokens, ())), Some((tag_tokens, _)), None) => {
            let msg = "enum cannot be both untagged and internally tagged";
            cx.error_spanned_by(untagged_tokens, msg);
            cx.error_spanned_by(tag_tokens, msg);
            TagType::External // doesn't matter, will error
        }
        (None, None, Some((content_tokens, _))) => {
            let msg = "#[api_schema(tag = \"...\", content = \"...\")] must be used together";
            cx.error_spanned_by(content_tokens, msg);
            TagType::External
        }
        (Some((untagged_tokens, ())), None, Some((content_tokens, _))) => {
            let msg = "untagged enum cannot have #[api_schema(content = \"...\")]";
            cx.error_spanned_by(untagged_tokens, msg);
            cx.error_spanned_by(content_tokens, msg);
            TagType::External
        }
        (None, Some((_, tag)), Some((_, content))) => TagType::Adjacent { tag, content },
        (Some((untagged_tokens, ())), Some((tag_tokens, _)), Some((content_tokens, _))) => {
            let msg = "untagged enum cannot have #[api_schema(tag = \"...\", content = \"...\")]";
            cx.error_spanned_by(untagged_tokens, msg);
            cx.error_spanned_by(tag_tokens, msg);
            cx.error_spanned_by(content_tokens, msg);
            TagType::External
        }
    }
}

fn decide_identifier(
    cx: &Ctxt,
    item: &syn::DeriveInput,
    field_identifier: BoolAttr,
    variant_identifier: BoolAttr,
) -> Identifier {
    match (
        &item.data,
        field_identifier.0.get_with_tokens(),
        variant_identifier.0.get_with_tokens(),
    ) {
        (_, None, None) => Identifier::No,
        (_, Some((field_identifier_tokens, ())), Some((variant_identifier_tokens, ()))) => {
            let msg = "#[api_property(field_identifier)] and #[api_variant(variant_identifier)] cannot both be set";
            cx.error_spanned_by(field_identifier_tokens, msg);
            cx.error_spanned_by(variant_identifier_tokens, msg);
            Identifier::No
        }
        (syn::Data::Enum(_), Some(_), None) => Identifier::Field,
        (syn::Data::Enum(_), None, Some(_)) => Identifier::Variant,
        (syn::Data::Struct(syn::DataStruct { struct_token, .. }), Some(_), None) => {
            let msg = "#[api_property(field_identifier)] can only be used on an enum";
            cx.error_spanned_by(struct_token, msg);
            Identifier::No
        }
        (syn::Data::Union(syn::DataUnion { union_token, .. }), Some(_), None) => {
            let msg = "#[api_property(field_identifier)] can only be used on an enum";
            cx.error_spanned_by(union_token, msg);
            Identifier::No
        }
        (syn::Data::Struct(syn::DataStruct { struct_token, .. }), None, Some(_)) => {
            let msg = "#[api_variant(variant_identifier)] can only be used on an enum";
            cx.error_spanned_by(struct_token, msg);
            Identifier::No
        }
        (syn::Data::Union(syn::DataUnion { union_token, .. }), None, Some(_)) => {
            let msg = "#[api_variant(variant_identifier)] can only be used on an enum";
            cx.error_spanned_by(union_token, msg);
            Identifier::No
        }
    }
}

/// Represents variant attribute information
pub struct Variant {
    name: MultiName,
    rename_all_rules: RenameAllRules,
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
    skip: bool,
    other: bool,
    untagged: bool,
    deprecated: bool,
    description: String,
}

impl Variant {
    pub fn from_ast(cx: &Ctxt, variant: &mut syn::Variant) -> Self {
        let mut ser_name = Attr::none(cx, RENAME);
        let mut de_name = Attr::none(cx, RENAME);
        let mut de_aliases = VecAttr::none(cx, RENAME);
        let mut skip = BoolAttr::none(cx, SKIP);
        let mut rename_all_ser_rule = Attr::none(cx, RENAME_ALL);
        let mut rename_all_de_rule = Attr::none(cx, RENAME_ALL);
        let mut ser_bound = Attr::none(cx, BOUND);
        let mut de_bound = Attr::none(cx, BOUND);
        let mut other = BoolAttr::none(cx, OTHER);
        let mut untagged = BoolAttr::none(cx, UNTAGGED);
        let mut description = Attr::none(cx, DESCRIPTION);
        let mut deprecated = Attr::none(cx, DESCRIPTION);

        for i in (0..variant.attrs.len()).rev() {
            let attr = &variant.attrs[i];

            if attr.path() == SERDE {
                cx.syn_error(syn::Error::new(
                    attr.path().span(),
                    "Custom serde attributes are disallowed.",
                ));
                continue;
            }

            if attr.path() != API_VARIANT {
                continue;
            }

            if let syn::Meta::List(meta) = &attr.meta {
                if meta.tokens.is_empty() {
                    variant.attrs.remove(i);
                    continue;
                }
            } else if let syn::Meta::Path(_) = &attr.meta {
                variant.attrs.remove(i);
                continue;
            }

            if let Err(err) = attr.parse_nested_meta(|meta| {
                if meta.path == RENAME {
                    // #[api_variant(rename = "foo")]
                    // #[api_variant(rename(serialize = "foo", deserialize = "bar"))]
                    let (ser, de) = get_multiple_renames(cx, &meta)?;
                    ser_name.set_opt(&meta.path, ser.as_ref().map(Name::from));
                    for de_value in de {
                        de_name.set_if_none(Name::from(&de_value));
                        de_aliases.insert(&meta.path, Name::from(&de_value));
                    }
                } else if meta.path == ALIAS {
                    // #[api_variant(alias = "foo")]
                    if let Some(s) = get_lit_str(cx, ALIAS, &meta)? {
                        de_aliases.insert(&meta.path, Name::from(&s));
                    }
                } else if meta.path == RENAME_ALL {
                    // #[api_variant(rename_all = "foo")]
                    // #[api_variant(rename_all(serialize = "foo", deserialize = "bar"))]
                    let one_name = meta.input.peek(Token![=]);
                    let (ser, de) = get_renames(cx, RENAME_ALL, &meta)?;
                    if let Some(ser) = ser {
                        match RenameRule::from_str(&ser.value()) {
                            Ok(rename_rule) => rename_all_ser_rule.set(&meta.path, rename_rule),
                            Err(err) => cx.error_spanned_by(ser, err),
                        }
                    }
                    if let Some(de) = de {
                        match RenameRule::from_str(&de.value()) {
                            Ok(rename_rule) => rename_all_de_rule.set(&meta.path, rename_rule),
                            Err(err) => {
                                if !one_name {
                                    cx.error_spanned_by(de, err);
                                }
                            }
                        }
                    }
                } else if meta.path == SKIP {
                    // #[api_variant(skip)]
                    skip.set_true(&meta.path);
                } else if meta.path == SKIP_DESERIALIZING {
                    // #[api_variant(skip_deserializing)]
                    cx.syn_error(syn::Error::new(
                        meta.path.span(),
                        "#[api_variant(skip_deserializing)] disallowed. Use #[api_variant(skip)] instead.",
                    ));
                } else if meta.path == SKIP_SERIALIZING {
                    // #[api_variant(skip_serializing)]
                    cx.syn_error(syn::Error::new(
                        meta.path.span(),
                        "#[api_variant(skip_serializing)] disallowed. Use #[api_variant(skip)] instead.",
                    ));
                } else if meta.path == OTHER {
                    // #[api_variant(other)]
                    other.set_true(&meta.path);
                } else if meta.path == BOUND {
                    // #[api_variant(bound = "T: SomeBound")]
                    // #[api_variant(bound(serialize = "...", deserialize = "..."))]
                    let (ser, de) = get_where_predicates(cx, &meta)?;
                    ser_bound.set_opt(&meta.path, ser);
                    de_bound.set_opt(&meta.path, de);
                } else if meta.path == WITH {
                    // #[api_variant(with = "...")]
                    cx.syn_error(syn::Error::new(
                        meta.path.span(),
                        "Custom (de)serializers are disallowed. Use a dedicated type for custom (de)serialization.",
                    ));
                } else if meta.path == SERIALIZE_WITH {
                    // #[api_variant(serialize_with = "...")]
                    cx.syn_error(syn::Error::new(
                        meta.path.span(),
                        "Custom serializers are disallowed. Use a dedicated type for custom de/serialization.",
                    ));
                } else if meta.path == DESERIALIZE_WITH {
                    // #[api_variant(deserialize_with = "...")]
                    cx.syn_error(syn::Error::new(
                        meta.path.span(),
                        "Custom deserializers are disallowed. Use a dedicated type for custom de/serialization.",
                    ));
                } else if meta.path == UNTAGGED {
                    untagged.set_true(&meta.path);
                } else if meta.path == DESCRIPTION {
                    // #[api_schema(description = "...")]
                    if let Some(s) = get_lit_str(cx, DESCRIPTION, &meta)? {
                        description.set(&meta.path, s.value());
                    }
                } else if meta.path == DEPRECATED {
                    if meta.input.peek(Token![=]) {
                        // #[api_schema(deprecated = "...")]
                        if let Some(value) = parse_lit_into_bool(cx, DEFAULT, &meta)? {
                            deprecated.set(&meta.path, value)
                        }
                    } else {
                        // #[api_schema(deprecated)]
                        deprecated.set(&meta.path, true)
                    }
                } else if meta.path != BORROW {
                    let path = meta.path.to_token_stream().to_string().replace(' ', "");
                    return Err(meta.error(format_args!("unknown variant attribute `{path}`")));
                }
                Ok(())
            }) {
                cx.syn_error(err);
            }

            match &mut variant.attrs[i].meta {
                syn::Meta::Path(_) => {}
                syn::Meta::List(meta_list) => {
                    meta_list.path = syn::Path::from(Ident::new(
                        SERDE.to_string().as_str(),
                        meta_list.path.span(),
                    ));
                }
                syn::Meta::NameValue(meta_name_value) => {
                    meta_name_value.path = syn::Path::from(Ident::new(
                        SERDE.to_string().as_str(),
                        meta_name_value.path.span(),
                    ));
                }
            }
        }

        for attr in &variant.attrs {
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

        Variant {
            name: MultiName::from_attrs(
                Name::from(&unraw(&variant.ident)),
                ser_name,
                de_name,
                Some(de_aliases),
            ),
            rename_all_rules: RenameAllRules {
                serialize: rename_all_ser_rule.get().unwrap_or(RenameRule::None),
                deserialize: rename_all_de_rule.get().unwrap_or(RenameRule::None),
            },
            ser_bound: ser_bound.get(),
            de_bound: de_bound.get(),
            skip: skip.get(),
            other: other.get(),
            untagged: untagged.get(),
            deprecated: deprecated.get().unwrap_or(false),
            description: match description.get() {
                Some(description) => description,
                None => {
                    cx.error_spanned_by(variant, "No description provided");
                    String::new()
                }
            },
        }
    }

    pub fn name(&self) -> &MultiName {
        &self.name
    }

    pub fn rename_by_rules(&mut self, rules: RenameAllRules) {
        if !self.name.serialize_renamed {
            self.name.serialize.value =
                rules.serialize.apply_to_variant(&self.name.serialize.value);
        }
        if !self.name.deserialize_renamed {
            self.name.deserialize.value = rules
                .deserialize
                .apply_to_variant(&self.name.deserialize.value);
        }
        self.name
            .deserialize_aliases
            .insert(self.name.deserialize.clone());
    }

    pub fn rename_all_rules(&self) -> RenameAllRules {
        self.rename_all_rules
    }

    pub fn ser_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.ser_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn de_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.de_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn skip(&self) -> bool {
        self.skip
    }

    pub fn other(&self) -> bool {
        self.other
    }

    pub fn untagged(&self) -> bool {
        self.untagged
    }

    pub fn deprecated(&self) -> bool {
        self.deprecated
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

/// Represents field attribute information
pub struct Field {
    name: MultiName,
    skip: bool,
    skip_serializing_if: Option<syn::ExprPath>,
    default: Default,
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
    flatten: bool,
    transparent: bool,
    deprecated: bool,
    description: String,
}

/// Represents the default to use for a field when deserializing.
#[derive(Debug, PartialEq, Eq)]
pub enum Default {
    /// Field must always be specified because it does not have a default.
    None,
    /// The default is given by `std::default::Default::default()`.
    #[allow(clippy::enum_variant_names)]
    Default,
    /// The default is given by this function.
    Path(syn::ExprPath),
}

impl Default {
    pub fn is_none(&self) -> bool {
        match self {
            Default::None => true,
            Default::Default | Default::Path(_) => false,
        }
    }

    pub fn or<'a>(&'a self, other: &'a Default) -> &'a Default {
        if self.is_none() {
            return other;
        }

        self
    }
}

impl Field {
    /// Extract out the `#[api_property(...)]` attributes from a struct field.
    pub fn from_ast(
        cx: &Ctxt,
        index: usize,
        field: &mut syn::Field,
        container_default: &Default,
    ) -> Self {
        let mut ser_name = Attr::none(cx, RENAME);
        let mut de_name = Attr::none(cx, RENAME);
        let mut de_aliases = VecAttr::none(cx, RENAME);
        let mut skip = BoolAttr::none(cx, SKIP);
        let mut skip_serializing_if = Attr::none(cx, SKIP_SERIALIZING_IF);
        let mut default = Attr::none(cx, DEFAULT);
        let mut ser_bound = Attr::none(cx, BOUND);
        let mut de_bound = Attr::none(cx, BOUND);
        let mut flatten = BoolAttr::none(cx, FLATTEN);
        let mut description = Attr::none(cx, DESCRIPTION);
        let mut deprecated = Attr::none(cx, DESCRIPTION);

        let ident = match &field.ident {
            Some(ident) => Name::from(&unraw(ident)),
            None => Name {
                value: index.to_string(),
                span: Span::call_site(),
            },
        };

        for i in (0..field.attrs.len()).rev() {
            let attr = &field.attrs[i];

            if attr.path() == SERDE {
                cx.syn_error(syn::Error::new(
                    attr.path().span(),
                    "Custom serde attributes are disallowed.",
                ));
                continue;
            }

            if attr.path() != API_PROPERTY {
                continue;
            }

            if let syn::Meta::List(meta) = &attr.meta {
                if meta.tokens.is_empty() {
                    field.attrs.remove(i);
                    continue;
                }
            } else if let syn::Meta::Path(_) = &attr.meta {
                field.attrs.remove(i);
                continue;
            }

            if let Err(err) = attr.parse_nested_meta(|meta| {
                if meta.path == RENAME {
                    // #[api_property(rename = "foo")]
                    // #[api_property(rename(serialize = "foo", deserialize = "bar"))]
                    let (ser, de) = get_multiple_renames(cx, &meta)?;
                    ser_name.set_opt(&meta.path, ser.as_ref().map(Name::from));
                    for de_value in de {
                        de_name.set_if_none(Name::from(&de_value));
                        de_aliases.insert(&meta.path, Name::from(&de_value));
                    }
                } else if meta.path == ALIAS {
                    // #[api_property(alias = "foo")]
                    if let Some(s) = get_lit_str(cx, ALIAS, &meta)? {
                        de_aliases.insert(&meta.path, Name::from(&s));
                    }
                } else if meta.path == DEFAULT {
                    if meta.input.peek(Token![=]) {
                        // #[api_property(default = "...")]
                        if let Some(path) = parse_lit_into_expr_path(cx, DEFAULT, &meta)? {
                            default.set(&meta.path, Default::Path(path));
                        }
                    } else {
                        // #[api_property(default)]
                        default.set(&meta.path, Default::Default);
                    }
                } else if meta.path == SKIP_SERIALIZING {
                    // #[api_property(skip_serializing)]
                    cx.syn_error(syn::Error::new(
                        meta.path.span(),
                        "#[api_property(skip_serializing)] disallowed. Use #[api_property(skip)] instead.",
                    ));
                } else if meta.path == SKIP_DESERIALIZING {
                    // #[api_property(skip_deserializing)]
                    cx.syn_error(syn::Error::new(
                        meta.path.span(),
                        "#[api_property(skip_deserializing)] disallowed. Use #[api_property(skip)] instead.",
                    ));
                } else if meta.path == SKIP {
                    // #[api_property(skip)]
                    skip.set_true(&meta.path);
                } else if meta.path == SKIP_SERIALIZING_IF {
                    // #[api_property(skip_serializing_if = "...")]
                    if let Some(path) = parse_lit_into_expr_path(cx, SKIP_SERIALIZING_IF, &meta)? {
                        skip_serializing_if.set(&meta.path, path);
                    }
                } else if meta.path == WITH {
                    // #[api_variant(with = "...")]
                    cx.syn_error(syn::Error::new(
                        meta.path.span(),
                        "Custom (de)serializers are disallowed. Use a dedicated type for custom (de)serialization.",
                    ));
                } else if meta.path == SERIALIZE_WITH {
                    // #[api_variant(serialize_with = "...")]
                    cx.syn_error(syn::Error::new(
                        meta.path.span(),
                        "Custom serializers are disallowed. Use a dedicated type for custom de/serialization.",
                    ));
                } else if meta.path == DESERIALIZE_WITH {
                    // #[api_variant(deserialize_with = "...")]
                    cx.syn_error(syn::Error::new(
                        meta.path.span(),
                        "Custom deserializers are disallowed. Use a dedicated type for custom de/serialization.",
                    ));
                } else if meta.path == BOUND {
                    // #[api_property(bound = "T: SomeBound")]
                    // #[api_property(bound(serialize = "...", deserialize = "..."))]
                    let (ser, de) = get_where_predicates(cx, &meta)?;
                    ser_bound.set_opt(&meta.path, ser);
                    de_bound.set_opt(&meta.path, de);
                } else if meta.path == GETTER {
                    // #[api_property(getter = "...")]
                    cx.syn_error(syn::Error::new(
                        meta.path.span(),
                        "Custom (de)serializers for foreign types are disallowed. Use a dedicated type for custom (de)serialization.",
                    ));
                } else if meta.path == FLATTEN {
                    // #[api_property(flatten)]
                    flatten.set_true(&meta.path);
                } else if meta.path == DESCRIPTION {
                    // #[api_schema(description = "...")]
                    if let Some(s) = get_lit_str(cx, DESCRIPTION, &meta)? {
                        description.set(&meta.path, s.value());
                    }
                } else if meta.path == DEPRECATED {
                    if meta.input.peek(Token![=]) {
                        // #[api_schema(deprecated = "...")]
                        if let Some(value) = parse_lit_into_bool(cx, DEFAULT, &meta)? {
                            deprecated.set(&meta.path, value)
                        }
                    } else {
                        // #[api_schema(deprecated)]
                        deprecated.set(&meta.path, true)
                    }
                } else if meta.path != BORROW{
                    let path = meta.path.to_token_stream().to_string().replace(' ', "");
                    return Err(meta.error(format_args!("unknown field attribute `{path}`")));
                }
                Ok(())
            }) {
                cx.syn_error(err);
            }

            match &mut field.attrs[i].meta {
                syn::Meta::Path(_) => {}
                syn::Meta::List(meta_list) => {
                    meta_list.path = syn::Path::from(Ident::new(
                        SERDE.to_string().as_str(),
                        meta_list.path.span(),
                    ));
                }
                syn::Meta::NameValue(meta_name_value) => {
                    meta_name_value.path = syn::Path::from(Ident::new(
                        SERDE.to_string().as_str(),
                        meta_name_value.path.span(),
                    ));
                }
            }
        }

        // Is skip_deserializing, initialize the field to Default::default() unless a
        // different default is specified by `#[api_property(default = "...")]` or `#[api_schema(default = "...")]`` on
        // ourselves or our container (e.g. the struct we are in) respectively.
        if let Default::None = *container_default
            && skip.0.value.is_some()
        {
            default.set_if_none(Default::Default);
        }

        for attr in &field.attrs {
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

        Field {
            name: MultiName::from_attrs(ident, ser_name, de_name, Some(de_aliases)),
            skip: skip.get(),
            skip_serializing_if: skip_serializing_if.get(),
            default: default.get().unwrap_or(Default::None),
            ser_bound: ser_bound.get(),
            de_bound: de_bound.get(),
            flatten: flatten.get(),
            transparent: false,
            deprecated: deprecated.get().unwrap_or(false),
            description: match description.get() {
                Some(description) => description,
                None => {
                    cx.error_spanned_by(field, "No description provided");
                    String::new()
                }
            },
        }
    }

    pub fn name(&self) -> &MultiName {
        &self.name
    }

    pub fn aliases(&self) -> &BTreeSet<Name> {
        self.name.deserialize_aliases()
    }

    pub fn rename_by_rules(&mut self, rules: RenameAllRules) {
        if !self.name.serialize_renamed {
            self.name.serialize.value = rules.serialize.apply_to_field(&self.name.serialize.value);
        }
        if !self.name.deserialize_renamed {
            self.name.deserialize.value = rules
                .deserialize
                .apply_to_field(&self.name.deserialize.value);
        }
        self.name
            .deserialize_aliases
            .insert(self.name.deserialize.clone());
    }

    pub fn skip(&self) -> bool {
        self.skip
    }

    pub fn skip_serializing_if(&self) -> Option<&syn::ExprPath> {
        self.skip_serializing_if.as_ref()
    }

    pub fn default(&self) -> &Default {
        &self.default
    }

    pub fn ser_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.ser_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn de_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.de_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn flatten(&self) -> bool {
        self.flatten
    }

    pub fn transparent(&self) -> bool {
        self.transparent
    }

    pub fn mark_transparent(&mut self) {
        self.transparent = true;
    }

    pub fn deprecated(&self) -> bool {
        self.deprecated
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

type SerAndDe<T> = (Option<T>, Option<T>);

fn get_ser_and_de<'c, T, F, R>(
    cx: &'c Ctxt,
    attr_name: Symbol,
    meta: &ParseNestedMeta,
    f: F,
) -> syn::Result<(VecAttr<'c, T>, VecAttr<'c, T>)>
where
    T: Clone,
    F: Fn(&Ctxt, Symbol, Symbol, &ParseNestedMeta) -> syn::Result<R>,
    R: Into<Option<T>>,
{
    let mut ser_meta = VecAttr::none(cx, attr_name);
    let mut de_meta = VecAttr::none(cx, attr_name);

    let lookahead = meta.input.lookahead1();
    if lookahead.peek(Token![=]) {
        if let Some(both) = f(cx, attr_name, attr_name, meta)?.into() {
            ser_meta.insert(&meta.path, both.clone());
            de_meta.insert(&meta.path, both);
        }
    } else if lookahead.peek(token::Paren) {
        meta.parse_nested_meta(|meta| {
            if meta.path == SERIALIZE {
                if let Some(v) = f(cx, attr_name, SERIALIZE, &meta)?.into() {
                    ser_meta.insert(&meta.path, v);
                }
            } else if meta.path == DESERIALIZE {
                if let Some(v) = f(cx, attr_name, DESERIALIZE, &meta)?.into() {
                    de_meta.insert(&meta.path, v);
                }
            } else {
                return Err(meta.error(format_args!(
                    "malformed {attr_name} attribute, expected `{attr_name}(serialize = ..., deserialize = ...)`",
                )));
            }
            Ok(())
        })?;
    } else {
        return Err(lookahead.error());
    }

    Ok((ser_meta, de_meta))
}

fn get_renames(
    cx: &Ctxt,
    attr_name: Symbol,
    meta: &ParseNestedMeta,
) -> syn::Result<SerAndDe<syn::LitStr>> {
    let (ser, de) = get_ser_and_de(cx, attr_name, meta, get_lit_str2)?;
    Ok((ser.at_most_one(), de.at_most_one()))
}

fn get_multiple_renames(
    cx: &Ctxt,
    meta: &ParseNestedMeta,
) -> syn::Result<(Option<syn::LitStr>, Vec<syn::LitStr>)> {
    let (ser, de) = get_ser_and_de(cx, RENAME, meta, get_lit_str2)?;
    Ok((ser.at_most_one(), de.get()))
}

fn get_where_predicates(
    cx: &Ctxt,
    meta: &ParseNestedMeta,
) -> syn::Result<SerAndDe<Vec<syn::WherePredicate>>> {
    let (ser, de) = get_ser_and_de(cx, BOUND, meta, parse_lit_into_where)?;
    Ok((ser.at_most_one(), de.at_most_one()))
}
