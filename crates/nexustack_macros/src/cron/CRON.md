The `#[cron]` attribute macro allows you to define and schedule cron jobs in a declarative manner.

# Usage

The `#[cron]` macro can be applied to an asynchronous function to define a cron job. The macro
supports two primary parameters:

- `schedule`: A static cron expression that specifies when the job should run.
- `schedule_with`: A function name that dynamically determines the schedule at runtime.

## Parameters

### `schedule`
A static cron expression that defines the schedule for the job. The expression follows the
standard cron syntax, with fields for seconds, minutes, hours, day of the month, month, day of
the week, and year. Predefined scheduling definitions like `@yearly`, `@monthly`, etc., are also
supported.

Example:
```rust, no_run
use nexustack::cron::{cron, CronResult};

#[cron(schedule = "0 0 * * * *")]
async fn hourly_job() -> CronResult {
    println!("This job runs at the start of every hour.");
    Ok(())
}
```

### `schedule_with`
A function name that dynamically determines the schedule at runtime. The function must accept a
`ServiceProvider` and return a `Result<Schedule, InjectionError>`. This is useful when the
schedule depends on runtime conditions or external configuration.

Example:
```rust, no_run
use nexustack::{
    cron::{cron, CronResult, schedule::Schedule},
    inject::{InjectionResult, ServiceProvider, injectable},
};

#[derive(Clone)]
#[injectable]
struct CronConfig {
    schedule: Schedule,
}

#[cron(schedule_with = "get_dynamic_schedule")]
async fn dynamic_job() -> CronResult {
    println!("This job runs based on a dynamic schedule.");
    Ok(())
}

async fn get_dynamic_schedule(service_provider: ServiceProvider) -> InjectionResult<Schedule> {
    let config = service_provider.resolve::<CronConfig>()?;
    Ok(config.schedule.clone())
}
```

## Dependency Injection

The `#[cron]` macro supports dependency injection for the job's parameters. Annotate the
parameters with `#[cron::service]` to resolve them from the scoped service provider.

Example:
```rust, no_run
use nexustack::{
    cron::{cron, CronResult},
    inject::injectable,
};

#[derive(Clone)]
#[injectable]
struct MyService;

#[cron(schedule = "0 0 * * * *")]
async fn job_with_service(#[cron::service] my_service: MyService) -> CronResult {
    println!("Running job with injected service!");
    Ok(())
}
```

## Predefined Scheduling Definitions

- `@yearly`: Runs once a year at midnight on January 1st.
- `@monthly`: Runs once a month at midnight on the first day.
- `@weekly`: Runs once a week at midnight on Sunday.
- `@daily`: Runs once a day at midnight.
- `@hourly`: Runs once an hour at the beginning of the hour.

Example:
```rust, no_run
use nexustack::cron::{cron, CronResult};

#[cron(schedule = "@daily")]
async fn daily_job() -> CronResult {
    println!("This job runs once a day at midnight.");
    Ok(())
}
```

For more details on cron expressions, refer to the [Wikipedia article on cron expressions](https://en.wikipedia.org/wiki/Cron#CRON_expression).