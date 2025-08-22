/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![warn(clippy::style)]
#![warn(clippy::perf)]
#![warn(clippy::complexity)]
#![warn(clippy::suspicious)]
#![warn(clippy::correctness)]
#![allow(clippy::redundant_pub_crate)]
#![cfg_attr(feature = "unsize", feature(unsize))]

#[cfg(feature = "derive")]
extern crate nexustack_macros;

#[doc = include_str!("./inject/README.md")]
pub mod inject;
mod utils;
