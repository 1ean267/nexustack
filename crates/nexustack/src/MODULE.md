# Nexustack Modules

## What is the Module System?

The Nexustack module system is a powerful abstraction that allows developers to define modular components in their applications. These modules are defined as traits, which encapsulate specific functionality and can be easily integrated into an application. The module system is designed to enhance code organization, reusability, and maintainability.

## Why and When to Use Modules

Modules are particularly useful in the following scenarios:

- **Large Applications**\
  When building large-scale applications, modules help in breaking down the application into smaller, manageable components.
- **Feature Toggles**\
  Modules allow conditional compilation of features, enabling or disabling specific functionality based on the application's requirements.
- **Code Reusability**\
  Modules can be reused across different parts of the application or even in different projects.
- **Separation of Concerns**\
  By encapsulating functionality within modules, you can achieve a clear separation of concerns, making the codebase easier to understand and maintain.

## How to Use Modules

To use modules in your application, you need to define them as traits and annotate them with the [`#[module]`](crate::module) attribute. The [`#[module]`](crate::module) attribute provides additional metadata and configuration options for the module.


Here is a basic usage example of defining and using a module:

```rust, no_run
use nexustack::{
    application_builder,
    Application as _,
    ApplicationBuilder,
    module,
    cron::{
        cron,
        cron_jobs,
        CronApplicationBuilder as _,
        CronResult,
        Cron,
    },
};


// -- In main.rs --

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = application_builder()
        .add_cron_with_default_clock()
        // Import the module into the application
        .add_notes()
        .build()?;

    app.run().await?;

    Ok(())
}


// -- In the modules mod.rs --

#[module(features = "Cron")]
pub trait NotesModule {
    fn add_notes(self) -> impl ApplicationBuilder {
        self.configure_cron(cron_jobs![remove_outdated_notes_cron_job])
    }
}

#[cron(schedule = "0 0 * * * *")]
async fn remove_outdated_notes_cron_job() -> CronResult {
    // Logic to remove outdated notes
    Ok(())
}
```

In this example, the `NotesModule` trait defines a module that adds a cron job to remove outdated notes.

## Modules vs Rust Modules and Cargo Features

- **Modules**\
  In Nexustack, modules are traits that encapsulate specific functionality. They are not the same as Rust's native modules, which are used for organizing code into namespaces.
- **Features**\
  Features in Nexustack are similar to Cargo features. They allow conditional compilation of code based on the specified features. However, Nexustack features are tied to modules and provide finer-grained control over functionality. Often, Nexustack features are behind Cargo feature flags. For example, to use the [`cron`](crate::cron) Nexustack feature, the Cargo `cron` feature must be enabled for the Nexustack crate.

## The [`#[module]`](crate::module) Attribute

The [`#[module]`](crate::module) attribute is used to define a module. It supports the following properties:

- **features**\
  Specifies the nexustack features required by the module. For example, [`#[module(features = "Cron")]`](crate::module) indicates that the module depends on the nexustack `Cron` feature.
- **crate**\
  Specifies a path to the Nexustack crate instance to use when referring to Nexustack APIs from generated code. This is normally only applicable when invoking re-exported Nexustack derives from a public macro in a different crate.

In this example, the module depends on the nexustack Features`Cron` and `Http`.


```rust, ignore
#[module(features = "Cron, Http")]
pub trait MyModule {
    fn add_my_module(self) -> impl ApplicationBuilder {
        [...]
    }
}
```

## Naming Conventions

When working with Nexustack modules, adhering to consistent naming conventions can greatly enhance the readability and maintainability of your codebase. Below are some recommended naming conventions:

### Module Traits
- **Convention**: Use the `Module` suffix for all module traits.
- **Example**: `NotesModule`, `UserModule`, `StockModule`.
- **Reason**: This makes it immediately clear that the trait represents a Nexustack module.

### Functions in Module Traits
- **Convention**: Use the `add_` prefix followed by the module name for functions that integrate the module into the application.
- **Example**: `add_notes`, `add_user`, `add_stock`.
- **Reason**: This ensures consistency and makes the purpose of the function obvious.

### File and Folder Structure
- **Convention**: Organize module-related files in a dedicated folder named after the module, and use `mod.rs` as the entry point.
- **Example**: For a `NotesModule`, create a folder `notes/` containing `mod.rs` and other related files.
- **Reason**: This keeps module-related code encapsulated and easy to locate.

By following these conventions, you can ensure that your Nexustack modules are easy to understand, maintain, and extend.

## Integration Workspace Setups

### Workspace Setup 1: Modules in Multiple Subfolders

This workspace setup is ideal for applications where modules are tightly coupled and share a common codebase. By organizing modules into subfolders, you can maintain a clear structure while keeping all related files in one place. This approach is particularly useful for medium-sized projects where the modules are not large enough to warrant separate crates but still require some level of separation for better organization.

#### Why Use This Setup?
- **Centralized Codebase**\
  All modules are part of the same crate, making it easier to manage dependencies and build processes.
- **Ease of Refactoring**\
  Since all code resides in a single crate, refactoring and moving code between modules is straightforward.
- **Simplified Dependency Management**\
  You don’t need to manage inter-crate dependencies, as everything is within the same crate.

#### Workspace Structure:
```text
my_project/
├── src/
│   ├── main.rs
│   ├── module_a/
│   │   ├── mod.rs
│   │   ├── cron_a.rs
│   │   ├── controller_x.rs
│   │   └── ...
│   └── module_b/
│       ├── mod.rs
│       ├── mqtt_hubs.rs
│       └── ...
```

#### Code Examples:
- `src/module_a/mod.rs`:

```rust, ignore
#[module(features = "Cron, Http")]
pub trait ModuleA {
    fn add_module_a(self) -> impl ApplicationBuilder {
        [...]
    }
}
```

- `src/module_b/mod.rs`:

```rust, ignore
#[module(features = "Mqtt")]
pub trait ModuleB {
    fn add_module_b(self) -> impl ApplicationBuilder {
        [...]
    }
}
```

### Workspace Setup 2: Modules as Separate Crates

This setup is best suited for large-scale applications where modules are independent enough to be developed, tested, and maintained separately. By placing each module in its own crate, you can achieve a high level of modularity and reusability. This approach is particularly useful for teams working on different modules simultaneously or when modules need to be reused across multiple projects.

#### Why Use This Setup?
- **High Modularity**\
  Each module is a separate crate, allowing for independent development and testing.
- **Reusability**\
  Modules can be published as separate crates and reused in other projects.
- **Team Collaboration**\
  Teams can work on different modules without interfering with each other.

#### Workspace Structure:
```text
my_workspace/
├── module_a/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── cron_a.rs
│       ├── controller_x.rs
├── module_b/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── mqtt_hubs.rs
│       └── ...
└── main_app/
    ├── Cargo.toml
    └── src/
        └── main.rs
```

#### Code Examples:
- `module_a/src/lib.rs`:

```rust, ignore
#[module(features = "Cron, Http")]
pub trait ModuleA {
    fn add_module_a(self) -> impl ApplicationBuilder {
        [...]
    }
}
```

- `module_b/src/lib.rs`:

```rust, ignore
#[module(features = "Mqtt")]
pub trait ModuleB {
    fn add_module_b(self) -> impl ApplicationBuilder {
        [...]
    }
}
```

- `main_app/src/main.rs`:

```rust, ignore
use module_a::ModuleA;
use module_b::ModuleB;

# fn main() {
    let app = MyApp::new()
        .add_module_a()
        .add_module_b();
    app.run();
# }
```
