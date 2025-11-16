/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use nexustack::{
    ApplicationBuilder,
    cron::{CronApplicationBuilder as _, CronResult, CronRunner, cron, cron_jobs},
    module,
};

/// Extension trait to add the Notes module to the application builder.
#[module(features = "CronRunner")]
pub trait NotesModule {
    /// Adds the Notes module to the application builder.
    fn add_notes(self) -> impl ApplicationBuilder {
        self.configure_cron(cron_jobs![remove_outdated_notes_cron_job])
    }
}

/// A cron job that removes outdated notes.
///
/// This job runs every hour to clean up notes that are no longer relevant.
#[cron(schedule = "0 0 * * * *")]
#[allow(clippy::unused_async)]
async fn remove_outdated_notes_cron_job() -> CronResult {
    // TODO: Implement the logic to remove outdated notes
    Ok(())
}
