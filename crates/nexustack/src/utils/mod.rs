/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

mod atomic_once_cell;
mod optional;

pub use atomic_once_cell::AtomicOnceCell;
pub use optional::Optional;

#[allow(dead_code)]
pub const fn ensure_send<T: Send>() {}

#[allow(dead_code)]
pub const fn ensure_sync<T: Sync>() {}

#[allow(dead_code)]
pub const fn ensure_clone<T: Clone>() {}

macro_rules! b_tree_set {
    () => {
        std::collections::BTreeSet::new()
    };
    ($($x:expr),+ $(,)?) => ({
        std::collections::BTreeSet::from([$($x,)+])
    });
}

pub(crate) use b_tree_set;

macro_rules! hash_map {
    () => {
        std::collections::HashMap::new()
    };
    ($($k:expr => $v:expr),+ $(,)?) => {
        std::collections::HashMap::from([
            $(($k,$v),)+
        ])
    };
}

pub(crate) use hash_map;
