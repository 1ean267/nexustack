/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::{ensure_send, ensure_sync};
use std::fmt::Debug;
use std::sync::atomic::Ordering;
use std::{cell::UnsafeCell, sync::atomic::AtomicU8};

const _: () = ensure_send::<AtomicOnceCell<()>>();
const _: () = ensure_sync::<AtomicOnceCell<()>>();

/// A thread-safe, single-assignment cell that can be initialized once and read many times.
///
/// The `AtomicOnceCell` is a synchronization primitive that allows you to store a value that
/// can only be written once but can be read multiple times. It is useful for scenarios where
/// you need to lazily initialize a value in a thread-safe manner.
///
/// # Example
///
/// ```rust
/// use nexustack::__private::utils::AtomicOnceCell;
/// use std::{sync::Arc, thread};
///
/// let cell = Arc::new(AtomicOnceCell::new());
/// let cell_clone = Arc::clone(&cell);
///
/// let handle = thread::spawn(move || {
///     cell_clone.set(42).unwrap();
/// });
///
/// handle.join().unwrap();
///
/// assert_eq!(cell.get(), Some(&42));
/// ```
#[derive(Debug)]
pub struct AtomicOnceCell<T> {
    inner: UnsafeCell<Option<T>>,
    state: AtomicU8,
}

/// Represents the state of an `AtomicOnceCell`.
///
/// This enum is used internally to track whether the cell is uninitialized, being initialized,
/// or fully initialized.
enum AtomicOnceCellState {
    /// The cell is uninitialized and can accept a value.
    Uninit,

    /// The cell is currently being initialized by another thread.
    Busy,

    /// The cell has been fully initialized and contains a value.
    Init,
}

impl<T> AtomicOnceCell<T> {
    /// Creates a new, uninitialized `AtomicOnceCell`.
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(None),
            state: AtomicU8::new(AtomicOnceCellState::Uninit as u8),
        }
    }

    /// Creates a new `AtomicOnceCell` initialized with the given value.
    pub const fn from_value(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(Some(value)),
            state: AtomicU8::new(AtomicOnceCellState::Init as u8),
        }
    }

    /// Returns a reference to the value if it has been initialized, or `None` otherwise.
    pub fn get(&self) -> Option<&T> {
        let state = self.state.load(Ordering::Acquire);

        if state != AtomicOnceCellState::Init as u8 {
            return None;
        }

        // SAFETY: We read the value after checking that it was fully initialized in a thread-safe way. As
        //         this cell is only written exactly once, when the init state is set, there are no
        //         further writes to the memory location.
        let result = unsafe { &*self.inner.get() }.as_ref();
        debug_assert!(result.is_some());
        result
    }

    /// Returns a mutable reference to the value if it has been initialized, or `None` otherwise.
    ///
    /// This function is only available when the `AtomicOnceCell` is not shared between threads.
    pub const fn get_mut(&mut self) -> Option<&mut T> {
        self.inner.get_mut().as_mut()
    }

    /// Sets the value of the cell. Returns an error containing the value if the cell
    /// has already been initialized.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexustack::__private::utils::AtomicOnceCell;
    ///
    /// let cell = AtomicOnceCell::new();
    /// assert!(cell.set(42).is_ok());
    /// assert!(cell.set(43).is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// This function returns an `Err` containing the value if the cell has already been initialized.
    /// The cell can only be set once, and subsequent attempts to set it will fail.
    pub fn set(&self, value: T) -> Result<(), T> {
        let res = self.state.compare_exchange(
            AtomicOnceCellState::Uninit as u8,
            AtomicOnceCellState::Busy as u8,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );

        if res.is_ok() {
            // SAFETY: There are no readers of this memory location, as the state is not set to init yet and
            //         we are the only writer, as we are the one that set the busy flag.
            *unsafe { &mut *self.inner.get() } = Some(value);
            self.state
                .store(AtomicOnceCellState::Init as u8, Ordering::Release);

            return Ok(());
        }

        Err(value)
    }

    /// Returns a reference to the value, initializing it with the provided closure if necessary.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexustack::__private::utils::AtomicOnceCell;
    ///
    /// let cell = AtomicOnceCell::new();
    /// let value = cell.get_or_init(|| 42);
    /// assert_eq!(value, &42);
    /// ```
    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        let res = self.state.compare_exchange(
            AtomicOnceCellState::Uninit as u8,
            AtomicOnceCellState::Busy as u8,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );

        if res.is_ok() {
            let value = f();

            // SAFETY: There are no readers of this memory location, as the state is not set to init yet and
            //         we are the only writer, as we are the one that set the busy flag.
            *unsafe { &mut *self.inner.get() } = Some(value);
            self.state
                .store(AtomicOnceCellState::Init as u8, Ordering::Release);
        }

        loop {
            if let Some(value) = self.get() {
                break value;
            }

            std::hint::spin_loop();
        }
    }

    /// Consumes the `AtomicOnceCell` and returns the inner value, if it was initialized.
    ///
    /// This function is only available when the `AtomicOnceCell` is not shared between threads.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexustack::__private::utils::AtomicOnceCell;
    ///
    /// let cell = AtomicOnceCell::from_value(42);
    /// assert_eq!(cell.into_inner(), Some(42));
    /// ```
    pub fn into_inner(self) -> Option<T> {
        self.inner.into_inner()
    }

    /// Takes the value out of the `AtomicOnceCell`, leaving it uninitialized.
    ///
    /// This function is only available when the `AtomicOnceCell` is not shared between threads.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexustack::__private::utils::AtomicOnceCell;
    ///
    /// let mut cell = AtomicOnceCell::from_value(42);
    /// assert_eq!(cell.take(), Some(42));
    /// assert!(cell.get().is_none());
    /// ```
    pub fn take(&mut self) -> Option<T> {
        let result = self.inner.get_mut().take();
        self.state
            .store(AtomicOnceCellState::Uninit as u8, Ordering::Release);
        result
    }
}

impl<T: Clone> Clone for AtomicOnceCell<T> {
    fn clone(&self) -> Self {
        self.get()
            .map_or_else(Self::new, |value| Self::from_value(value.clone()))
    }
}

impl<T> Default for AtomicOnceCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<T> for AtomicOnceCell<T> {
    fn from(value: T) -> Self {
        Self::from_value(value)
    }
}

impl<T: PartialEq> PartialEq for AtomicOnceCell<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T: Eq> Eq for AtomicOnceCell<T> {}

unsafe impl<T: Send> Send for AtomicOnceCell<T> {}
unsafe impl<T: Sync + Send> Sync for AtomicOnceCell<T> {}
