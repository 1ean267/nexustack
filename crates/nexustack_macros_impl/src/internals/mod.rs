/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

pub mod callsite;
mod ctxt;
mod receiver;

pub(crate) mod case;
pub(crate) mod respan;

pub use self::callsite::callsite;
pub use self::ctxt::Ctxt;
pub use self::receiver::replace_receiver;

pub(crate) trait IntoIteratorExt: IntoIterator {
    fn exactly_one(self) -> Option<Self::Item>;
}

impl<I: IntoIterator> IntoIteratorExt for I {
    fn exactly_one(self) -> Option<Self::Item> {
        let mut iter = self.into_iter();

        iter.next()
            .and_then(|item| iter.next().is_none().then_some(item))
    }
}
