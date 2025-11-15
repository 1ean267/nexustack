/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://docs.rs/serde/latest/src/serde/ser/impls.rs.html
 */

use crate::openapi::{schema::Schema, schema_builder::SchemaBuilder};

#[cfg(target_has_atomic = "8")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", target_has_atomic = "8"))))]
impl Schema for std::sync::atomic::AtomicBool {
    type Example = <bool as Schema>::Example;
    type Examples = <bool as Schema>::Examples;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        <bool as Schema>::describe(schema_builder)
    }
}

macro_rules! atomic_impl {
    ($ty:path, $size:expr, $primitive:ty) => {
        #[cfg(target_has_atomic = $size)]
        #[cfg_attr(docsrs, doc(cfg(all(feature = "std", target_has_atomic = $size))))]
        impl Schema for $ty {
            type Example = <$primitive as Schema>::Example;
            type Examples = <$primitive as Schema>::Examples;

            #[inline]
            fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
            where
                B: SchemaBuilder<Self::Examples>,
            {
                <$primitive as Schema>::describe(schema_builder)
            }
        }
    };
}

atomic_impl!(std::sync::atomic::AtomicI8, "8", i8);
atomic_impl!(std::sync::atomic::AtomicI16, "16", i16);
atomic_impl!(std::sync::atomic::AtomicI32, "32", i32);
atomic_impl!(std::sync::atomic::AtomicI64, "64", i64);
atomic_impl!(std::sync::atomic::AtomicU8, "8", u8);
atomic_impl!(std::sync::atomic::AtomicU16, "16", u16);
atomic_impl!(std::sync::atomic::AtomicU32, "32", u32);
atomic_impl!(std::sync::atomic::AtomicU64, "64", u64);
