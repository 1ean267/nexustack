/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/// A trait for extracting the inner type from an optional type.
pub trait Optional {
    /// The inner type contained within the optional type.
    type Inner;
}

impl<T> Optional for Option<T> {
    type Inner = T;
}
