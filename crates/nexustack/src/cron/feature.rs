/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    ApplicationBuilder, ApplicationPart, ApplicationPartBuilder, Index,
    application::{Here, InHead, InTail, Node},
    cron::{CronClock, CronError, CronJob, CronResult, DefaultCronClock},
    inject::{ServiceProvider, ServiceScope},
};
use chrono::TimeZone;
use std::{borrow::Cow, fmt::Write as _, marker::PhantomData, time::Instant};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

/// A trait that extends the `ApplicationBuilder` to add support for configuring and running cron jobs.
///
/// This trait provides methods to integrate cron job functionality into the application, including
/// adding cron services and configuring cron jobs with specific schedules and execution logic.
pub trait CronApplicationBuilder: ApplicationBuilder {
    /// Adds cron services to the application using the specified clock implementation.
    ///
    /// This method allows you to add cron functionality to the application with a custom clock
    /// implementation. The clock is used to determine the current time and schedule the execution
    /// of cron jobs.
    ///
    /// # Type Parameters
    /// - `Clock`: The clock implementation to use. It must implement the [`CronClock`] trait.
    ///
    /// # Returns
    /// An updated application builder with the cron services added.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::{
    ///     application_builder,
    ///     Application as _,
    ///     ApplicationBuilder as _,
    ///     cron::{DefaultCronClock, CronApplicationBuilder as _},
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let app = application_builder()
    ///    .add_cron::<DefaultCronClock>()
    ///    .build()?;
    ///
    /// app.run().await?;
    ///
    /// #   Ok(())
    /// # }
    /// ```
    fn add_cron<Clock>(
        self,
    ) -> impl ApplicationBuilder<Chain = Node<CronApplicationPartBuilder<Clock>, Self::Chain>>
    where
        Clock: CronClock + 'static,
        Self: Sized;

    /// Adds cron services to the application using the default clock implementation.
    ///
    /// This method is a convenience function that adds cron functionality to the application
    /// using the [`DefaultCronClock`] implementation. The default clock is suitable for most
    /// use cases and provides a reliable way to schedule and execute cron jobs.
    ///
    /// # Returns
    /// An updated application builder with the cron services added.
    ///
    /// # Example
    /// ```rust
    /// use nexustack::{
    ///     application_builder,
    ///     Application as _,
    ///     ApplicationBuilder as _,
    ///     cron::CronApplicationBuilder as _,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let app = application_builder()
    ///     .add_cron_with_default_clock()
    ///     .build()?;
    ///
    /// app.run().await?;
    ///
    /// #   Ok(())
    /// # }
    /// ```
    fn add_cron_with_default_clock(
        self,
    ) -> impl ApplicationBuilder<Chain = Node<CronApplicationPartBuilder<DefaultCronClock>, Self::Chain>>
    where
        Self: Sized;

    /// Configures a cron job of the specified type.
    // TODO: This should be named add_cron_job (Which is not in sync with the other app features)
    fn configure_cron<I: Index, F>(
        self,
        configure: F,
    ) -> impl ApplicationBuilder<Chain = Self::Chain>
    where
        Self: Sized,
        Self::Chain: Cron<I>,
        F: FnOnce(&mut Self::Chain);
}

impl<B: ApplicationBuilder> CronApplicationBuilder for B {
    fn add_cron<Clock>(
        self,
    ) -> impl ApplicationBuilder<Chain = Node<CronApplicationPartBuilder<Clock>, Self::Chain>>
    where
        Clock: CronClock + 'static,
    {
        self.add_application_part_with_factory(|| CronApplicationPartBuilder {
            _clock: PhantomData,
            cron_job_names: String::new(),
            cron_task_factories: Vec::new(),
        })
    }

    fn add_cron_with_default_clock(
        self,
    ) -> impl ApplicationBuilder<Chain = Node<CronApplicationPartBuilder<DefaultCronClock>, Self::Chain>>
    where
        Self: Sized,
    {
        self.configure_services(|services| {
            services.add_value(DefaultCronClock::default());
        })
        .add_cron::<DefaultCronClock>()
    }

    fn configure_cron<I: Index, F>(
        self,
        configure: F,
    ) -> impl ApplicationBuilder<Chain = Self::Chain>
    where
        Self: Sized,
        Self::Chain: Cron<I>,
        F: FnOnce(&mut Self::Chain),
    {
        self.configure_chain(configure)
    }
}

