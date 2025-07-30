/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

pub mod atomic_once_cell;

#[allow(dead_code)]
pub const fn ensure_send<T: Send>() -> () {}

#[allow(dead_code)]
pub const fn ensure_sync<T: Sync>() -> () {}

#[allow(dead_code)]
pub const fn ensure_clone<T: Clone>() -> () {}
