/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use nexustack_inject_macros_impl::expand_injectable;
use quote::quote;
use rust_format::{Formatter, RustFmt};

static EXPECTED: &str = stringify! {
    struct TupleStruct(pub Dependency1, Dependency2);

    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
        non_camel_case_types,
        deprecated
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate nexustack_inject as _nexustack_inject;
        #[automatically_derived]
        impl _nexustack_inject::FromInjector for TupleStruct {
            fn from_injector(
                injector: &_nexustack_inject::Injector,
            ) -> _nexustack_inject::ConstructionResult<Self> {
                let arg_0 = injector.resolve::<Dependency1>()?;
                let arg_1 = injector.resolve::<Dependency2>()?;
                _nexustack_inject::IntoConstructionResult::into_construction_result(Self (arg_0, arg_1))
            }
        }
        #[automatically_derived]
        impl _nexustack_inject::Injectable for TupleStruct {}
        #[automatically_derived]
        impl _nexustack_inject::IntoConstructionResult for TupleStruct {
            type Service = TupleStruct;
            fn into_construction_result(self) -> _nexustack_inject::ConstructionResult<Self::Service> {
                _nexustack_inject::ConstructionResult::Ok(self)
            }
        }
    };
};

#[test]
fn test_tuple_struct() {
    let attr = quote! {};
    let input = quote! {
        // #[injectable]
        struct TupleStruct(pub Dependency1, Dependency2);
    };

    let expected = RustFmt::default()
        .format_str(EXPECTED)
        .unwrap()
        .replace("\r\n", "\n");

    let actual = RustFmt::default()
        .format_tokens(expand_injectable(attr, input).unwrap())
        .unwrap()
        .replace("\r\n", "\n");

    assert_eq!(actual, expected);
}
