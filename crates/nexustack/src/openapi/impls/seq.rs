/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on https://docs.rs/serde/latest/src/serde/ser/impls.rs.html
 */

use crate::openapi::{
    example::SchemaExamples,
    schema::Schema,
    schema_builder::{IntoSchemaBuilder, SchemaBuilder},
};

impl<T> Schema for [T]
where
    T: Schema,
{
    type Example = Vec<<T as Schema>::Example>;
    type Examples = <[Self::Example; 4] as IntoIterator>::IntoIter;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        let is_human_readable = schema_builder.is_human_readable();
        <T as Schema>::describe(
            schema_builder
                .describe_seq(
                    None,
                    None,
                    false,
                    None,
                    || {
                        Ok([
                            Vec::new(),
                            std::iter::repeat(())
                                .flat_map(|()| {
                                    match <T as SchemaExamples>::examples(is_human_readable) {
                                        Ok(p) => either::Either::Left(p.map(Ok)),
                                        Err(e) => either::Either::Right(std::iter::once(Err(e))),
                                    }
                                })
                                .take(1)
                                .collect::<Result<Vec<_>, _>>()?,
                            std::iter::repeat(())
                                .flat_map(|()| {
                                    match <T as SchemaExamples>::examples(is_human_readable) {
                                        Ok(p) => either::Either::Left(p.map(Ok)),
                                        Err(e) => either::Either::Right(std::iter::once(Err(e))),
                                    }
                                })
                                .take(2)
                                .collect::<Result<Vec<_>, _>>()?,
                            std::iter::repeat(())
                                .flat_map(|()| {
                                    match <T as SchemaExamples>::examples(is_human_readable) {
                                        Ok(p) => either::Either::Left(p.map(Ok)),
                                        Err(e) => either::Either::Right(std::iter::once(Err(e))),
                                    }
                                })
                                .take(10)
                                .collect::<Result<Vec<_>, _>>()?,
                        ])
                    },
                    false,
                )?
                .into_schema_builder(),
        )
    }
}

macro_rules! seq_impl {
    (
        use $u:path;
        $(#[$attr:meta])*
        $unique:ident => $ty:ident <T $(: $tbound1:ident $(+ $tbound2:path)*)* $(, $typaram:ident : $bound:path)*>
    ) => {
        use $u;
        $(#[$attr])*
        impl<T $(, $typaram)*> Schema for $ty<T $(, $typaram)*>
        where
            T: Schema $(+ $tbound1 $(+ $tbound2)*)*,
            $($typaram: $bound,)*
        {
            type Example = Vec<<T as Schema>::Example>;
            type Examples =
                <[Self::Example; 4] as IntoIterator>::IntoIter;

            #[inline]
            fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
            where
                B: SchemaBuilder<Self::Examples>,
            {
                let is_human_readable = schema_builder.is_human_readable();
                <<$ty<T $(, $typaram)*> as IntoIterator>::Item>::describe(
                    schema_builder
                        .describe_seq(
                            None,
                            None,
                            $unique,
                            None,
                            || {
                                Ok([
                                    Vec::new(),
                                    std::iter::repeat(())
                                        .flat_map(|_| {
                                            match <T as SchemaExamples>::examples(is_human_readable) {
                                                Ok(p) => either::Either::Left(p.map(Ok)),
                                                Err(e) => either::Either::Right(std::iter::once(Err(e))),
                                            }
                                        })
                                        .take(1)
                                        .collect::<Result<Vec<_>, _>>()?,
                                    std::iter::repeat(())
                                        .flat_map(|_| {
                                            match <T as SchemaExamples>::examples(is_human_readable) {
                                                Ok(p) => either::Either::Left(p.map(Ok)),
                                                Err(e) => either::Either::Right(std::iter::once(Err(e))),
                                            }
                                        })
                                        .take(2)
                                        .collect::<Result<Vec<_>, _>>()?,
                                    std::iter::repeat(())
                                        .flat_map(|_| {
                                            match <T as SchemaExamples>::examples(is_human_readable) {
                                                Ok(p) => either::Either::Left(p.map(Ok)),
                                                Err(e) => either::Either::Right(std::iter::once(Err(e))),
                                            }
                                        })
                                        .take(10)
                                        .collect::<Result<Vec<_>, _>>()?,
                                ])
                            },
                            false,
                        )?
                        .into_schema_builder(),
                )
            }
        }
    }
}

seq_impl! {
    use std::collections::BinaryHeap;

    false => BinaryHeap<T: Ord>
}

seq_impl! {
    use std::collections::BTreeSet;

    true => BTreeSet<T: Ord>
}

seq_impl! {
    use std::collections::HashSet;

    true => HashSet<T: Eq + std::hash::Hash, H: std::hash::BuildHasher>
}

seq_impl! {
    use std::collections::LinkedList;

    false => LinkedList<T>
}

seq_impl! {
    use std::vec::Vec;

    false => Vec<T>
}

seq_impl! {
    use std::collections::VecDeque;

    false => VecDeque<T>
}
