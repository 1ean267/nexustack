/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::BoxError;

/// An error representing a failure in cron job scheduling or execution
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum CronError {
    /// Raised when the operation was canceled
    #[error("Operation was canceled")]
    Canceled,
    /// Raised when the schedule could not be determined
    #[error("Failed to determine schedule")]
    ScheduleError(#[source] BoxError),

    /// Raised when the cron job fails to run
    #[error("Failed to run cron job")]
    RunError(#[source] BoxError),
}

/// A cron result representing the result of a cron job scheduling or execution
pub type CronResult<T = ()> = Result<T, CronError>;
