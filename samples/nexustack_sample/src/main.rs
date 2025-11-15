/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use std::str::FromStr as _;

use nexustack::{
    Application, ApplicationBuilder, application_builder,
    cron::{
        CronApplicationBuilder as _, CronResult, CronRunner as _, cron, cron_jobs,
        schedule::Schedule,
    },
    inject::{InjectionResult, ServiceProvider, injectable},
};
use tracing::instrument;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("RUST_LOG", "info");
    }

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let app = application_builder()
        .configure_services(|services| {
            services
                .add_value(CronConfig {
                    // Can be loaded from env / cli args / config files
                    schedule: Schedule::from_str("0 * * * * *").unwrap(), // Every minute
                })
                .add_scoped::<MyService>();
        })
        .add_cron_with_default_clock()
        .configure_cron(cron_jobs![
            remove_expired_sessions_cron_job,
            some_other_cron_job,
        ])
        .build()?;

    app.run().await?;

    Ok(())
}

#[derive(Clone, Debug)]
#[injectable]
struct CronConfig {
    schedule: Schedule,
}

#[derive(Clone, Debug)]
#[injectable]
struct MyService;

#[cron(schedule = "*/30 * * * * *")] // Every 30 seconds
#[instrument]
#[allow(clippy::used_underscore_binding)]
async fn remove_expired_sessions_cron_job(#[cron::service] _service: MyService) -> CronResult {
    tracing::info!("remove_expired_sessions_cron_job");
    Ok(())
}

#[cron(schedule_with = "cron_schedule")]
#[instrument]
#[allow(clippy::used_underscore_binding)]
async fn some_other_cron_job(#[cron::service] _config: CronConfig) -> CronResult {
    tracing::info!("some_other_cron_job");
    Ok(())
}

#[allow(clippy::unused_async)]
async fn cron_schedule(service_provider: ServiceProvider) -> InjectionResult<Schedule> {
    Ok(service_provider.resolve::<CronConfig>()?.schedule)
}
