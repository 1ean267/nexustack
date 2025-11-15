/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use std::borrow::Cow;

use crate::inject::ServiceProvider;
use cron::Schedule;

mod clock;
mod error;
mod feature;

pub use clock::{CronClock, DefaultCronClock};
pub use error::{CronError, CronResult};
pub use feature::{CronApplicationBuilder, CronRunner};

pub use nexustack_macros::cron_jobs;

#[cfg(feature = "derive")]
pub use nexustack_macros::cron;

#[doc = include_str!("../../../nexustack_macros_impl/src/cron/expand/CRON.md")]
const fn _check_cron_doc_test() {}

/// A module that re-exports the [`mod@cron`] crate.
///
/// This module provides access to all the types and functionality of the
/// [`cron`](https://docs.rs/cron) crate. For detailed documentation, refer to
/// the [`cron` crate documentation](https://docs.rs/cron).
#[path = ""]
pub mod schedule {
    pub use ::cron::*;
}

/// A trait representing a cron job.
///
/// Implement this trait to define the schedule
/// and execution logic for a cron job.
pub trait CronJob {
    /// Defines the schedule for the cron job.
    ///
    /// # Parameters
    /// - `service_provider`: A scoped [`ServiceProvider`] instance that provides access to
    ///   the application's services and dependencies.
    ///
    /// # Returns
    /// A future that resolves to a [`CronResult`] containing the [`Schedule`] for the cron job.
    fn schedule(
        service_provider: ServiceProvider,
    ) -> impl Future<Output = CronResult<Schedule>> + Send;

    /// Executes the logic for the cron job.
    ///
    /// # Parameters
    /// - `service_provider`: A scoped [`ServiceProvider`] instance that provides access to
    ///   the application's services and dependencies.
    ///
    /// # Returns
    /// A future that resolves to a [`CronResult`] indicating the success or failure of the job.
    fn run(service_provider: ServiceProvider) -> impl Future<Output = CronResult> + Send;

    /// Returns the name of this cron job as a string.
    ///
    /// # Returns
    /// A [`Cow<str>`] containing the type name of the cron job. By default, this is the Rust type name, but can be overridden for custom display.
    #[must_use]
    fn name() -> Cow<'static, str> {
        Cow::Borrowed(std::any::type_name::<Self>())
    }
}
