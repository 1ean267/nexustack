/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

mod core {
    pub use std::*;
}

pub use self::core::borrow::Cow;
pub use self::core::default::Default;
pub use self::core::iter::Chain;
pub use self::core::iter::Iterator;
pub use self::core::iter::Map;
pub use self::core::iter::Once;
pub use self::core::iter::Zip;
pub use self::core::iter::once;
pub use self::core::marker::PhantomData;
pub use self::core::marker::Send;
pub use self::core::option::Option::{self, None, Some};
pub use self::core::result::Result::{self, Err, Ok};

#[cfg(feature = "http")]
#[path = ""]
pub mod std {
    #[path = ""]
    pub mod net {
        pub use std::net::SocketAddr;
    }
}

#[path = ""]
pub mod utils {
    pub use crate::utils::AtomicOnceCell;
    pub use crate::utils::Optional;
}

#[cfg(feature = "cron")]
#[path = ""]
pub mod cron {
    pub use cron::Schedule;
}

#[cfg(feature = "http")]
#[path = ""]
pub mod axum {
    pub use axum::*;
}

#[cfg(feature = "http")]
#[path = ""]
pub mod axum_extra {
    pub use axum_extra::*;
}

#[cfg(feature = "axum-client-ip")]
#[path = ""]
pub mod axum_client_ip {
    pub use axum_client_ip::*;
}

#[cfg(feature = "http")]
#[path = ""]
pub mod serde {
    pub use serde::*;
}
