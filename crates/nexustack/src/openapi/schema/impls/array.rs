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
    SchemaExamples,
    schema::{
        Schema,
        builder::{SchemaBuilder, TupleSchemaBuilder},
    },
};

// Does not require T: Schema.
impl<T> Schema for [T; 0] {
    type Example = [(); 0];
    type Examples = std::iter::Once<Self::Example>;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        schema_builder
            .describe_tuple(0, None, || Ok(std::iter::once([])), false)?
            .end()
    }
}

macro_rules! array_impls {
    ($($len:tt)+) => {
        $(
            impl<T> Schema for [T; $len]
            where
                T: Schema,
            {
                type Example = [<T as Schema>::Example; $len];
                type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;

                #[inline]
                fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
                where
                    B: SchemaBuilder<Self::Examples>,
                {
                    let is_human_readable = schema_builder.is_human_readable();
                    let mut tuple_schema_builder = schema_builder.describe_tuple(
                        2,
                        None,
                        || {
                            Ok([
                                std::iter::repeat(())
                                    .flat_map(|_| {
                                        match <T as SchemaExamples>::examples(is_human_readable) {
                                            Ok(p) => either::Either::Left(p.map(Ok)),
                                            Err(e) => either::Either::Right(std::iter::once(Err(e))),
                                        }
                                    })
                                    .take($len)
                                    .collect::<Result<Vec<_>,_>>()?
                                    .try_into()
                                    .map_err(|_| "Wrong array size")
                                    .unwrap(),
                                std::iter::repeat(())
                                    .flat_map(|_| {
                                        match <T as SchemaExamples>::examples(is_human_readable) {
                                            Ok(p) => either::Either::Left(p.map(Ok)),
                                            Err(e) => either::Either::Right(std::iter::once(Err(e))),
                                        }
                                    })
                                    .take($len)
                                    .collect::<Result<Vec<_>,_>>()?
                                    .try_into()
                                    .map_err(|_| "Wrong array size")
                                    .unwrap(),
                                std::iter::repeat(())
                                    .flat_map(|_| {
                                        match <T as SchemaExamples>::examples(is_human_readable) {
                                            Ok(p) => either::Either::Left(p.map(Ok)),
                                            Err(e) => either::Either::Right(std::iter::once(Err(e))),
                                        }
                                    })
                                    .take($len)
                                    .collect::<Result<Vec<_>,_>>()?
                                    .try_into()
                                    .map_err(|_| "Wrong array size")
                                    .unwrap()
                            ])
                        },
                        false,
                    )?;

                    for _ in 0..$len {
                        TupleSchemaBuilder::collect_element(
                            &mut tuple_schema_builder,
                            None,
                            false,
                            <T>::describe,
                        )?;
                    }

                    tuple_schema_builder.end()
                }
            }
        )+
    }
}

array_impls! {
    1 2 3 4 5 6 7 8 9 10
    11 12 13 14 15 16 17 18 19 20
    21 22 23 24 25 26 27 28 29 30
    31 32
}
