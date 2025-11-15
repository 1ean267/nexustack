# Nexustack cron

Nexustack cron provides functionality for scheduling and running cron jobs in a Nexustack application.
It allows developers to define and manage periodic tasks (cron jobs) within their application by
integrating with the application's dependency injection system to provide services and configurations
to the cron jobs.

# Setup
To use the `cron` feature, follow these steps:
1. Enable the cron feature of nexustack, by adding the `cron`feature to your `Cargo.toml` like in the following snippet:\
    ```yaml
    [features]
    nexustack = { version = "*", features = ["cron"] }
    ```
2. Configure your application to enable the `cron` feature in your `main.rs`\
    ```rust, no_run
    use nexustack::{
        application_builder,
        Application as _,
        ApplicationBuilder as _,
        cron::{CronApplicationBuilder as _, CronRunner as _},
    };

    #[tokio::main(flavor = "multi_thread")]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        let app = application_builder()
            .add_cron_with_default_clock()
            .build()?;

        app.run().await?;

        Ok(())
    }
    ```

3. Define your cron jobs by implementing the [`crate::cron::CronJob`] trait or using the [`#[cron]`](crate::cron) attribute macro.\
    ```rust, no_run
    use nexustack::cron::{CronResult, cron};

    #[cron(schedule = "0 0 * * * *")]
    async fn my_cron_job() -> CronResult {
        Ok(())
    }
    ```
4. Configure the application to include the cron jobs and their schedules either in the module where the cron-job resides (See the description of the modules section) or in your `main.rs` for cron-jobs that are part of the main module \
    ```rust, no_run
    use nexustack::{
        application_builder,
        Application as _,
        ApplicationBuilder as _,
        cron::{
            cron_jobs,
            cron,
            CronApplicationBuilder as _,
            CronResult,
            CronRunner as _,
        },
    };

    #[cron(schedule = "0 0 * * * *")]
    async fn my_cron_job() -> CronResult {
        Ok(())
    }

    #[tokio::main(flavor = "multi_thread")]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        let app = application_builder()
            .add_cron_with_default_clock()
            .configure_cron(cron_jobs![
                my_cron_job,
            ])
            .build()?;

        app.run().await?;

        Ok(())
    }
    ```

# Definiting cron jobs via the `#[cron]` Attribute

To define and schedule a cron job that runs periodically, you can use the `#[cron]` attribute macro on a function definition.

## Configuring the schedule parameter

The `#[cron]` attribute accepts a `schedule` parameter that specifies the cron expression for the job.
This expression determines when the job will run. The format of the cron expression is as follows:

```text
* * * * * * *
| | | | | | |
| | | | | | +-- Year (1970–2099) (optional)
| | | | | +---- Day of week (0 - 6) (Sunday=0)
| | | | +------ Month (1 - 12 or JAN-DEC)
| | | +-------- Day of month (1 - 31)
| | +---------- Hours (0 - 23)
| +------------ Minutes (0 - 59)
+-------------- Seconds (0 - 59) (optional)
```

### Field Descriptions
- **Seconds** 
  (Optional) Values range from `0-59` \
  Special characters: `* / , -`
- **Minutes**
  (Mandatory) Values range from `0-59`\
  Special characters: `* / , -`
- **Hours**
  (Mandatory) Values range from `0-23`\
  Special characters: `* / , -`
- **Day of Month**
  (Mandatory) Values range from `1-31`\
  pecial characters: `* / , - L W`
- **Month**
  (Mandatory) Values range from `1-12` or `JAN-DEC`\
  Special characters: `* / , -`
- **Day of Week**
  (Mandatory) Values range from `0-6` or `SUN-SAT`\
  (Sunday is `0`)\
  Special characters: `* / , - L #`
- **Year**
  (Optional) Values range from `1970-2099`\
  Special characters: `* / , -`

```rust, no_run
use nexustack::cron::{cron, CronResult};

/// Every 15 seconds
#[cron(schedule = "*/15 * * * * *")] 
async fn every_15_seconds() -> CronResult {
    println!("This job runs every 15 seconds.");
    Ok(())
}

/// At the start and middle of every minute
#[cron(schedule = "0,30 * * * * *")] 
async fn twice_per_minute() -> CronResult {
    println!("This job runs at the start and middle of every minute.");
    Ok(())
}

/// At midnight and noon every day
#[cron(schedule = "0 0,12 * * * *")] 
async fn midnight_and_noon() -> CronResult {
    println!("This job runs at midnight and noon every day.");
    Ok(())
}
```

