/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/check.rs
 */

use crate::internals::Ctxt;
use crate::openapi::internals::ast::{Container, Data, Field, Style};
use crate::openapi::internals::attr::{Default, Identifier, TagType};
use crate::openapi::internals::{Derive, ungroup};
use syn::Type;

// Cross-cutting checks that require looking at more than a single attrs object.
// Simpler checks should happen when parsing and building the attrs.
pub fn check(cx: &Ctxt, cont: &mut Container) {
    let derive = cont.attrs.derive();

    check_default_on_tuple(cx, cont);
    check_flatten(cx, cont);
    check_identifier(cx, cont);
    check_internal_tag_field_name_conflict(cx, cont);
    check_adjacent_tag_conflict(cx, cont);
    check_transparent(cx, cont, derive);
    check_from_and_try_from(cx, cont);

    if cont.attrs.derive() == Derive::ReadWrite {
        if let Some(type_from) = cont.attrs.type_from() {
            if let Some(type_into) = cont.attrs.type_into() {
                if type_into != type_from {
                    cx.error_spanned_by(
                    &cont.original,
                    "If #[api_schema(from)] and #[api_schema(into)] are used on a read-write type, the specified types must match".to_string(),
                );
                }
            } else {
                cx.error_spanned_by(
                    &cont.original,
                    "If #[api_schema(from)] is used on a read-write type, #[api_schema(into)] must be specified too".to_string(),
                );
            }
        }

        if let Some(type_try_from) = cont.attrs.type_try_from() {
            if let Some(type_into) = cont.attrs.type_into() {
                if type_into != type_try_from {
                    cx.error_spanned_by(
                    &cont.original,
                    "If #[api_schema(try_from)] and #[api_schema(into)] are used on a read-write type, the specified types must match".to_string(),
                );
                }
            } else {
                cx.error_spanned_by(
                    &cont.original,
                    "If #[api_schema(try_from)] is used on a read-write type, #[api_schema(into)] must be specified too".to_string(),
                );
            }
        }

        if let Some(type_into) = cont.attrs.type_into() {
            if let Some(type_from) = cont.attrs.type_from() {
                if type_into != type_from {
                    cx.error_spanned_by(
                    &cont.original,
                    "If #[api_schema(into)] and #[api_schema(from)] are used on a read-write type, the specified types must match".to_string(),
                );
                }
            } else if let Some(type_try_from) = cont.attrs.type_try_from() {
                if type_into != type_try_from {
                    cx.error_spanned_by(
                        &cont.original,
                        "If #[api_schema(into)] and #[api_schema(try_from)] are used on a read-write type, the specified types must match".to_string(),
                    );
                }
            } else {
                cx.error_spanned_by(
                    &cont.original,
                    "If #[api_schema(into)] is used on a read-write type, either #[api_schema(from)] or #[api_schema(try_from)] must be specified too".to_string(),
                );
            }
        }
    }

    if cont.attrs.derive() == Derive::ReadWrite
        && let Data::Struct(_, fields) = &cont.data
    {
        for field in fields {
            if field.attrs.skip_serializing_if().is_none() && !field.attrs.skip() {
                if !cont.attrs.default().is_none() {
                    cx.error_spanned_by(
                            field.ty,
                            "#[api_schema(default)] cannot be used on read-write structs with fields that have no #[api_property(skip_serializing_if)] attribute".to_string(),
                        );
                }

                if !field.attrs.default().is_none() {
                    cx.error_spanned_by(
                            field.original,
                            "#[api_property(default)] cannot be used on fields of read-write structs that have no #[api_property(skip_serializing_if)] attribute".to_string(),
                        );
                }
            }
        }
    }

    if let Data::Struct(_, fields) = &cont.data {
        for field in fields {
            if field.attrs.skip_serializing_if().is_some() {
                if cont.attrs.derive() == Derive::Read {
                    cx.error_spanned_by(
                        field.ty,
                        "#[api_property(skip_serializing_if)] cannot be used on fields of read structs".to_string(),
                    );
                } else if cont.attrs.default().or(field.attrs.default()).is_none() {
                    cx.error_spanned_by(
                        field.original,
                        "#[api_property(skip_serializing_if)]  cannot be used on fields of read structs that have neither #[api_schema(default)] nor #[api_property(default)] set".to_string(),
                    );
                }
            }
        }
    }

    // No skip in new-type struct
    // See https://github.com/serde-rs/serde/issues/2105
    if let Data::Struct(Style::Newtype, fields) = &cont.data {
        for field in fields.iter() {
            if field.attrs.skip() {
                cx.error_spanned_by(
                    &cont.original,
                    "Tuple struct marked with #[api_schema] must have at least a single non-skipped properties.".to_string(),
                );
            }
        }
    }

    // Not all properties skipped in tuple struct
    // See https://github.com/serde-rs/serde/issues/2105
    if let Data::Struct(Style::Tuple, fields) = &cont.data
        && fields.iter().all(|field| field.attrs.skip())
    {
        cx.error_spanned_by(
                &cont.original,
                "Tuple struct marked with #[api_schema] must have at least a single non-skipped properties.".to_string(),
            );
    }

    // No default on new-type struct
    if cont.attrs.derive().read()
        && let Data::Struct(Style::Newtype, fields) = &cont.data
    {
        for field in fields.iter() {
            if !field.attrs.skip() && *field.attrs.default() != Default::None {
                cx.error_spanned_by(
                        field.ty,
                        "#[api_property(default)] cannot be used on non-skipped properties of read or read-write newtype structs".to_string(),
                    );
            }
        }
        if *cont.attrs.default() != Default::None && fields.iter().any(|field| !field.attrs.skip())
        {
            cx.error_spanned_by(
                    &cont.original,
                    "#[api_schema(default)] cannot be used on read or read-write newtype structs that contain non-skipped properties".to_string(),
                );
        }
    }

    // No skip_serializing_if on new-type struct
    if cont.attrs.derive().write()
        && let Data::Struct(Style::Newtype, fields) = &cont.data
    {
        for field in fields.iter() {
            if !field.attrs.skip() && field.attrs.skip_serializing_if().is_some() {
                cx.error_spanned_by(
                        field.ty,
                        "#[api_property(skip_serializing_if)] cannot be used on non-skipped properties of write or read-write newtype structs".to_string(),
                    );
            }
        }
    }

    if let Data::Enum(variants) = &cont.data {
        for variant in variants {
            if let Style::Newtype = variant.style {
                // No default on new-type variants
                if cont.attrs.derive() == Derive::ReadWrite || cont.attrs.derive() == Derive::Read {
                    for field in variant.fields.iter() {
                        if !field.attrs.skip() && *field.attrs.default() != Default::None {
                            cx.error_spanned_by(
                        field.ty,
                        "#[api_property(default)] cannot be used on non-skipped properties of newtype variants in read or read-write enums".to_string(),
                    );
                        }
                    }

                    // TODO: Has the '#[api_schema(default)]' an influence on new-type variants?
                    if *cont.attrs.default() != Default::None
                        && variant.fields.iter().any(|field| !field.attrs.skip())
                    {
                        cx.error_spanned_by(
                    &cont.original,
                    "#[api_schema(default)] cannot be used on newtype variants in read or read-write enums that contain non-skipped properties".to_string(),
                );
                    }
                }

                // No skip_serializing_if on new-type variants
                if cont.attrs.derive() == Derive::ReadWrite || cont.attrs.derive() == Derive::Write
                {
                    for field in variant.fields.iter() {
                        if !field.attrs.skip() && field.attrs.skip_serializing_if().is_some() {
                            cx.error_spanned_by(
                        field.ty,
                        "#[api_property(skip_serializing_if)] cannot be used on non-skipped properties of newtype variants in read or read-write enums".to_string(),
                    );
                        }
                    }
                }
            }
        }
    }

    // No default on tuple struct
    if cont.attrs.derive().read()
        && let Data::Struct(Style::Tuple, fields) = &cont.data
    {
        for field in fields.iter() {
            if !field.attrs.skip() && *field.attrs.default() != Default::None {
                cx.error_spanned_by(
                        field.ty,
                        "#[api_property(default)] cannot be used on non-skipped properties of read or read-write tuple structs".to_string(),
                    );
            }
        }
        if *cont.attrs.default() != Default::None && fields.iter().any(|field| !field.attrs.skip())
        {
            cx.error_spanned_by(
                    &cont.original,
                    "#[api_schema(default)] cannot be used on read or read-write tuple structs that contain non-skipped properties".to_string(),
                );
        }
    }

    // No skip_serializing_if on tuple struct
    if cont.attrs.derive().write()
        && let Data::Struct(Style::Tuple, fields) = &cont.data
    {
        for field in fields.iter() {
            if !field.attrs.skip() && field.attrs.skip_serializing_if().is_some() {
                cx.error_spanned_by(
                        field.ty,
                        "#[api_property(skip_serializing_if)] cannot be used on non-skipped properties of write or read-write tuple structs".to_string(),
                    );
            }
        }
    }

    if let Data::Enum(variants) = &cont.data {
        for variant in variants {
            if let Style::Tuple = variant.style {
                // No default on new-type variants
                if cont.attrs.derive() == Derive::ReadWrite || cont.attrs.derive() == Derive::Read {
                    for field in variant.fields.iter() {
                        if !field.attrs.skip() && *field.attrs.default() != Default::None {
                            cx.error_spanned_by(
                        field.ty,
                        "#[api_property(default)] cannot be used on non-skipped properties of tuple variants in read or read-write enums".to_string(),
                    );
                        }
                    }

                    // TODO: Has the '#[api_schema(default)]' an influence on new-type variants?
                    if *cont.attrs.default() != Default::None
                        && variant.fields.iter().any(|field| !field.attrs.skip())
                    {
                        cx.error_spanned_by(
                    &cont.original,
                    "#[api_schema(default)] cannot be used on tuple variants in read or read-write enums that contain non-skipped properties".to_string(),
                );
                    }
                }

                // No skip_serializing_if on new-type variants
                if cont.attrs.derive() == Derive::ReadWrite || cont.attrs.derive() == Derive::Write
                {
                    for field in variant.fields.iter() {
                        if !field.attrs.skip() && field.attrs.skip_serializing_if().is_some() {
                            cx.error_spanned_by(
                        field.ty,
                        "#[api_property(skip_serializing_if)] cannot be used on non-skipped properties of tuple variants in read or read-write enums".to_string(),
                    );
                        }
                    }
                }
            }
        }
    }
}

