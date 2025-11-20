/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::cron::{CronError, CronResult};
use chrono::{DateTime, TimeZone, Utc};
use std::pin::Pin;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

/// A trait that abstracts access to the current date and time.
///
/// It is providing delay functionality. This is useful for swapping out
/// time-related behavior, such as in testing scenarios, within the cron subsystem.
/// Implementations operate in a configurable [`chrono::TimeZone`] via the associated
/// [`Self::TimeZone`] type.
pub trait CronClock {
    /// The associated time zone type for the clock.
    ///
    /// This type is used to represent the time zone in which the clock operates.
    type TimeZone: TimeZone;

    /// The associated future type returned by the `delay_until` method.
    ///
    /// This future resolves to a `CronResult` containing the actual date and time when the delay ended,
    /// or an error if the delay was canceled.
    type DelayUntilFuture<'a>: Future<Output = CronResult<DateTime<Self::TimeZone>>> + Send + 'a
    where
        Self: 'a;

    /// Returns the time zone in which the clock operates.
    ///
    /// # Returns
    ///
    /// The time zone associated with the clock.
    fn timezone(&self) -> Self::TimeZone;

    /// Returns the current date and time in the clock's time zone.
    fn now(&self) -> DateTime<Self::TimeZone>;

    /// Delays execution until the specified `date_time` is reached or the cancellation token is triggered.
    ///
    /// # Paramaters
    ///
    /// - `date_time` - The target date and time to delay until.
    /// - `cancellation_token` - A token that can be used to cancel the delay.
    ///
    /// # Returns
    ///
    /// A `CronResult` containing the actual date and time when the delay ended, or an error if the delay was canceled.
    ///
    /// # Errors
    ///
    /// This function returns a `CronError::Canceled` if the cancellation token is triggered
    /// before the specified `date_time` is reached.
    fn delay_until(
        &self,
        date_time: DateTime<Self::TimeZone>,
        cancellation_token: CancellationToken,
    ) -> Self::DelayUntilFuture<'_>;
}

/// The default implementation of the `CronClock` trait backed by the system clock.
///
/// The time zone is configurable via the `Timezone` type parameter and defaults to [`Utc`].
/// For example, `DefaultCronClock<Utc>` yields UTC-based timestamps, while
/// `DefaultCronClock<chrono::Local>` yields local-time timestamps.
#[derive(Clone, Debug)]
pub struct DefaultCronClock<Timezone: chrono::TimeZone = Utc>(Timezone);

impl Default for DefaultCronClock<Utc> {
    fn default() -> Self {
        Self(Utc)
    }
}

// TODO: Implement custom future that does not need to be boxed
async fn delay_until_impl<Timezone>(
    clock: &'_ DefaultCronClock<Timezone>,
    date_time: DateTime<Timezone>,
    cancellation_token: CancellationToken,
) -> CronResult<DateTime<Timezone>>
where
    Timezone: chrono::TimeZone + Send + Sync,
    <Timezone as TimeZone>::Offset: Send,
{
    loop {
        let now = clock.now();
        let duration = (date_time.clone() - &now).to_std().unwrap_or_default();

        if duration.is_zero() {
            return Ok(now);
        }

        tokio::select! {
            () = sleep(duration) => { }
            () = cancellation_token.cancelled() => {
                return Err(CronError::Canceled);
            }
        }
    }
}

impl<Timezone> CronClock for DefaultCronClock<Timezone>
where
    Timezone: chrono::TimeZone + Send + Sync,
    <Timezone as TimeZone>::Offset: Send,
{
    type TimeZone = Timezone;
    type DelayUntilFuture<'a>
        = Pin<Box<dyn Future<Output = CronResult<DateTime<Timezone>>> + Send + 'a>>
    where
        Timezone: 'a;

    fn timezone(&self) -> Self::TimeZone {
        self.0.clone()
    }

    /// Returns the current date and time in the clock's time zone using the system clock.
    fn now(&self) -> DateTime<Self::TimeZone> {
        self.0.from_utc_datetime(&Utc::now().naive_utc())
    }

    /// Delays execution until the specified `date_time` is reached or the cancellation token is triggered.
    ///
    /// # Paramaters
    ///
    /// - `date_time` - The target date and time to delay until.
    /// - `cancellation_token` - A token that can be used to cancel the delay.
    ///
    /// # Returns
    ///
    /// A `CronClockFuture` that resolves to a `CronResult` containing the actual date and time when the delay ended,
    /// or an error if the delay was canceled.
    fn delay_until(
        &self,
        date_time: DateTime<Self::TimeZone>,
        cancellation_token: CancellationToken,
    ) -> Self::DelayUntilFuture<'_> {
        Box::pin(delay_until_impl(self, date_time, cancellation_token))
    }
}
