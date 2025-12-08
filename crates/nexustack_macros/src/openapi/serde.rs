/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    internals::default::Default,
    openapi::internals::{
        ast::{Container, Field, Variant},
        attr,
    },
};
use proc_macro2::TokenStream;
use quote::quote;

pub fn build_cont_attribute(cont: &Container) -> TokenStream {
    let opts = [
        build_cont_rename_opt(cont),
        build_cont_rename_all_opt(cont),
        build_cont_rename_all_fields_opt(cont),
        build_cont_deny_unknown_fields_opt(cont),
        build_cont_tag_opt(cont),
        build_cont_content_opt(cont),
        build_cont_untagged_opt(cont),
        build_cont_bound_opt(cont),
        build_cont_default_opt(cont),
        build_cont_transparent_opt(cont),
        build_cont_from_opt(cont),
        build_cont_try_from_opt(cont),
        build_cont_into_opt(cont),
        build_crate_opt(cont),
        build_expecting_opt(cont),
        build_cont_field_variant_identifier_opt(cont),
    ];

    opts.into_iter()
        .filter(|opt| !opt.is_empty())
        .reduce(|a, b| quote! { #a, #b })
        .map(|opts| quote! { #[serde(#opts)] })
        .unwrap_or(TokenStream::new())
}

pub fn build_example_struct_attribute(cont: &Container) -> TokenStream {
    let opts = [
        build_cont_rename_opt(cont),
        build_cont_rename_all_opt(cont),
        build_crate_opt(cont),
        quote! { bound = "" },
    ];

    opts.into_iter()
        .filter(|opt| !opt.is_empty())
        .reduce(|a, b| quote! { #a, #b })
        .map(|opts| quote! { #[serde(#opts)] })
        .unwrap_or(TokenStream::new())
}

pub fn build_example_enum_attribute(cont: &Container) -> TokenStream {
    let opts = [
        build_cont_rename_opt(cont),
        build_cont_rename_all_opt(cont),
        build_cont_rename_all_fields_opt(cont),
        build_cont_tag_opt(cont),
        build_cont_content_opt(cont),
        build_cont_untagged_opt(cont),
        build_crate_opt(cont),
        quote! { bound = "" },
    ];

    opts.into_iter()
        .filter(|opt| !opt.is_empty())
        .reduce(|a, b| quote! { #a, #b })
        .map(|opts| quote! { #[serde(#opts)] })
        .unwrap_or(TokenStream::new())
}

fn build_cont_rename_opt(cont: &Container) -> TokenStream {
    match (
        cont.attrs.name().serialize_renamed,
        cont.attrs.name().deserialize_renamed,
    ) {
        (true, true)
            if cont.attrs.name().serialize_name() == cont.attrs.name().deserialize_name() =>
        {
            let name = cont.attrs.name().serialize_name();
            quote! { rename = #name }
        }
        (true, true) => {
            let serialize_name = cont.attrs.name().serialize_name();
            let deserialize_name = cont.attrs.name().serialize_name();
            quote! { rename(serialize = #serialize_name, deserialize = #deserialize_name) }
        }
        (true, false) => {
            let serialize_name = cont.attrs.name().serialize_name();
            quote! { rename(serialize = #serialize_name) }
        }
        (false, true) => {
            let deserialize_name = cont.attrs.name().serialize_name();
            quote! { rename(deserialize = #deserialize_name) }
        }
        _ => TokenStream::new(),
    }
}

fn build_cont_rename_all_opt(cont: &Container) -> TokenStream {
    match (
        cont.attrs.rename_all_rules().serialize,
        cont.attrs.rename_all_rules().deserialize,
    ) {
        (attr::RenameRule::None, attr::RenameRule::None) => TokenStream::new(),
        (attr::RenameRule::None, deserialize_rename) => {
            let deserialize_rename = deserialize_rename.to_string();
            quote! { rename_all(deserialize = #deserialize_rename) }
        }
        (serialize_rename, attr::RenameRule::None) => {
            let serialize_rename = serialize_rename.to_string();
            quote! { rename_all(serialize = #serialize_rename) }
        }
        (serialize_rename, deserialize_rename) if serialize_rename == deserialize_rename => {
            let rename = serialize_rename.to_string();
            quote! { rename_all = #rename }
        }
        (serialize_rename, deserialize_rename) => {
            let serialize_rename = serialize_rename.to_string();
            let deserialize_rename = deserialize_rename.to_string();
            quote! { rename_all(serialize = #serialize_rename, deserialize = #deserialize_rename) }
        }
    }
}

fn build_cont_rename_all_fields_opt(cont: &Container) -> TokenStream {
    match (
        cont.attrs.rename_all_fields_rules().serialize,
        cont.attrs.rename_all_fields_rules().deserialize,
    ) {
        (attr::RenameRule::None, attr::RenameRule::None) => TokenStream::new(),
        (attr::RenameRule::None, deserialize_rename) => {
            let deserialize_rename = deserialize_rename.to_string();
            quote! { rename_all_fields(deserialize = #deserialize_rename) }
        }
        (serialize_rename, attr::RenameRule::None) => {
            let serialize_rename = serialize_rename.to_string();
            quote! { rename_all_fields(serialize = #serialize_rename) }
        }
        (serialize_rename, deserialize_rename) if serialize_rename == deserialize_rename => {
            let rename = serialize_rename.to_string();
            quote! { rename_all_fields = #rename }
        }
        (serialize_rename, deserialize_rename) => {
            let serialize_rename = serialize_rename.to_string();
            let deserialize_rename = deserialize_rename.to_string();
            quote! { rename_all_fields(serialize = #serialize_rename, deserialize = #deserialize_rename) }
        }
    }
}

fn build_cont_transparent_opt(cont: &Container) -> TokenStream {
    match cont.attrs.transparent() {
        true => quote! { transparent },
        false => TokenStream::new(),
    }
}

fn build_cont_deny_unknown_fields_opt(cont: &Container) -> TokenStream {
    match cont.attrs.deny_unknown_fields() {
        true => quote! { deny_unknown_fields },
        false => TokenStream::new(),
    }
}

fn build_cont_default_opt(cont: &Container) -> TokenStream {
    match cont.attrs.default() {
        Default::None => TokenStream::new(),
        Default::Default => quote! { default },
        Default::Path(expr_path) => {
            // TODO: Does the stringify work??
            let expr_path = quote! { #expr_path }.to_string();
            quote! { default = #expr_path }
        }
    }
}

fn build_cont_bound_opt(cont: &Container) -> TokenStream {
    match (cont.attrs.ser_bound(), cont.attrs.de_bound()) {
        (None, None) => TokenStream::new(),
        (None, Some(de_bound)) => {
            // TODO: Does the stringify work??
            let de_bound = quote! { #(#de_bound),* }.to_string();
            quote! { bound(deserialize = #de_bound) }
        }
        (Some(ser_bound), None) => {
            // TODO: Does the stringify work??
            let ser_bound = quote! { #(#ser_bound),* }.to_string();
            quote! { bound(serialize = #ser_bound) }
        }
        (Some(ser_bound), Some(de_bound)) if ser_bound == de_bound => {
            // TODO: Does the stringify work??
            let ser_bound = quote! { #(#ser_bound),* }.to_string();
            quote! { bound = #ser_bound }
        }
        (Some(ser_bound), Some(de_bound)) => {
            // TODO: Does the stringify work??
            let ser_bound = quote! { #(#ser_bound),* }.to_string();
            let de_bound = quote! { #(#de_bound),* }.to_string();
            quote! { bound(serialize = #ser_bound, deserialize = #de_bound)}
        }
    }
}

fn build_cont_untagged_opt(cont: &Container) -> TokenStream {
    match cont.attrs.tag() {
        attr::TagType::None => quote! { untagged },
        _ => TokenStream::new(),
    }
}

fn build_cont_tag_opt(cont: &Container) -> TokenStream {
    match cont.attrs.tag() {
        attr::TagType::Internal { tag } => quote! { tag = #tag },
        attr::TagType::Adjacent { tag, .. } => quote! { tag = #tag },
        _ => TokenStream::new(),
    }
}

fn build_cont_content_opt(cont: &Container) -> TokenStream {
    match cont.attrs.tag() {
        attr::TagType::Adjacent { content, .. } => quote! { content = #content },
        _ => TokenStream::new(),
    }
}

fn build_cont_from_opt(cont: &Container) -> TokenStream {
    match cont.attrs.type_from() {
        // TODO: Does the stringify work??
        Some(type_from) => {
            let type_from = quote! { #type_from }.to_string();
            quote! { from = #type_from }
        }
        None => TokenStream::new(),
    }
}

fn build_cont_try_from_opt(cont: &Container) -> TokenStream {
    match cont.attrs.type_try_from() {
        // TODO: Does the stringify work??
        Some(type_try_from) => {
            let type_try_from = quote! { #type_try_from }.to_string();
            quote! { try_from = #type_try_from }
        }
        None => TokenStream::new(),
    }
}

fn build_cont_into_opt(cont: &Container) -> TokenStream {
    match cont.attrs.type_into() {
        // TODO: Does the stringify work??
        Some(type_into) => {
            let type_into = quote! { #type_into }.to_string();
            quote! { into = #type_into }
        }
        None => TokenStream::new(),
    }
}

fn build_cont_field_variant_identifier_opt(cont: &Container) -> TokenStream {
    match cont.attrs.identifier() {
        attr::Identifier::No => TokenStream::new(),
        attr::Identifier::Field => quote! { field_identifier },
        attr::Identifier::Variant => quote! { variant_identifier },
    }
}

fn build_crate_opt(cont: &Container) -> TokenStream {
    match cont.attrs.custom_serde_path() {
        Some(custom_serde_path) => {
            // TODO: Does the stringify work??
            let custom_serde_path = quote! { #custom_serde_path }.to_string();
            quote! { crate = #custom_serde_path }
        }
        None => TokenStream::new(),
    }
}

fn build_expecting_opt(cont: &Container) -> TokenStream {
    match cont.attrs.expecting() {
        Some(expecting) => quote! { expecting = #expecting },
        None => TokenStream::new(),
    }
}

pub fn build_example_field_attribute(field: &Field<'_>) -> TokenStream {
    let opts = [
        build_field_rename_opt(field),
        build_field_flatten_opt(field),
        // TODO: How to handle skip_serialize_if, the example and field type do not necessarily match
        quote! { bound = "" },
    ];

    opts.into_iter()
        .filter(|opt| !opt.is_empty())
        .reduce(|a, b| quote! { #a, #b })
        .map(|opts| quote! { #[serde(#opts)] })
        .unwrap_or(TokenStream::new())
}

fn build_field_rename_opt(field: &Field<'_>) -> TokenStream {
    match (
        field.attrs.name().serialize_renamed,
        field.attrs.name().deserialize_renamed,
    ) {
        (true, true)
            if field.attrs.name().serialize_name() == field.attrs.name().deserialize_name() =>
        {
            let name = field.attrs.name().serialize_name();
            quote! { rename = #name }
        }
        (true, true) => {
            let serialize_name = field.attrs.name().serialize_name();
            let deserialize_name = field.attrs.name().serialize_name();
            quote! { rename(serialize = #serialize_name, deserialize = #deserialize_name) }
        }
        (true, false) => {
            let serialize_name = field.attrs.name().serialize_name();
            quote! { rename(serialize = #serialize_name) }
        }
        (false, true) => {
            let deserialize_name = field.attrs.name().serialize_name();
            quote! { rename(deserialize = #deserialize_name) }
        }
        _ => TokenStream::new(),
    }
}

fn build_field_flatten_opt(field: &Field<'_>) -> TokenStream {
    match field.attrs.flatten() {
        true => quote! { flatten },
        false => TokenStream::new(),
    }
}

pub fn build_example_variant_attribute(variant: &Variant<'_>) -> TokenStream {
    let opts = [
        build_variant_rename_opt(variant),
        build_variant_rename_all_opt(variant),
        build_variant_other_opt(variant),
        build_variant_untagged_opt(variant),
        // TODO: How to handle skip_serialize_if, the example and field type do not necessarily match
        quote! { bound = "" },
    ];

    opts.into_iter()
        .filter(|opt| !opt.is_empty())
        .reduce(|a, b| quote! { #a, #b })
        .map(|opts| quote! { #[serde(#opts)] })
        .unwrap_or(TokenStream::new())
}

fn build_variant_rename_opt(variant: &Variant<'_>) -> TokenStream {
    match (
        variant.attrs.name().serialize_renamed,
        variant.attrs.name().deserialize_renamed,
    ) {
        (true, true)
            if variant.attrs.name().serialize_name() == variant.attrs.name().deserialize_name() =>
        {
            let name = variant.attrs.name().serialize_name();
            quote! { rename = #name }
        }
        (true, true) => {
            let serialize_name = variant.attrs.name().serialize_name();
            let deserialize_name = variant.attrs.name().serialize_name();
            quote! { rename(serialize = #serialize_name, deserialize = #deserialize_name) }
        }
        (true, false) => {
            let serialize_name = variant.attrs.name().serialize_name();
            quote! { rename(serialize = #serialize_name) }
        }
        (false, true) => {
            let deserialize_name = variant.attrs.name().serialize_name();
            quote! { rename(deserialize = #deserialize_name) }
        }
        _ => TokenStream::new(),
    }
}

fn build_variant_rename_all_opt(variant: &Variant<'_>) -> TokenStream {
    match (
        variant.attrs.rename_all_rules().serialize,
        variant.attrs.rename_all_rules().deserialize,
    ) {
        (attr::RenameRule::None, attr::RenameRule::None) => TokenStream::new(),
        (attr::RenameRule::None, deserialize_rename) => {
            let deserialize_rename = deserialize_rename.to_string();
            quote! { rename_all(deserialize = #deserialize_rename) }
        }
        (serialize_rename, attr::RenameRule::None) => {
            let serialize_rename = serialize_rename.to_string();
            quote! { rename_all(serialize = #serialize_rename) }
        }
        (serialize_rename, deserialize_rename) if serialize_rename == deserialize_rename => {
            let rename = serialize_rename.to_string();
            quote! { rename_all = #rename }
        }
        (serialize_rename, deserialize_rename) => {
            let serialize_rename = serialize_rename.to_string();
            let deserialize_rename = deserialize_rename.to_string();
            quote! { rename_all(serialize = #serialize_rename, deserialize = #deserialize_rename) }
        }
    }
}

fn build_variant_other_opt(variant: &Variant<'_>) -> TokenStream {
    match variant.attrs.other() {
        true => quote! { other },
        false => TokenStream::new(),
    }
}

fn build_variant_untagged_opt(variant: &Variant<'_>) -> TokenStream {
    match variant.attrs.untagged() {
        true => quote! { untagged },
        false => TokenStream::new(),
    }
}