// If some field of a tuple struct is marked #[api_property(default)] then all fields
// after it must also be marked with that attribute, or the struct must have a
// container-level api_schema(default) attribute. A field's default value is only
// used for tuple fields if the sequence is exhausted at that point; that means
// all subsequent fields will fail to deserialize if they don't have their own
// default.
fn check_default_on_tuple(cx: &Ctxt, cont: &Container) {
    if let Default::None = cont.attrs.default()
        && let Data::Struct(Style::Tuple, fields) = &cont.data
    {
        let mut first_default_index = None;
        for (i, field) in fields.iter().enumerate() {
            // Skipped fields automatically get the #[serde(default)]
            // attribute. We are interested only on non-skipped fields here.
            if field.attrs.skip() {
                continue;
            }
            if let Default::None = field.attrs.default() {
                if let Some(first) = first_default_index {
                    cx.error_spanned_by(
                            field.ty,
                            format!("field must have #[api_property(default)] because previous field {first} has #[api_property(default)]"),
                        );
                }
                continue;
            }
            if first_default_index.is_none() {
                first_default_index = Some(i);
            }
        }
    }
}

// Flattening has some restrictions we can test.
fn check_flatten(cx: &Ctxt, cont: &Container) {
    match &cont.data {
        Data::Enum(variants) => {
            for variant in variants {
                for field in &variant.fields {
                    if !field.attrs.flatten() {
                        continue;
                    }
                    cx.error_spanned_by(
                        field.original,
                        "#[api_property(flatten)] cannot be used on enum variants fields.",
                    );
                }
            }
        }
        Data::Struct(style, fields) => {
            for field in fields {
                check_flatten_field(cx, *style, field);
            }
        }
    }
}

