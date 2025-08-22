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

#[derive(Debug)]
pub struct AtomicOnceCell<T> {
    inner: UnsafeCell<Option<T>>,
    state: AtomicU8,
}

enum AtomicOnceCellState {
    Uninit,
    Busy,
    Init,
}

impl<T> AtomicOnceCell<T> {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(None),
            state: AtomicU8::new(AtomicOnceCellState::Uninit as u8),
        }
    }

    pub const fn from_value(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(Some(value)),
            state: AtomicU8::new(AtomicOnceCellState::Init as u8),
        }
    }

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

    #[cfg(any())]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.inner.get_mut().as_mut()
    }

    pub fn set(&self, value: T) -> Result<(), T> {
        let res = self.state.compare_exchange(
            AtomicOnceCellState::Uninit as u8,
            AtomicOnceCellState::Busy as u8,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );

        if res.is_ok() {
            // SAFETY: There are no readers of this memory location, as the state is not set to init yes and
            //         we are the only writer, as we are the one that set the busy flag.
            *unsafe { &mut *self.inner.get() } = Some(value);
            self.state
                .store(AtomicOnceCellState::Init as u8, Ordering::Release);

            return Ok(());
        }

        Err(value)
    }

    #[cfg(any())]
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

        if let Ok(_) = res {
            let value = f();

            // SAFETY: There are no readers of this memory location, as the state is not set to init yes and
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

    #[cfg(any())]
    pub fn into_inner(self) -> Option<T> {
        self.inner.into_inner()
    }

    #[cfg(any())]
    pub fn take(&mut self) -> Option<T> {
        let result = std::mem::replace(self.inner.get_mut(), None);
        self.state
            .store(AtomicOnceCellState::Uninit as u8, Ordering::Release);
        result
    }
}

impl<T: Clone> Clone for AtomicOnceCell<T> {
    fn clone(&self) -> Self {
        match self.get() {
            Some(value) => Self::from_value(value.clone()),
            None => Self::new(),
        }
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
