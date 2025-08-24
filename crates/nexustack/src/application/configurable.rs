/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

// TODO: Allow the closure to error?

/// A trait for types that can be configured using a closure.
pub(crate) trait Configurable<'a> {
    /// Configures the item using the provided closure.
    ///
    /// # Arguments
    ///
    /// * `configure` - A closure that takes a mutable reference to the item (`&mut I`) and performs configuration.
    ///
    /// # Type Parameters
    ///
    /// * `I` - The item to be configured.
    /// * `C` - The closure type. Must implement `FnOnce(&mut I) -> ()`.
    ///
    /// # Errors
    ///
    /// Returns `Err(C)` if the configuration cannot be applied.
    fn configure<I: 'a, C>(&mut self, configure: C) -> Result<(), C>
    where
        C: FnOnce(&mut I);
}