fn check_flatten_field(cx: &Ctxt, style: Style, field: &Field) {
    if !field.attrs.flatten() {
        return;
    }

    if field.attrs.skip() {
        cx.error_spanned_by(
            field.original,
            "#[api_property(flatten)] cannot be combined with #[api_property(skip)]",
        );
    }

    if field.attrs.skip() {
        cx.error_spanned_by(
            field.original,
            "#[api_property(flatten)] cannot be combined with #[api_property(skip)]",
        );
    }

    if field.attrs.skip_serializing_if().is_some() {
        cx.error_spanned_by(
            field.original,
            "#[api_property(flatten)] cannot be combined with #[api_property(skip_serializing_if = \"...\")]",
        );
    }

    match style {
        Style::Tuple => {
            cx.error_spanned_by(
                field.original,
                "#[api_property(flatten)] cannot be used on tuple structs",
            );
        }
        Style::Newtype => {
            cx.error_spanned_by(
                field.original,
                "#[api_property(flatten)] cannot be used on newtype structs",
            );
        }
        _ => {}
    }
}

// The `other` attribute must be used at most once and it must be the last
// variant of an enum.
//
// Inside a `variant_identifier` all variants must be unit variants. Inside a
// `field_identifier` all but possibly one variant must be unit variants. The
// last variant may be a newtype variant which is an implicit "other" case.
fn check_identifier(cx: &Ctxt, cont: &Container) {
    let variants = match &cont.data {
        Data::Enum(variants) => variants,
        Data::Struct(_, _) => return,
    };

    for (i, variant) in variants.iter().enumerate() {
        match (
            variant.style,
            cont.attrs.identifier(),
            variant.attrs.other(),
            cont.attrs.tag(),
        ) {
            // The `other` attribute may not be used in a variant_identifier.
            (_, Identifier::Variant, true, _) => {
                cx.error_spanned_by(
                    &variant.original,
                    "#[api_variant(other)] may not be used on a variant identifier",
                );
            }

            // Variant with `other` attribute cannot appear in untagged enum
            (_, Identifier::No, true, &TagType::None) => {
                cx.error_spanned_by(
                    &variant.original,
                    "#[api_variant(other)] cannot appear on untagged enum",
                );
            }

            // Variant with `other` attribute must be the last one.
            (Style::Unit, Identifier::Field, true, _) | (Style::Unit, Identifier::No, true, _) => {
                if i < variants.len() - 1 {
                    cx.error_spanned_by(
                        &variant.original,
                        "#[api_variant(other)] must be on the last variant",
                    );
                }
            }

            // Variant with `other` attribute must be a unit variant.
            (_, Identifier::Field, true, _) | (_, Identifier::No, true, _) => {
                cx.error_spanned_by(
                    &variant.original,
                    "#[api_variant(other)] must be on a unit variant",
                );
            }

            // Any sort of variant is allowed if this is not an identifier.
            (_, Identifier::No, false, _) => {}

            // Unit variant without `other` attribute is always fine.
            (Style::Unit, _, false, _) => {}

            // The last field is allowed to be a newtype catch-all.
            (Style::Newtype, Identifier::Field, false, _) => {
                if i < variants.len() - 1 {
                    cx.error_spanned_by(
                        &variant.original,
                        format!("`{}` must be the last variant", variant.ident),
                    );
                }
            }

            (_, Identifier::Field, false, _) => {
                cx.error_spanned_by(
                    &variant.original,
                    "#[api_variant(field_identifier)] may only contain unit variants",
                );
            }

            (_, Identifier::Variant, false, _) => {
                cx.error_spanned_by(
                    &variant.original,
                    "#[api_variant(variant_identifier)] may only contain unit variants",
                );
            }
        }
    }
}

