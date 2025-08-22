/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use nexustack_macros_impl::inject::expand_injectable;
use quote::quote;
use rust_format::{Formatter, RustFmt};

static EXPECTED: &str = stringify! {
    impl Basic {
        pub fn new() -> Self {}
    }
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
        extern crate nexustack as _nexustack;
        #[automatically_derived]
        impl _nexustack::inject::FromInjector for Basic {
            fn from_injector(
                injector: &_nexustack::inject::Injector,
            ) -> _nexustack::inject::ConstructionResult<Self> {
                _nexustack::inject::IntoConstructionResult::into_construction_result(Self::new())
            }
        }
        #[automatically_derived]
        impl _nexustack::inject::Injectable for Basic {}
        #[automatically_derived]
        impl _nexustack::inject::IntoConstructionResult for Basic {
            type Service = Basic;
            fn into_construction_result(self) -> _nexustack::inject::ConstructionResult<Self::Service> {
                _nexustack::inject::ConstructionResult::Ok(self)
            }
        }
    };
};

#[test]
fn test_no_deps() {
    let attr = quote! {};
    let input = quote! {
        // #[injectable]
        impl Basic {
            pub fn new() -> Self {}
        }
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
