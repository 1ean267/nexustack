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
    use crate::application::Node;

    /// A sealed trait to prevent external implementations of `Index`.
    pub trait Seal<T> {}

    impl Seal<()> for super::Here {}
    impl<T> Seal<()> for super::InTail<T> {}
    impl<T> Seal<()> for super::InHead<T> {}

    impl<T> Seal<super::Here> for T {}

    impl<Head, Tail, HeadIndex> Seal<super::InHead<HeadIndex>> for Node<Head, Tail>
    where
        HeadIndex: super::Index,
        Head: super::Chain<HeadIndex>,
    {
    }

    impl<Head, Tail, TailIndex> Seal<super::InTail<TailIndex>> for Node<Head, Tail>
    where
        TailIndex: super::Index,
        Tail: super::Chain<TailIndex>,
    {
    }
}

/// A trait representing an index in a type-level list.
///
/// This trait is sealed and cannot be implemented outside of this crate.
pub trait Index: seal::Seal<()> {}

/// Represents the starting point (head) of a type-level list.
pub struct Here {
    _priv: (),
}

/// Represents the current element in a type-level list.
///
/// # Type Parameters
/// - `T`: The type of the current element.
pub struct InHead<T> {
    _marker: PhantomData<T>,
}

/// Represents the next element in a type-level list.
///
/// # Type Parameters
/// - `T`: The type of the next element.
pub struct InTail<T> {
    _marker: PhantomData<T>,
}

impl Index for Here {}

impl<T: Index> Index for InHead<T> {}

impl<T: Index> Index for InTail<T> {}

/// A trait for accessing elements in a type-level list.
///
/// # Type Parameters
/// - `I`: The index of the element in the list, which must implement the `Index` trait.
///
/// This trait is sealed and cannot be implemented outside of this crate.
pub trait Chain<I: Index>: seal::Seal<I> {
    /// The type of the element at the specified index.
    type Element;

    /// Retrieves an immutable reference to the element at the specified index.
    ///
    /// # Returns
    /// A reference to the element of type `Self::Element`.
    fn get(&self) -> &Self::Element;

    /// Retrieves a mutable reference to the element at the specified index.
    ///
    /// # Returns
    /// A mutable reference to the element of type `Self::Element`.
    fn get_mut(&mut self) -> &mut Self::Element;
}

impl<T> Chain<Here> for T {
    type Element = T;

    /// Retrieves an immutable reference to the current element.
    ///
    /// # Returns
    /// A reference to the current element of type `T`.
    fn get(&self) -> &T {
        self
    }

    /// Retrieves a mutable reference to the current element.
    ///
    /// # Returns
    /// A mutable reference to the current element of type `T`.
    fn get_mut(&mut self) -> &mut T {
        self
    }
}