### Predefined Scheduling Definitions
The `#[cron]` attribute also supports predefined scheduling definitions:
- `@yearly`\
  Run once a year at midnight on January 1st\
  Equivalent to `0 0 0 1 1 * *`
- `@monthly`\
  Run once a month at midnight on the first day of the month\
  Equivalent to `0 0 0 1 * * *`
- `@weekly`\
  Run once a week at midnight on Sunday\
  Equivalent to `0 0 0 * * 0 *`
- `@daily`\
  Run once a day at midnight\
  Equivalent to `0 0 0 * * * *`
- `@hourly`\
  Run once an hour at the beginning of the hour\
  Equivalent to `0 0 * * * * *`

```rust, no_run
use nexustack::cron::{cron, CronResult};

/// Once a year at midnight on January 1st
#[cron(schedule = "@yearly")]
async fn yearly_job() -> CronResult {
    println!("This job runs once a year at midnight on January 1st.");
    Ok(())
}

/// Once a month at midnight on the first day
#[cron(schedule = "@monthly")]
async fn monthly_job() -> CronResult {
    println!("This job runs once a month at midnight on the first day.");
    Ok(())
}

/// Once a week at midnight on Sunday
#[cron(schedule = "@weekly")]
async fn weekly_job() -> CronResult {
    println!("This job runs once a week at midnight on Sunday.");
    Ok(())
}
```

