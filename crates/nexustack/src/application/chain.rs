/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on Frunk (https://github.com/lloydmeta/frunk)
 *
 * The MIT License (MIT)
 * Copyright (c) 2016 by Lloyd Chan
 */

use std::marker::PhantomData;

mod seal {
    pub trait Seal {}

    impl Seal for super::Here {}
    impl<T> Seal for super::There<T> {}
}

/// A trait representing an index in a type-level list.
///
/// This trait is sealed and cannot be implemented outside of this crate.
pub trait Index: seal::Seal {}

/// Represents the starting point (head) of a type-level list.
pub struct Here {
    _priv: (),
}

/// Represents the next element in a type-level list.
pub struct There<T> {
    _marker: PhantomData<T>,
}

impl Index for Here {}

impl<T: Index> Index for There<T> {}

/// A trait for accessing elements in a type-level list.
///
/// # Type Parameters
/// - `S`: The type of the element to access.
/// - `I`: The index of the element in the list, which must implement the `Index` trait.
// TODO: Should not be implementable outside of this crate
pub trait Chain<S, I: Index> {
    /// Retrieves an immutable reference to the element at the specified index.
    ///
    /// # Returns
    /// A reference to the element of type `S`.
    fn get(&self) -> &S;

    /// Retrieves a mutable reference to the element at the specified index.
    ///
    /// # Returns
    /// A mutable reference to the element of type `S`.
    fn get_mut(&mut self) -> &mut S;
}
