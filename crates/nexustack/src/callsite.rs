/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */
use std::{
    fmt::{Debug, Display},
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_SEQ_NUM: AtomicUsize = AtomicUsize::new(1);

/// Represents a unique location in the source code where an event or operation occurs.
///
/// Each `Callsite` instance is assigned a unique sequence number upon creation,
/// and stores the file name, line, and column number of its origin.
/// This is useful for debugging, logging, or tracing code execution.
///
/// # Examples
///
/// ```rust
/// use nexustack::callsite;
///
/// callsite!(cs1);
/// callsite!(cs2);
///
/// assert_ne!(*cs1, *cs2); // Each callsite is unique
/// println!("Callsite 1: {}", cs1);
/// println!("Callsite 2: {}", cs2);
/// ```
#[derive(Copy, Clone, Debug)]
pub struct Callsite {
    /// Unique sequence number for this callsite.
    seq_num: usize,
    /// The file name where the callsite was created.
    file: &'static str,
    /// The line number in the file.
    line: usize,
    /// The column number in the file.
    column: usize,
}

impl Callsite {
    /// Creates a new `Callsite` instance.
    ///
    /// This method is intended for use by the `callsite!` macro and is not part of the public API.
    ///
    /// # Arguments
    ///
    /// * `file` - The file name where the callsite is located.
    /// * `line` - The line number in the file.
    /// * `column` - The column number in the file.
    ///
    /// # Returns
    ///
    /// A new `Callsite` instance with a unique sequence number.
    #[doc(hidden)]
    pub fn new(file: &'static str, line: usize, column: usize) -> Self {
        Self {
            seq_num: NEXT_SEQ_NUM.fetch_add(1, Ordering::Relaxed),
            file,
            line,
            column,
        }
    }

    /// Returns the file name where this callsite was created.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexustack::{callsite, Callsite};
    /// callsite!(cs);
    /// assert!(cs.file().ends_with(".rs"));
    /// ```
    #[must_use]
    pub const fn file(&self) -> &'static str {
        self.file
    }

    /// Returns the line number of this callsite.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexustack::callsite;
    /// callsite!(cs);
    /// assert!(cs.line() > 0);
    /// ```
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Returns the column number of this callsite.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexustack::callsite;
    /// callsite!(cs);
    /// assert!(cs.column() > 0);
    /// ```
    #[must_use]
    pub const fn column(&self) -> usize {
        self.column
    }
}

impl PartialEq for Callsite {
    fn eq(&self, other: &Self) -> bool {
        self.seq_num == other.seq_num
    }
}

impl Eq for Callsite {}

impl Display for Callsite {
    /// Formats the callsite as `file:line:column`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexustack::callsite;
    /// callsite!(cs);
    /// println!("{}", cs); // e.g., "src/main.rs:10:5"
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file(), self.line(), self.column())
    }
}

/// Macro to initialize a callsite with a unique identifier and provide a dereferenceable
/// static instance of the [`Callsite`].
///
/// This macro creates a struct and a static instance of it, which dereferences to a
/// lazily-initialized [`Callsite`]. The callsite is initialized the first time it is accessed.
///
/// # Arguments
///
/// * `$v` - The visibility of the generated struct and static instance.
/// * `$i` - The identifier for the generated struct and static instance.
///
/// # Examples
///
/// ```rust
/// use nexustack::{callsite};
///
/// callsite!(pub MY_CALLSITE);
///
/// println!("Callsite: {}", MY_CALLSITE);
/// ```
#[macro_export]
macro_rules! callsite {
    ($v:vis $i:ident) => {

        #[allow(missing_copy_implementations)]
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
        $v struct $i {__private_field: ()}

        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        $v static $i: $i = $i {__private_field: ()};

        const _: () = {
            static CALLSITE: $crate::__private::utils::AtomicOnceCell<$crate::Callsite> = $crate::__private::utils::AtomicOnceCell::new();
            // TODO: Macro hygiene
            impl ::std::ops::Deref for $i {
                type Target = $crate::Callsite;

                fn deref(&self) -> &Self::Target {
                    CALLSITE.get_or_init(|| $crate::Callsite::new(file!(), line!() as usize, column!() as usize))
                }
            }

            impl ::std::fmt::Debug for $i {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    ::std::fmt::Debug::fmt(&**self, f)
                }
            }

            impl ::std::fmt::Display for $i {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    ::std::fmt::Display::fmt(&**self, f)
                }
            }
        };
    };
}
