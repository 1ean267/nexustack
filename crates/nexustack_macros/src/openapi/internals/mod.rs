/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

pub(crate) mod ast;
pub(crate) mod attr;
pub(crate) mod check;
pub(crate) mod name;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Derive {
    Write,
    Read,
    ReadWrite,
}

pub fn ungroup(mut ty: &syn::Type) -> &syn::Type {
    while let syn::Type::Group(group) = ty {
        ty = &group.elem;
    }
    ty
}

impl Derive {
    pub fn read(&self) -> bool {
        match self {
            Self::Write => false,
            Self::Read => true,
            Self::ReadWrite => true,
        }
    }

    pub fn write(&self) -> bool {
        match self {
            Self::Write => true,
            Self::Read => false,
            Self::ReadWrite => true,
        }
    }
}
