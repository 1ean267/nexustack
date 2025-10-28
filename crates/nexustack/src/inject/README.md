# Nexustack inject

Nexustack inject is a dependency inject (DI) solution that is inspired by popular frameworks in other languages like [NestJS providers](https://docs.nestjs.com/providers) and [ASP.NET Core Dependency injection](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/dependency-injection).  

It provides a public API that can be used to register services with different lifetime characteristics and later resolve them from the built-in container. Custom services can either manually implement the necessary traits use the provided macro for ease of use.

## Usage examples

Simple sample application that registers a bunch of services and later resolves them.

```rust

use std::sync::Arc;
use nexustack::inject::{ServiceCollection, ServiceProvider};

#[derive(Clone)]
struct UnitService;

struct SingletonService(UnitService);

fn main() {
    let services = build_service_provider();
    let singleton_service = services.resolve::<Arc<SingletonService>>().unwrap();
}

fn build_service_provider() -> ServiceProvider {
    let mut services = ServiceCollection::new();
    services
        .add_value(UnitService)
        .add_singleton_factory(|injector| Ok(Arc::new(SingletonService(injector.resolve()?))));
    services.build()
}

```

Usage of the built in macro to enable custom services to have dependencies and to be injectable.

```rust

use std::sync::Arc;
use nexustack::inject::{injectable, ServiceCollection, ServiceProvider};

#[derive(Clone)]
struct UnitService;

#[derive(Clone)]
struct CustomService {
    unit_service: UnitService
}

#[injectable]
impl CustomService {
    pub fn new(unit_service: UnitService) -> Self {
        Self { unit_service }
    }
}

fn main() {
    let services = build_service_provider();
    let singleton_service = services.resolve::<CustomService>().unwrap();
}

fn build_service_provider() -> ServiceProvider {
    let mut services = ServiceCollection::new();
    services
        .add_value(UnitService)
        .add_singleton::<CustomService>();
    services.build()
}

```