// The tag of an internally-tagged struct variant must not be the same as either
// one of its fields, as this would result in duplicate keys in the serialized
// output and/or ambiguity in the to-be-deserialized input.
fn check_internal_tag_field_name_conflict(cx: &Ctxt, cont: &Container) {
    let variants = match &cont.data {
        Data::Enum(variants) => variants,
        Data::Struct(_, _) => return,
    };

    let tag = match cont.attrs.tag() {
        TagType::Internal { tag } => tag.as_str(),
        TagType::External | TagType::Adjacent { .. } | TagType::None => return,
    };

    let diagnose_conflict = || {
        cx.error_spanned_by(
            &cont.original,
            format!("variant field name `{tag}` conflicts with internal tag"),
        );
    };

    for variant in variants {
        match variant.style {
            Style::Struct => {
                if variant.attrs.untagged() {
                    continue;
                }
                for field in &variant.fields {
                    let check = !(field.attrs.skip() || variant.attrs.skip());

                    let name = field.attrs.name();
                    let ser_name = name.serialize_name();

                    if check {
                        if ser_name.value == tag {
                            diagnose_conflict();
                            return;
                        }

                        for de_name in field.attrs.aliases() {
                            if de_name.value == tag {
                                diagnose_conflict();
                                return;
                            }
                        }
                    }
                }
            }
            Style::Unit | Style::Newtype | Style::Tuple => {}
        }
    }
}

