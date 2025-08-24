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
    schema_builder::{MapSchemaBuilder, SchemaBuilder},
};

pub struct FakeMap<K, V>(Vec<(K, V)>);

impl<K: serde::Serialize, V: serde::Serialize> serde::Serialize for FakeMap<K, V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_map(self.0.iter().map(|(a, b)| (a, b)))
    }
}

macro_rules! map_impl {
    (
        use $u:path;
        $(#[$attr:meta])*
        $ty:ident <K $(: $kbound1:ident $(+ $kbound2:path)*)*, V $(, $typaram:ident : $bound:path)*>
    ) => {
        use $u;
        $(#[$attr])*
        impl<K, V $(, $typaram)*> Schema for $ty<K, V $(, $typaram)*>
        where
            K: Schema $(+ $kbound1 $(+ $kbound2)*)*,
            V: Schema,
            $($typaram: $bound,)*
        {
            type Example = FakeMap<<K as Schema>::Example, <V as Schema>::Example>;
            type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;

            #[inline]
            fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
            where
                B: SchemaBuilder<Self::Examples>,
            {
                let is_human_readable = schema_builder.is_human_readable();
                let mut map_schema_builder = schema_builder.describe_map(
                    None,
                    None,
                    || {
                        Ok([
                            FakeMap(Vec::new()),
                            FakeMap(
                                <K as SchemaExamples>::examples(is_human_readable)?
                                    .zip(<V as SchemaExamples>::examples(is_human_readable)?)
                                    .collect(),
                            ),
                            FakeMap(
                                <K as SchemaExamples>::examples(is_human_readable)?
                                    .zip(<V as SchemaExamples>::examples(is_human_readable)?)
                                    .collect(),
                            ),
                        ])
                    },
                    false,
                )?;
                map_schema_builder.collect_additional_elements(
                    K::describe,
                    None,
                    false,
                    V::describe,
                )?;
                map_schema_builder.end()
            }
        }
    }
}

map_impl! {
    use std::collections::BTreeMap;

    BTreeMap<K: Ord, V>
}

map_impl! {
    use std::collections::HashMap;

    HashMap<K: Eq + std::hash::Hash, V, H: std::hash::BuildHasher>
}
