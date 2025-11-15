/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

#[cfg(feature = "openapi")]
mod callsite;

#[cfg(any(feature = "openapi", feature = "cron", feature = "inject"))]
mod ctxt;

#[cfg(feature = "openapi")]
mod receiver;

#[cfg(any(feature = "openapi", feature = "cron", feature = "inject"))]
pub(crate) mod attr;
#[cfg(feature = "openapi")]
pub(crate) mod case;
#[cfg(feature = "openapi")]
pub(crate) mod respan;

#[cfg(any(feature = "openapi", feature = "cron", feature = "inject"))]
pub(crate) mod symbol;

#[cfg(feature = "openapi")]
pub use self::callsite::callsite;

#[cfg(any(feature = "openapi", feature = "cron", feature = "inject"))]
pub use self::ctxt::Ctxt;

#[cfg(feature = "openapi")]
pub use self::receiver::replace_receiver;

#[cfg(feature = "openapi")]
pub(crate) trait IntoIteratorExt: IntoIterator {
    fn exactly_one(self) -> Option<Self::Item>;
}

#[cfg(feature = "openapi")]
impl<I: IntoIterator> IntoIteratorExt for I {
    fn exactly_one(self) -> Option<Self::Item> {
        let mut iter = self.into_iter();

        iter.next()
            .and_then(|item| iter.next().is_none().then_some(item))
    }
}