// In the case of adjacently-tagged enums, the type and the contents tag must
// differ, for the same reason.
fn check_adjacent_tag_conflict(cx: &Ctxt, cont: &Container) {
    let (type_tag, content_tag) = match cont.attrs.tag() {
        TagType::Adjacent { tag, content } => (tag, content),
        TagType::Internal { .. } | TagType::External | TagType::None => return,
    };

    if type_tag == content_tag {
        cx.error_spanned_by(
            &cont.original,
            format!("enum tags `{type_tag}` for type and content conflict with each other"),
        );
    }
}

// Enums and unit structs cannot be transparent.
fn check_transparent(cx: &Ctxt, cont: &mut Container, derive: Derive) {
    if !cont.attrs.transparent() {
        return;
    }

    if cont.attrs.type_from().is_some() {
        cx.error_spanned_by(
            &cont.original,
            "#[api_schema(transparent)] is not allowed with #[api_schema(from = \"...\")]",
        );
    }

    if cont.attrs.type_try_from().is_some() {
        cx.error_spanned_by(
            &cont.original,
            "#[api_schema(transparent)] is not allowed with #[api_schema(try_from = \"...\")]",
        );
    }

    if cont.attrs.type_into().is_some() {
        cx.error_spanned_by(
            &cont.original,
            "#[api_schema(transparent)] is not allowed with #[api_schema(into = \"...\")]",
        );
    }

    let fields = match &mut cont.data {
        Data::Enum(_) => {
            cx.error_spanned_by(
                &cont.original,
                "#[api_schema(transparent)] is not allowed on an enum",
            );
            return;
        }
        Data::Struct(Style::Unit, _) => {
            cx.error_spanned_by(
                &cont.original,
                "#[api_schema(transparent)] is not allowed on a unit struct",
            );
            return;
        }
        Data::Struct(_, fields) => fields,
    };

    let mut transparent_field = None;

    for field in fields {
        if allow_transparent(field, derive) {
            if transparent_field.is_some() {
                cx.error_spanned_by(
                    &cont.original,
                    "#[api_property(transparent)] requires struct to have at most one transparent field",
                );
                return;
            }
            transparent_field = Some(field);
        }
    }

    match transparent_field {
        Some(transparent_field) => transparent_field.attrs.mark_transparent(),
        None => match derive {
            Derive::Write => {
                cx.error_spanned_by(
                    &cont.original,
                    "#[api_property(transparent)] requires at least one field that is not skipped",
                );
            }
            Derive::Read => {
                cx.error_spanned_by(
                    &cont.original,
                    "#[api_property(transparent)] requires at least one field that is neither skipped nor has a default",
                );
            }
            Derive::ReadWrite => {
                cx.error_spanned_by(
                    &cont.original,
                    "#[api_property(transparent)] requires at least one field that is neither skipped nor has a default",
                );
            }
        },
    }
}

fn allow_transparent(field: &Field, derive: Derive) -> bool {
    if let Type::Path(ty) = ungroup(field.ty)
        && let Some(seg) = ty.path.segments.last()
        && seg.ident == "PhantomData"
    {
        return false;
    }

    match derive {
        Derive::Write => !field.attrs.skip(),
        Derive::Read => !field.attrs.skip() && field.attrs.default().is_none(),
        Derive::ReadWrite => !field.attrs.skip() && field.attrs.default().is_none(),
    }
}

fn check_from_and_try_from(cx: &Ctxt, cont: &mut Container) {
    if cont.attrs.type_from().is_some() && cont.attrs.type_try_from().is_some() {
        cx.error_spanned_by(
            &cont.original,
            "#[api_schema(from = \"...\")] and #[api_schema(try_from = \"...\")] conflict with each other",
        );
    }
}