/// A trait for configuring and managing cron jobs within an application.
///
/// This trait provides methods to add cron jobs to the application. It is typically implemented
/// by application parts or nodes in the application builder chain.
pub trait Cron<Index> {
    /// Adds a cron job of the specified type to the application.
    ///
    /// This method registers a cron job by its type, allowing it to be scheduled and executed
    /// as part of the application's cron system.
    ///
    /// # Type Parameters
    /// - `Job`: The type of the cron job to add. The type must implement the [`CronJob`] trait.
    ///
    /// # Returns
    /// A mutable reference to `Self`, enabling method chaining.
    fn add_cron_job<Job>(&mut self) -> &mut Self
    where
        Job: CronJob + 'static;
}

impl<Head, Tail, HeadIndex> Cron<InHead<HeadIndex>> for Node<Head, Tail>
where
    HeadIndex: Index,
    Head: Cron<HeadIndex>,
{
    fn add_cron_job<Job>(&mut self) -> &mut Self
    where
        Job: CronJob + 'static,
    {
        self.head.add_cron_job::<Job>();
        self
    }
}

impl<Head, Tail, TailIndex> Cron<InTail<TailIndex>> for Node<Head, Tail>
where
    TailIndex: Index,
    Tail: Cron<TailIndex>,
{
    fn add_cron_job<Job>(&mut self) -> &mut Self
    where
        Job: CronJob + 'static,
    {
        self.tail.add_cron_job::<Job>();
        self
    }
}

impl<Clock> Cron<Here> for CronApplicationPartBuilder<Clock>
where
    Clock: CronClock + Send + 'static,
    <<Clock as CronClock>::TimeZone as TimeZone>::Offset: Send,
{
    fn add_cron_job<Job>(&mut self) -> &mut Self
    where
        Job: CronJob + 'static,
    {
        self.cron_task_factories.push(Box::new(
            |service_provider: ServiceProvider, cancellation_token: CancellationToken| {
                cron_job_task::<Job, Clock>(service_provider, cancellation_token)
            },
        ));
        if self.cron_job_names.is_empty() {
            write!(self.cron_job_names, "{}", cron_job_name::<Job>())
                .expect("Failed to write cron job name");
        } else {
            write!(self.cron_job_names, "{}, ", cron_job_name::<Job>())
                .expect("Failed to write cron job name");
        }
        self
    }
}

fn cron_job_task<Job, Clock>(
    service_provider: ServiceProvider,
    cancellation_token: CancellationToken,
) -> JoinHandle<CronResult>
where
    Job: CronJob + 'static,
    Clock: CronClock + Send + 'static,
    <<Clock as CronClock>::TimeZone as TimeZone>::Offset: Send,
{
    tokio::spawn(run_cron_job::<Job, Clock>(
        service_provider,
        cancellation_token,
    ))
}

fn cron_job_name<Job>() -> Cow<'static, str>
where
    Job: CronJob,
{
    Job::name()
}

#[tracing::instrument(
    name = "cron_job.run",
    skip(service_provider, cancellation_token),
    fields(cron_job = cron_job_name::<Job>().to_string())
)]
async fn execute_job<Job, Clock>(
    service_provider: ServiceProvider,
    cancellation_token: CancellationToken,
) -> CronResult<()>
where
    Job: CronJob,
    Clock: CronClock + Send + 'static,
    <<Clock as CronClock>::TimeZone as TimeZone>::Offset: Send,
{
    let start = Instant::now();
    tracing::trace!("Running cron job");

    let service_scope = service_provider
        .resolve::<ServiceScope>()
        .map_err(|err| CronError::RunError(err.into()))
        .inspect_err(|err| tracing::error!(%err, "Failed to resolve service scope"))?;
    let scoped_service_provider = service_scope.service_provider();

    cancellation_token
        .run_until_cancelled_owned(Job::run(scoped_service_provider.clone()))
        .await
        .ok_or_else(|| CronError::Canceled)
        .flatten()
        .inspect_err(|err| {
            if matches!(err, CronError::Canceled) {
                tracing::debug!(took_ms = start.elapsed().as_millis(), "Cron job execution was canceled");
            } else {
                tracing::error!(took_ms = start.elapsed().as_millis(), %err, "Error during cron job execution");
            }
        })
        .inspect(|()| {
            tracing::debug!(took_ms = start.elapsed().as_millis(),"Cron job executed successfully");
        })
}

