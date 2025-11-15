/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use syn::visit::Visit;
use syn::visit_mut::VisitMut;

use crate::openapi::internals::ast::{Container, Field};

pub fn make_lifetimes_static(t: &mut syn::Type) {
    struct LifetimeReplacer;

    impl VisitMut for LifetimeReplacer {
        fn visit_lifetime_mut(&mut self, lt: &mut syn::Lifetime) {
            if lt.ident != "'static" {
                *lt = syn::Lifetime::new("'static", lt.span());
            }
        }
    }

    let mut replacer = LifetimeReplacer;
    replacer.visit_type_mut(t);
}

pub fn field_contains_generic_params(field: &Field, cont: &Container) -> bool {
    let type_params: Vec<&syn::Ident> = cont
        .generics
        .type_params()
        .map(|type_param| &type_param.ident)
        .collect::<Vec<_>>();

    if type_params.is_empty() {
        return false;
    }

    let ty = field.ty;

    let mut visitor = TypeParamFinder {
        type_params,
        found: false,
    };

    visitor.visit_type(ty);
    visitor.found
}

struct TypeParamFinder<'a> {
    type_params: Vec<&'a syn::Ident>,
    found: bool,
}

impl<'a, 'b> syn::visit::Visit<'b> for TypeParamFinder<'a> {
    fn visit_ident(&mut self, i: &'b syn::Ident) {
        if self.type_params.contains(&i) {
            self.found = true;
        }
    }
}
