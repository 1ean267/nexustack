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
        builder::{IntoSchemaBuilder, SchemaBuilder},
    },
};

macro_rules! deref_impl {
    (
        $(#[$attr:meta])*
        <$($desc:tt)+
    ) => {
        $(#[$attr])*
        impl <$($desc)+ {
            type Example = <T as Schema>::Example;
            type Examples = <T as Schema>::Examples;

            #[inline]
            fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
                where B: SchemaBuilder<Self::Examples>,
            {
                <T as Schema>::describe(schema_builder)
            }
        }
    };
}

deref_impl! {
    <'a, T> Schema for &'a T where T: ?Sized + Schema
}

deref_impl! {
    <'a, T> Schema for &'a mut T where T: ?Sized + Schema
}

deref_impl! {
    <T> Schema for Box<T> where T: ?Sized + Schema
}

deref_impl! {
    <T> Schema for std::rc::Rc<T> where T: ?Sized + Schema
}

deref_impl! {
    <T> Schema for std::sync::Arc<T> where T: ?Sized + Schema
}

deref_impl! {
    <'a, T> Schema for std::borrow::Cow<'a, T> where T: ?Sized + Schema + ToOwned
}

impl<T> Schema for std::rc::Weak<T>
where
    T: ?Sized + Schema,
{
    type Example = Option<<T as Schema>::Example>;
    type Examples = std::iter::Chain<
        std::iter::Map<<T as Schema>::Examples, fn(<T as Schema>::Example) -> Self::Example>,
        std::iter::Once<Self::Example>,
    >;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        let is_human_readable = schema_builder.is_human_readable();
        <T as Schema>::describe(
            schema_builder
                .describe_option(
                    None,
                    || {
                        Ok(<T as SchemaExamples>::examples(is_human_readable)?
                            .map(Some as _)
                            .chain(std::iter::once(None)))
                    },
                    false,
                )?
                .into_schema_builder(),
        )
    }
}

impl<T> Schema for std::sync::Weak<T>
where
    T: ?Sized + Schema,
{
    type Example = Option<<T as Schema>::Example>;
    type Examples = std::iter::Chain<
        std::iter::Map<<T as Schema>::Examples, fn(<T as Schema>::Example) -> Self::Example>,
        std::iter::Once<Self::Example>,
    >;

    #[inline]
    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: SchemaBuilder<Self::Examples>,
    {
        let is_human_readable = schema_builder.is_human_readable();
        <T as Schema>::describe(
            schema_builder
                .describe_option(
                    None,
                    || {
                        Ok(<T as SchemaExamples>::examples(is_human_readable)?
                            .map(Some as _)
                            .chain(std::iter::once(None)))
                    },
                    false,
                )?
                .into_schema_builder(),
        )
    }
}
