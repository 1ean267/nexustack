/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    application::{configurable::Configurable, instrumentation::WithInstrumentation, node::Node},
    inject::{ConstructionResult, ServiceCollection, ServiceProvider},
};
use ::either::Either;
use std::{borrow::Cow, time::Instant};
use tokio_util::sync::CancellationToken;

mod configurable;
mod either;
mod empty;
mod instrumentation;
mod node;

/// Builder trait for constructing application parts.
///
/// Implementors of this trait are responsible for producing an application part instance from a service provider.
/// This is typically used in the application builder to allow flexible composition of application parts.
///
/// See [`ApplicationPart`] for lifecycle hooks and error handling.
pub trait ApplicationPartBuilder {
    /// The type of application part produced by this builder.
    ///
    /// This must implement [`ApplicationPart`].
    type ApplicationPart: ApplicationPart;

    /// Builds an application part instance using the provided service provider.
    ///
    /// # Arguments
    /// * `service_provider` - The application's service provider, used to resolve dependencies for the application part.
    ///
    /// # Returns
    /// A [`ConstructionResult`] containing the constructed [`Self::ApplicationPart`] on success, or an error if construction fails.
    ///
    /// # Errors
    /// Returns an error if the application part or any of its dependencies cannot be constructed from the service provider.
    fn build(self, service_provider: ServiceProvider) -> ConstructionResult<Self::ApplicationPart>;
}

/// Trait representing a part of an application lifecycle.
///
/// Implementors define async hooks for startup, running, and shutdown phases.
/// All hooks should return immediately if cancelled via the provided `CancellationToken`.
/// Hooks are run in parallel for all parts.
pub trait ApplicationPart {
    /// The error type returned by this application part's hooks.
    type Error;

