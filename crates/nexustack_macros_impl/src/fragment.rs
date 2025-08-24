/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://github.com/serde-rs/serde/blob/master/serde_derive/src/fragment.rs
 */

use proc_macro2::TokenStream;
use quote::ToTokens;

pub enum Fragment {
    /// Tokens that can be used as an expression.
    Expr(TokenStream),
    /// Tokens that can be used inside a block. The surrounding curly braces are
    /// not part of these tokens.
    Block(TokenStream),
}

macro_rules! quote_expr {
    ($($tt:tt)*) => {
        $crate::fragment::Fragment::Expr(quote!($($tt)*))
    }
}

macro_rules! quote_block {
    ($($tt:tt)*) => {
        $crate::fragment::Fragment::Block(quote!($($tt)*))
    }
}

/// Interpolate a fragment as the statements of a block.
pub struct Stmts(pub Fragment);
impl ToTokens for Stmts {
    fn to_tokens(&self, out: &mut TokenStream) {
        match &self.0 {
            Fragment::Expr(expr) => expr.to_tokens(out),
            Fragment::Block(block) => block.to_tokens(out),
        }
    }
}

impl AsRef<TokenStream> for Fragment {
    fn as_ref(&self) -> &TokenStream {
        match self {
            Fragment::Expr(expr) => expr,
            Fragment::Block(block) => block,
        }
    }
}
