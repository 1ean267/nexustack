<div align="center">
  <img alt="Nexustack logo" src="`HTTP`s://github.com/1ean267/nexustack/raw/main/artifacts/logo_no_text_round.png" height="128">
  <h1>Nexustack</h1>
  <a href="`HTTP`s://github.com/1ean267/nexustack/actions">
    <img alt="GitHub branch check runs" src="`HTTP`s://img.shields.io/github/check-runs/1ean267/nexustack/main?style=for-the-badge&labelColor=000000">
  </a>
  <a href="`HTTP`s://crates.io/crates/nexustack">
    <img alt="Deps.rs Crate Dependencies (latest)" src="`HTTP`s://img.shields.io/deps-rs/nexustack/latest?style=for-the-badge&labelColor=000000">
  </a>
  <a href="`HTTP`s://crates.io/crates/nexustack"><img alt="Version" src="`HTTP`s://img.shields.io/crates/v/nexustack.svg?style=for-the-badge&labelColor=000000"></a>
  <a href="`HTTP`s://docs.rs/nexustack/latest/nexustack/">
    <img alt="docs.rs" src="`HTTP`s://img.shields.io/docsrs/nexustack?style=for-the-badge&labelColor=000000">
  </a>
  <a href="`HTTP`s://github.com/1ean267/nexustack/blob/main/LICENSE">
    <img alt="License" src="`HTTP`s://img.shields.io/crates/l/nexustack?style=for-the-badge&labelColor=000000">
  </a>
</div>

> []()\
> **Warning:** Nexustack is a work in progress. Do not use in production environments yet\
> []()\
> []()

Nexustack is not just another `HTTP` web framework. While there are countless web frameworks available, Nexustack aims to unify multiple functionalities under a single, cohesive system. It is designed to provide a stable, tested, and enterprise-ready solution for building modern applications.

### Unified Functionality

Nexustack stands out by integrating multiple functionalities into a single framework. Instead of relying on separate libraries for `HTTP`, `WebSockets`, `MQTT`, `AMQP`, and cron jobs, Nexustack provides a unified solution. This eliminates the need to manage multiple dependencies and ensures that all features work seamlessly together. By combining these capabilities, Nexustack simplifies the development of modern, multi-protocol applications.

### Dependency Injection (DI)

A robust dependency injection (DI) system is at the core of Nexustack. Inspired by frameworks like `NestJS` and `ASP.NET Core`, the DI system ensures modularity and testability. Developers can register services with different lifetimes (singleton, scoped, or transient) and resolve them effortlessly. This approach promotes clean architecture, reduces coupling, and makes it easier to write unit tests for individual components.

### Modular Architecture

Nexustack encourages developers to structure their applications into coherent modules. Each module encapsulates a specific feature or domain, making the application easier to maintain and scale. This modular design aligns with best practices for enterprise software development, enabling teams to work on different parts of the application independently.

### API Documentation

Documentation is a critical aspect of any application, and Nexustack makes it effortless. The framework automatically generates `OpenAPI` documentation for `HTTP` services and `AsyncAPI` documentation for `WebSockets`, `MQTT`, and `AMQP` services. This ensures that your APIs are well-documented and ready for integration with other systems, saving time and effort during development.

### Enterprise Focus

Nexustack is designed with enterprise environments in mind. It prioritizes stability, testing, and consistency, making it suitable for large-scale applications. The framework provides a unified way to use its features, reducing the learning curve for developers and ensuring that applications are built on a solid foundation. Nexustack aims to be a reliable choice for organizations looking to adopt Rust for their enterprise solutions.

## Features

- **`HTTP` Server** · Serve `RESTful` APIs with ease.
- **`WebSockets`** · Real-time communication using `WebSockets`.
- **`MQTT`** · Support for `MQTT` protocol for `IoT` and messaging.
- **`AMQP`** · Integration with `AMQP` for message brokers like `RabbitMQ`.
- **[Cron Jobs](`crate::cron`)** · Schedule and manage periodic tasks.
- **[Dependency Injection](`crate::inject`)** · A powerful DI system inspired by `NestJS` and `ASP.NET Core`.
- **Modular Design** · Structure your application into reusable modules.
- **[`OpenAPI` Documentation](`crate::openapi`)** · Automatically generate `OpenAPI` documentation for your `HTTP` services.
- **`AsyncAPI` Documentation** · Generate `AsyncAPI` documentation for `WebSockets`, `MQTT`, and `AMQP` services.

> **Note**: Not all features are implemented yet. Nexustack is a work in progress.


## Quick-Start Guide

### Installation

Add `nexustack` by adding in as dependency to your `Cargo.toml`:

```toml
[dependencies]
nexustack = "0.1"
```
You can also add it by running the following command:
```bash
cargo add nexustack
```

### Basic Setup

Create a new Rust project and set up a simple Nexustack application:

```rust, no_run
use nexustack::{
    application_builder,
    Application as _,
    ApplicationBuilder as _,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = application_builder().build()?;
    app.run().await?;
    Ok(())
}
```

### Adding Features

#### Cron Jobs

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
      },
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = application_builder()
        .add_cron_with_default_clock()
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

For more examples, refer to the documentation of each feature.


## License

Nexustack is licensed under the MIT License. See the [LICENSE](https://github.com/1ean267/nexustack/blob/main/LICENSE) file for details.

## Contributing

Contributions are welcome! If you'd like to contribute to Nexustack, please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Write tests for your changes.
4. Submit a pull request.

Before contributing, please ensure that your code adheres to the project's coding standards and passes all tests.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in nexustack by you, shall be licensed as MIT, without any additional terms or conditions.