For more details on cron expressions, refer to the [Wikipedia article on cron expressions](https://en.wikipedia.org/wiki/Cron#CRON_expression).


In the following example, a simple cron job is defined to run every hour. The `my_cron_job` function is annotated with the `#[cron]` attribute, specifying the schedule as `0 0 * * * *`, which corresponds to the start of every hour. The application is configured to include this cron job using the `configure_cron` method. When the application runs, the cron job will execute at the specified schedule, printing a message to the console each time it runs.

```rust, no_run
use nexustack::{
    application_builder,
    Application,
    ApplicationBuilder as _,
    cron::{
        cron,
        cron_jobs,
        CronApplicationBuilder as _,
        CronResult,
        CronRunner as _,
    },
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = application_builder()
        .add_cron_with_default_clock()
        .configure_cron(cron_jobs![my_cron_job])
        .build()?;

    app.run().await?;
    Ok(())
}

/// Every hour
#[cron(schedule = "0 0 * * * *")]
async fn my_cron_job() -> CronResult {
    println!("Running my cron job!");
    Ok(())
}
```

## Dependency Injection

Dependency Injection (DI) allows you to provide services to your cron jobs in a clean and modular way. This is particularly useful when your cron jobs depend on external resources, such as database connections, configuration values, or other application services. By leveraging DI, you can ensure that your cron jobs remain testable, maintainable, and decoupled from the underlying service implementations.

When a cron job runs, a new DI scope is created for that specific execution. This means that the services injected into the cron job are resolved from a scoped service provider, ensuring that each run has its own isolated set of dependencies. This is especially beneficial for managing stateful or per-request services, as it prevents interference between concurrent executions of the same job.

To use DI in a cron job, you can annotate the parameters of the job function with the `#[cron::service]` attribute. These parameters will be automatically resolved from the scoped service provider when the job is executed.


```rust, no_run
use nexustack::{
    application_builder,
    Application as _,
    ApplicationBuilder as _,
    cron::{
        cron,
        cron_jobs,
        CronResult,
        CronApplicationBuilder as _,
        CronRunner as _,
    },
    inject::{ServiceProvider, injectable},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = application_builder()
        .configure_services(|services| {
            services.add_scoped::<MyService>();
        })
        .add_cron_with_default_clock()
        .configure_cron(cron_jobs![my_cron_job])
        .build()?;

    app.run().await?;
    Ok(())
}

#[derive(Clone)]
#[injectable]
struct MyService;

/// Every hour
#[cron(schedule = "0 0 * * * *")]
async fn my_cron_job(#[cron::service] my_service: MyService) -> CronResult {
    println!("Running my cron job with a service!");
    Ok(())
}
```

In this example, the `MyService` dependency is registered as a scoped service. Each time the `my_cron_job` function is executed, a new instance of `MyService` is resolved from the scoped service provider and injected into the job. This ensures that the service is isolated for each execution of the job.

## Dynamic Scheduling

Dynamic scheduling allows you to determine the schedule for a cron job at runtime, rather than specifying it statically in the `#[cron]` attribute. This is particularly useful when the schedule depends on external configuration, user input, or other runtime conditions that are not known at compile time.

To use dynamic scheduling, the `#[cron]` attribute provides the `schedule_with` parameter. This parameter accepts the name of a function that will be called once during application startup to determine the schedule for the cron job. The function must return the schedule as a `nexustack::cron::schedule::Schedule` object, which represents the parsed schedule.

### How the `schedule_with` Function Works

The `schedule_with` function:
1. **Accepts a `ServiceProvider` as its parameter**\
   This allows the function to access application services to compute the schedule dynamically.
2. **Runs only once at application startup**\
   The schedule is resolved during the initialization phase of the application and remains fixed for the lifetime of the application.
3. **Uses a non-scoped `ServiceProvider`**\
   The `ServiceProvider` provided to the function can only resolve singleton and transient services. Scoped services are not available because the function is executed outside the context of a specific cron job run.

### Requirements for the `schedule_with` Function

- The function must return a `Into<CronResult<Schedule>>`, where `Schedule` is the parsed schedule object of type `nexustack::cron::schedule`.
- The returned `Schedule` must be valid; otherwise, the application will fail to start.
- The function should handle any errors gracefully and return an appropriate error if the schedule cannot be determined.

### When to Use Dynamic Scheduling

Dynamic scheduling is necessary when:
- The schedule depends on runtime conditions, such as user preferences or external configuration files.
- The schedule needs to be flexible and configurable without requiring a code change or recompilation.
- The schedule is determined by services or data that are resolved at application startup.

By using dynamic scheduling, you can create more flexible and adaptable cron jobs that respond to runtime requirements while maintaining the benefits of a structured and managed scheduling system.

### Example Usage

```rust, no_run
use nexustack::{
    application_builder,
    Application as _,
    ApplicationBuilder as _,
    cron::{
        cron,
        cron_jobs,
        CronApplicationBuilder as _,
        CronResult,
        CronRunner as _, 
        schedule::Schedule,
    },
    inject::{ServiceProvider, injectable, InjectionResult},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = application_builder()
        .configure_services(|services| {
            services.add_value(CronConfig {
                // Schedule can f.e. be loaded from environment, user input or config files
                schedule: "0 0 * * * *".parse().unwrap(),
            });
        })
        .add_cron_with_default_clock()
        .configure_cron(cron_jobs![my_dynamic_cron_job])
        .build()?;

    app.run().await?;
    Ok(())
}

#[derive(Clone)]
#[injectable]
struct CronConfig {
    schedule: Schedule,
}

#[cron(schedule_with = "get_schedule")]
async fn my_dynamic_cron_job() -> CronResult {
    println!("Running my dynamically scheduled cron job!");
    Ok(())
}

async fn get_schedule(service_provider: ServiceProvider) -> InjectionResult<Schedule> {
    // Resolve the CronConfig service to get the schedule string
    let config = service_provider.resolve::<CronConfig>()?;
    Ok(config.schedule.clone())
}
```

In the example, dynamic scheduling is used to determine the cron job's schedule at runtime. The `CronConfig` struct holds the schedule as a `Schedule` object, which is registered as a singleton service in the application's service container. The `get_schedule` function, specified in the `#[cron(schedule_with = "get_schedule")]` attribute, resolves the `CronConfig` service and retrieves the schedule.

This function is called once during application startup to compute the schedule for the `my_dynamic_cron_job` function. The resolved schedule remains fixed for the lifetime of the application. This approach allows the schedule to be configured dynamically, such as through external configuration files or runtime parameters, without requiring code changes or recompilation.

# Manually implementing a cron job

In some cases, the `#[cron]` attribute macro may not provide enough flexibility for defining complex cron jobs. For example, you may need to dynamically determine both the schedule and the execution logic at runtime, or you may want to implement custom error handling or logging mechanisms. In such scenarios, you can manually implement the `CronJob` trait to define your cron job.

When manually implementing a cron job, you have to implement the `schedule` and `run` methods:

1. **`schedule`**\
   The `schedule` method is responsible for determining the cron job's schedule. It accepts a `ServiceProvider` to resolve any required dependencies (e.g., `CronConfig` in this example) and returns a `Schedule` object.\
   \
   The `ServiceProvider` provided to the `schedule` method is **not scoped**. This means it can only resolve singleton and transient services. Scoped services are not available because the `schedule` method is executed during application startup, outside the context of a specific cron job run.

2. **`run`**\
   The `run` method defines the execution logic for the cron job. It also accepts a `ServiceProvider` to resolve dependencies (e.g., `MyService` in this example) and performs the job's logic.\
   \
   The `ServiceProvider` provided to the `run` method is **scoped**. This means it resolves services within the context of the specific cron job execution. Scoped services are created anew for each execution of the cron job, ensuring isolation between runs.

The distinction in scoped'ness of the `ServiceProvider` instances is important when designing your cron job. Use the `schedule` method for resolving configuration or other global services that determine the schedule, and use the `run` method for resolving services that are specific to the execution of the job.

### Why and When to Manually Implement a Cron Job

This approach is ideal when you need full control over the cron job's behavior. It allows you to dynamically determine the schedule, resolve dependencies, and implement custom logic for the job's execution. While the `#[cron]` attribute is sufficient for most use cases, manually implementing the `CronJob` trait provides the flexibility needed for more advanced scenarios.

As a check-list, you should consider manually implementing a cron job when:
- **Dynamic Behavior**: The schedule or execution logic depends on runtime conditions that cannot be expressed using the `#[cron]` attribute.
- **Custom Error Handling**: You need fine-grained control over how errors are handled during the execution of the cron job.
- **Advanced Logging**: You want to integrate custom logging or monitoring for the cron job's execution.
- **Complex Dependencies**: The cron job requires complex dependency injection or initialization logic that is not supported by the `#[cron]` attribute.

By manually implementing the `CronJob` trait, you gain full control over the cron job's behavior, including how the schedule is determined and how the job is executed.

### Example

The following example demonstrates how to manually implement a cron job:

```rust, no_run
use nexustack::{
    cron::{
        CronError,
        CronJob,
        CronResult,
        schedule::Schedule,
    },
    inject::{ServiceProvider, injectable},
};
use std::future::Future;

#[derive(Clone)]
#[injectable]
struct MyService;

#[derive(Clone)]
#[injectable]
struct CronConfig {
    schedule: Schedule,
}

struct MyCronJob;

impl CronJob for MyCronJob {
    async fn schedule(
        service_provider: ServiceProvider,
    ) -> CronResult<Schedule> {
        // Resolve the CronConfig service to get the schedule
        let config = service_provider
            .resolve::<CronConfig>()
            .map_err(|err| CronError::ScheduleError(err.into()))?;
        
        Ok(config.schedule.clone())
    }

    async fn run(service_provider: ServiceProvider) -> CronResult {
        // Resolve the MyService dependency
        let my_service = service_provider
            .resolve::<MyService>()
            .map_err(|err| CronError::RunError(err.into()))?;
        
        // Perform the cron job logic
        println!("Running MyCronJob with MyService!");
        
        // Return success
        Ok(())
    }
}
```

# Custom cron clock

A custom cron clock allows you to define how time is managed and perceived within the cron subsystem. This is particularly useful in scenarios such as:

- **Testing**\
  Simulate time progression or control time deterministically.
- **Custom Time Zones**\
  Use a specific time zone or calendar system.
- **Mocking**\
  Replace the default system clock with a mock implementation for debugging or testing.

## Structure and Principles

A custom cron clock must implement the `CronClock` trait, which defines the following methods:
- `timezone`\
  Returns the time zone in which the clock operates.
- `now`\
  Returns the current date and time in the clock's time zone.
- `delay_until`\
  Delays execution until a specified date and time or until a cancellation token is triggered.

The `CronClock` trait ensures that the clock integrates seamlessly with the cron subsystem, providing consistent and predictable behavior.

## Steps to Create and Register a Custom Cron Clock

1. **Define the Custom Clock**\
   Implement the `CronClock` trait for your custom clock.
2. **Register the Custom Clock in the DI System**\
   Add the custom clock to the application's dependency injection (DI) system using the `configure_services` method.
3. **Register the Clock with the Cron System**\
   Use the `add_cron` method of the `ApplicationBuilder` to register the custom clock.
4. **Use the Clock**\
   The custom clock will be used to schedule and execute cron jobs.

## Example

The following example demonstrates how to implement a custom cron clock using an enum `DateTimeProvider`. This enum has two variants:
- `DateTimeProvider::System`\
  Uses the system clock.
- `DateTimeProvider::Test`\
  Simulates time progression for testing purposes.


```rust, no_run
use nexustack::{
    application_builder,
    ApplicationBuilder as _,
    Application as _,
    cron::{
        cron,
        cron_jobs,
        CronApplicationBuilder as _,
        CronClock,
        CronError,
        CronResult,
        CronRunner as _,
    },
    inject::{ServiceProvider, injectable},
};
use chrono::{DateTime, TimeZone, Utc};
use std::{pin::Pin, future::Future};
use tokio::{time::{sleep, Duration}, sync::Notify};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub enum DateTimeProvider {
    System,
    #[cfg(test)]
    Test(TestDateTimeProvider),
}

#[cfg(test)]
struct TestDateTimeProvider {
    current_time: Cell<DateTime<Utc>>,
    notify_time_advanced: Notify,
}

#[cfg(test)]
impl TestDateTimeProvider {
    pub fn advance_time(&mut self, duration: Duration) {
        self.current_time.update(|current_time| current_time + chrono::Duration::from_std(duration).unwrap());
        self.notify_time_advanced.notify_waiters();
    }
}

impl DateTimeProvider {
    pub fn system() -> Self {
        Self::System
    }

    #[cfg(test)]
    pub fn test(current_time: DateTime<Utc>) -> Self {
        Self::Test(TestDateTimeProvider {
            current_time: Cell::new(current_time),
            notify_time_advanced: Notify::new(),
        })
    }
}

impl CronClock for DateTimeProvider {
    type TimeZone = Utc;
    type DelayUntilFuture<'a> = Pin<Box<dyn Future<Output = CronResult<DateTime<Utc>>> + Send + 'a>>;

    fn timezone(&self) -> Self::TimeZone {
        Utc
    }

    fn now(&self) -> DateTime<Self::TimeZone> {
        match self {
            DateTimeProvider::System => Utc::now(),
            #[cfg(test)]
            DateTimeProvider::Test(inner) => inner.current_time.clone().take(),
        }
    }

    fn delay_until(
        &self,
        date_time: DateTime<Self::TimeZone>,
        cancellation_token: CancellationToken,
    ) -> Self::DelayUntilFuture<'_> {
        match self {
            DateTimeProvider::System => Box::pin(
                async move {
                    loop {
                        let now = self.now();
                        let duration = (date_time - now).to_std().unwrap_or_default();

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
            ),
            #[cfg(test)]
            DateTimeProvider::Test(inner) => Box::pin(
                async move {
                    let now = self.now();
                    let duration = (date_time - now).to_std().unwrap_or_default();

                    if duration.is_zero() {
                        return Ok(now);
                    }

                    tokio::select! {
                        () = inner.notify_time_advanced.notified() => { }
                        () = cancellation_token.cancelled() => {
                            return Err(CronError::Canceled);
                        }
                    }
                }
            )
        }  
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = application_builder()
        .configure_services(|services| {
            // Register the custom clock in the DI system
            services.add_value(DateTimeProvider::System);
        })
        // Register the custom clock with the cron system
        .add_cron::<DateTimeProvider>() 
        .configure_cron(cron_jobs![my_cron_job])
        .build()?;

    app.run().await?;
    Ok(())
}

#[cron(schedule = "0 0 * * * *")]
async fn my_cron_job() -> CronResult {
    println!("Running my cron job!");
    Ok(())
}
```