#[tracing::instrument(
    name = "cron_job.task",
    skip(service_provider, cancellation_token),
    fields(cron_job = cron_job_name::<Job>().to_string())
)]
async fn run_cron_job<Job, Clock>(
    service_provider: ServiceProvider,
    cancellation_token: CancellationToken,
) -> CronResult<()>
where
    Job: CronJob,
    Clock: CronClock + Send + 'static,
    <<Clock as CronClock>::TimeZone as TimeZone>::Offset: Send,
{
    let start = Instant::now();
    tracing::debug!("Starting cron job task",);

    let clock = service_provider
        .resolve::<Clock>()
        .map_err(|err| CronError::RunError(err.into()))
        .inspect_err(|err| tracing::error!(%err, "Failed to resolve clock"))?;

    let schedule = cancellation_token
        .clone()
        .run_until_cancelled_owned(Job::schedule(service_provider.clone()))
        .await
        .ok_or_else(|| CronError::Canceled)
        .flatten()
        .inspect_err(|err| {
            if matches!(err, CronError::Canceled) {
                tracing::debug!("Cron job task was canceled");
            } else {
                tracing::error!(%err, "Failed to resolve schedule");
            }
        })?;

    let now = clock.now();
    let upcoming_iter = schedule.after(&now);

    for upcoming in upcoming_iter {
        tracing::trace!(
            next_run = %upcoming.to_rfc3339(),
            "Next scheduled run for cron job",
        );

        clock
            .delay_until(upcoming, cancellation_token.clone())
            .await
            .inspect_err(|err| {
                if matches!(err, CronError::Canceled) {
                    tracing::debug!("Cron job task was canceled");
                } else {
                    tracing::error!(%err, "Failed to delay until next schedule");
                }
            })?;

        execute_job::<Job, Clock>(service_provider.clone(), cancellation_token.clone()).await?;
    }

    tracing::debug!(
        took_ms = start.elapsed().as_millis(),
        "Cron job task completed successfully"
    );

    Ok(())
}

type CronTaskFactory =
    Box<dyn FnOnce(ServiceProvider, CancellationToken) -> JoinHandle<CronResult> + Send + Sync>;

pub struct CronApplicationPartBuilder<Clock> {
    _clock: PhantomData<fn() -> Clock>,
    cron_job_names: String,
    cron_task_factories: Vec<CronTaskFactory>,
}

impl<Clock> ApplicationPartBuilder for CronApplicationPartBuilder<Clock> {
    type ApplicationPart = CronApplicationPart;

    fn build(
        self,
        service_provider: ServiceProvider,
    ) -> crate::inject::ConstructionResult<Self::ApplicationPart> {
        Ok(CronApplicationPart {
            cron_job_names: self.cron_job_names,
            cron_task_factories: self.cron_task_factories,
            service_provider,
        })
    }
}

pub struct CronApplicationPart {
    cron_job_names: String,
    cron_task_factories: Vec<CronTaskFactory>,
    service_provider: ServiceProvider,
}

impl ApplicationPart for CronApplicationPart {
    type Error = CronError;

    #[tracing::instrument(
        name = "schedule_cron_jobs",
        skip(self, cancellation_token),
        fields(cron_jobs = self.cron_job_names)
    )]
    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<(), Self::Error> {
        tracing::debug!("Executing run phase for cron application part");
        let start = Instant::now();

        let cron_tasks: Vec<JoinHandle<Result<(), CronError>>> = self
            .cron_task_factories
            .drain(..)
            .map(|factory| factory(self.service_provider.clone(), cancellation_token.clone()))
            .collect::<Vec<_>>();

        for cron_task in cron_tasks {
            let cron_task_result = cron_task
                .await
                .map_err(|err| CronError::RunError(err.into()))
                .flatten()
                .inspect(|()| {
                    tracing::debug!(
                        took_ms = start.elapsed().as_millis(),
                        "Executed cron task successfully"
                    );
                })
                .inspect_err(|err| {
                    if matches!(err, CronError::Canceled) {
                        tracing::debug!(
                            took_ms = start.elapsed().as_millis(),
                            "Cron task was canceled"
                        );
                    } else {
                        tracing::error!(
                            took_ms = start.elapsed().as_millis(),
                            %err,
                            "Error during cron task execution"
                        );
                    }
                });

            if let Err(err) = cron_task_result
                && !matches!(err, CronError::Canceled)
            {
                return Err(err);
            }
        }

        tracing::debug!(
            took_ms = start.elapsed().as_millis(),
            "Completed run phase for cron application part"
        );

        Ok(())
    }
}

const _: () = {
    const fn ok<Head>()
    where
        Head: ApplicationPartBuilder,
        <Head as ApplicationPartBuilder>::ApplicationPart: Send + Sync,
        <<Head as ApplicationPartBuilder>::ApplicationPart as ApplicationPart>::Error:
            std::fmt::Display + Send,
    {
    }

    ok::<CronApplicationPartBuilder<DefaultCronClock>>();
};