    /// Returns the name of this application part as a string.
    ///
    /// # Returns
    /// A [`Cow<str>`] containing the type name of the application part. By default, this is the Rust type name, but can be overridden for custom display.
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed(std::any::type_name::<Self>())
    }

    /// Called before application startup for this part.
    ///
    /// # Arguments
    /// * `cancellation_token` - A token that can be used to cancel startup. If cancelled, this function should return immediately and not error.
    ///
    /// # Returns
    /// A future that resolves to `Result<(), Self::Error>`.
    fn before_startup(
        &self,
        cancellation_token: CancellationToken,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_ {
        let _ = cancellation_token;
        async move { Ok(()) }
    }

    /// Runs the main logic for this application part.
    ///
    /// # Arguments
    /// * `cancellation_token` - A token that can be used to cancel execution. If cancelled, this function should return immediately and not error.
    ///
    /// # Returns
    /// A future that resolves to `Result<(), Self::Error>`.
    fn run(
        &self,
        cancellation_token: CancellationToken,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_;

    /// Called before application shutdown for this part.
    ///
    /// # Arguments
    /// * `cancellation_token` - A token that can be used to cancel execution. If cancelled, this function should return immediately and not error.
    ///
    /// # Returns
    /// A future that resolves to `Result<(), Self::Error>`. Should not block for too long.
    fn before_shutdown(
        &self,
        cancellation_token: CancellationToken,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_ {
        let _ = cancellation_token;
        async move { Ok(()) }
    }
}

/// Creates a new [`ApplicationBuilder`] for constructing an application.
///
/// # Returns
/// An [`ApplicationBuilder`] instance for chaining application part and service configuration.
#[must_use]
pub fn application_builder() -> impl ApplicationBuilder {
    ApplicationBuilderConcrete::default()
}

/// Trait for building and configuring an application from parts and services.
///
/// This trait provides a fluent API for composing application parts and configuring dependency injection
/// before producing a final [`Application`] instance. Implementors are typically not used directly; instead,
/// use the [`application_builder`] function to obtain a builder.
pub trait ApplicationBuilder {
    /// The type of the application part builder chain managed by this builder.
    type ApplicationPartBuilder: ApplicationPartBuilder;

    /// Adds an application part to the builder using a default-constructed builder and a configuration closure.
    ///
    /// # Type Parameters
    /// - `B`: The application part builder type to add. Must implement [`ApplicationPartBuilder`] and [`Default`].
    /// - `C`: The configuration closure type.
    ///
    /// # Arguments
    /// * `configuration` - A closure that receives a mutable reference to the part builder and configures it.
    ///
    /// # Returns
    /// A new builder with the application part added.
    #[must_use]
    fn add_application_part<B, C>(self, configuration: C) -> impl ApplicationBuilder
    where
        Self: Sized,
        B: ApplicationPartBuilder + Default + 'static,
        <B as ApplicationPartBuilder>::ApplicationPart: Send + Sync,
        <<B as ApplicationPartBuilder>::ApplicationPart as ApplicationPart>::Error:
            std::fmt::Display + Send,
        C: FnOnce(&mut B),
    {
        self.add_application_part_with_factory(B::default, configuration)
    }

    /// Adds an application part to the builder using a custom factory and configuration closure.
    ///
    /// # Type Parameters
    /// - `B`: The application part builder type to add. Must implement [`ApplicationPartBuilder`].
    /// - `F`: The factory closure type.
    /// - `C`: The configuration closure type.
    ///
    /// # Arguments
    /// * `factory` - A closure that produces a new instance of the part builder.
    /// * `configure` - A closure that receives a mutable reference to the part builder and configures it.
    ///
    /// # Returns
    /// A new builder with the application part added.
    #[must_use]
    fn add_application_part_with_factory<B, F, C>(
        self,
        factory: F,
        configure: C,
    ) -> impl ApplicationBuilder
    where
        B: ApplicationPartBuilder + 'static,
        <B as ApplicationPartBuilder>::ApplicationPart: Send + Sync,
        <<B as ApplicationPartBuilder>::ApplicationPart as ApplicationPart>::Error:
            std::fmt::Display + Send,
        F: FnOnce() -> B,
        C: FnOnce(&mut B);

    /// Configures the service collection for dependency injection.
    ///
    /// # Type Parameters
    /// - `F`: The configuration closure type.
    ///
    /// # Arguments
    /// * `configure` - A closure that receives a mutable reference to the [`ServiceCollection`] and can register services.
    ///
    /// # Returns
    /// The builder instance, allowing further chaining.
    #[must_use]
    fn configure_services<F>(self, configure: F) -> impl ApplicationBuilder
    where
        F: FnOnce(&mut ServiceCollection);

    /// Builds the final [`Application`] instance from the collected parts and configured services.
    ///
    /// # Returns
    /// A [`ConstructionResult`] containing the constructed [`Application`] on success, or an error if construction fails.
    ///
    /// # Errors
    /// Returns an error if any application part or its dependencies cannot be constructed, or if service configuration fails.
    fn build(self) -> ConstructionResult<impl Application>;
}

/// Concrete implementation of [`ApplicationBuilder`].
///
/// # Type Parameters
/// - `B`: The type of the application part builder chain.
struct ApplicationBuilderConcrete<B> {
    /// The collection of services to be injected into application parts.
    service_collection: ServiceCollection,
    /// The builder chain for application parts.
    builder: B,
}

impl<A> ApplicationBuilder for ApplicationBuilderConcrete<A>
where
    A: ApplicationPartBuilder + Configurable<'static>,
    <A as ApplicationPartBuilder>::ApplicationPart: Sync,
    <<A as ApplicationPartBuilder>::ApplicationPart as ApplicationPart>::Error:
        std::fmt::Display + Send,
{
    type ApplicationPartBuilder = A;

    fn add_application_part_with_factory<B, F, C>(
        mut self,
        factory: F,
        configure: C,
    ) -> impl ApplicationBuilder
    where
        B: ApplicationPartBuilder + 'static,
        <B as ApplicationPartBuilder>::ApplicationPart: Send + Sync,
        <<B as ApplicationPartBuilder>::ApplicationPart as ApplicationPart>::Error:
            std::fmt::Display + Send,
        F: FnOnce() -> B,
        C: FnOnce(&mut B),
    {
        let configure = |builder: &mut WithInstrumentation<B>| configure(&mut builder.0);
        let apply_result = self.builder.configure(configure);

        if let Err(configure) = apply_result {
            let mut application_part_builder = WithInstrumentation(factory());
            configure(&mut application_part_builder);
            ApplicationBuilderConcrete {
                service_collection: self.service_collection,
                builder: Either::Right(Node {
                    head: application_part_builder,
                    tail: self.builder,
                }),
            }
        } else {
            ApplicationBuilderConcrete {
                service_collection: self.service_collection,
                builder: Either::Left(self.builder),
            }
        }
    }

    fn configure_services<F>(mut self, configure: F) -> impl ApplicationBuilder
    where
        F: FnOnce(&mut ServiceCollection),
    {
        configure(&mut self.service_collection);
        self
    }

    fn build(self) -> ConstructionResult<impl Application> {
        let service_provider = self.service_collection.build();

        Ok(ApplicationConcrete {
            service_provider: service_provider.clone(),
            app_part: self.builder.build(service_provider)?,
        })
    }
}

impl Default for ApplicationBuilderConcrete<()> {
    fn default() -> Self {
        Self {
            service_collection: ServiceCollection::new(),
            builder: (),
        }
    }
}

/// Trait representing a running application composed of one or more application parts and a configured service provider.
///
/// This trait provides methods to run the application lifecycle, including startup, execution, and shutdown phases.
pub trait Application {
    /// The error type returned by this application during its lifecycle phases.
    type Error;

    /// Runs the application, executing all lifecycle phases (startup, run, shutdown) for the collected application parts.
    ///
    /// This method creates a new cancellation token and listens for shutdown signals (e.g., Ctrl+C).
    ///
    /// # Returns
    /// * `Result<(), Self::Error>` - Returns `Ok(())` if the application runs and shuts down successfully, or an error if any part fails.
    ///
    /// # Errors
    /// Returns an error if any application part fails during startup, run, or shutdown.
    fn run(self) -> impl Future<Output = Result<(), Self::Error>> + Send
    where
        Self: Send + Sized,
    {
        let cancellation_token = CancellationToken::new();
        async move { self.run_with_cancellation_token(cancellation_token).await }
    }

    /// Runs the application, executing all lifecycle phases (startup, run, shutdown) for the collected application parts.
    ///
    /// This method listens for shutdown signals (e.g., Ctrl+C) and cancels the application when either the specified cancellation token
    /// is triggered or a shutdown signal is received. The specified cancellation token is not cancelled by this method.
    ///
    /// # Arguments
    /// * `cancellation_token` - A [`CancellationToken`] used to control graceful shutdown. The application will shut down when this token is cancelled or a shutdown signal is received.
    ///
    /// # Returns
    /// * `Result<(), Self::Error>` - Returns `Ok(())` if the application runs and shuts down successfully, or an error if any part fails.
    ///
    /// # Errors
    /// Returns an error if any application part fails during startup, run, or shutdown.
    fn run_with_cancellation_token(
        self,
        cancellation_token: CancellationToken,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send
    where
        Self: Send + Sized;

    /// Returns a reference to the application's [`ServiceProvider`].
    ///
    /// # Returns
    /// * `&ServiceProvider` - The service provider used to resolve dependencies for application parts.
    fn service_provider(&self) -> &ServiceProvider;
}

/// Represents a running application composed of one or more application parts and a configured service provider.
///
/// This type is created by [`ApplicationBuilder::build`] and provides methods to run the application lifecycle, including startup, execution, and shutdown.
///
/// # Type Parameters
/// - `T`: The type representing the collected application parts. This is usually built up by chaining calls to [`ApplicationBuilder::add_application_part`].
struct ApplicationConcrete<T> {
    /// The application's service provider, used to resolve dependencies for application parts.
    service_provider: ServiceProvider,
    /// The collected application parts to be run by the application.
    app_part: T,
}

impl<T> Application for ApplicationConcrete<T>
where
    Self: Sync,
    T: ApplicationPart,
    <T as ApplicationPart>::Error: std::fmt::Display,
{
    type Error = <T as ApplicationPart>::Error;

    async fn run_with_cancellation_token(
        self,
        cancellation_token: CancellationToken,
    ) -> Result<(), Self::Error> {
        let start_ts = Instant::now();
        let cancellation_token = cancellation_token.child_token();

        // Spawn signal listener for graceful shutdown
        tokio::task::spawn({
            let cancellation_token = cancellation_token.clone();

            async move {
                let _ = tokio::signal::ctrl_c().await.inspect_err(|err| {
                    tracing::error!(%err, "Failed to listen for shutdown signal");
                });

                tracing::info!("Shutdown signal received. Shutting down application.");
                cancellation_token.cancel();
            }
        });

        self.before_startup(cancellation_token.clone()).await?;
        self.run_core(cancellation_token).await?;

        // TODO: Implement a timeout for graceful shutdown
        let forced_shutdown = CancellationToken::new();

        self.before_shutdown(forced_shutdown).await?;

        tracing::info!(
            took_ms = start_ts.elapsed().as_millis(),
            "Application shutting down after successful run"
        );

        Ok(())
    }

    fn service_provider(&self) -> &ServiceProvider {
        &self.service_provider
    }
}

impl<T> ApplicationConcrete<T>
where
    Self: Sync,
    T: ApplicationPart,
    <T as ApplicationPart>::Error: std::fmt::Display,
{
    /// Executes the `before_startup` lifecycle phase for all collected application parts.
    ///
    /// # Parameters
    /// - `cancellation_token`: A [`CancellationToken`] used to control graceful shutdown.
    ///
    /// # Returns
    /// - `Result<(), T::Error>`: Returns `Ok(())` if all parts start successfully, or an error from any part.
    #[tracing::instrument(
        name = "application.before_startup",
        skip(self, cancellation_token),
        fields(application_parts = self.app_part.name().to_string())
    )]
    async fn before_startup(&self, cancellation_token: CancellationToken) -> Result<(), T::Error> {
        tracing::debug!("Executing before_startup phase");
        let start = Instant::now();

        self.app_part
            .before_startup(cancellation_token)
            .await
            .inspect(|()| {
            tracing::debug!(
                took_ms = start.elapsed().as_millis(),
                "Executed before_startup phase sucessfully"
            );
        })
        .inspect_err(
            |err| tracing::error!(took_ms = start.elapsed().as_millis(), %err, "Error during before_startup phase"),
        )
    }

    /// Executes the `run` lifecycle phase for all collected application parts.
    ///
    /// # Parameters
    /// - `cancellation_token`: A [`CancellationToken`] used to control graceful shutdown.
    ///
    /// # Returns
    /// - `Result<(), T::Error>`: Returns `Ok(())` if all parts run successfully, or an error from any part.
    #[tracing::instrument(
        name = "application.run",
        skip(self, cancellation_token),
        fields(application_parts = self.app_part.name().to_string())
    )]
    async fn run_core(&self, cancellation_token: CancellationToken) -> Result<(), T::Error> {
        tracing::debug!("Executing run phase");
        let start = Instant::now();

        self.app_part
            .run(cancellation_token)
            .await
            .inspect(|()| {
            tracing::debug!(
                took_ms = start.elapsed().as_millis(),
                "Executed run phase sucessfully"
            );
        })
        .inspect_err(
            |err| tracing::error!(took_ms = start.elapsed().as_millis(), %err, "Error during run phase"),
        )
    }

    /// Executes the `before_shutdown` lifecycle phase for all collected application parts.
    ///
    /// # Parameters
    /// - `cancellation_token`: A [`CancellationToken`] used to control forceful shutdown.
    ///
    /// # Returns
    /// - `Result<(), T::Error>`: Returns `Ok(())` if all parts shut down successfully, or an error from any part.
    #[tracing::instrument(
        name = "application.before_shutdown",
        skip(self, cancellation_token),
        fields(application_parts = self.app_part.name().to_string())
    )]
    async fn before_shutdown(&self, cancellation_token: CancellationToken) -> Result<(), T::Error> {
        tracing::debug!("Executing before_shutdown phase");
        let start = Instant::now();

        self.app_part
            .before_shutdown(cancellation_token)
            .await
            .inspect(|()| {
            tracing::debug!(
                took_ms = start.elapsed().as_millis(),
                "Executed before_shutdown phase sucessfully"
            );
        })
        .inspect_err(
            |err| tracing::error!(took_ms = start.elapsed().as_millis(), %err, "Error during before_shutdown phase"),
        )
    }
}
