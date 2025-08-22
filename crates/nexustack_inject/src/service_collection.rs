/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::{
    container::{
        ContainerBuilder, ScopedContainerEntryBuilder, ScopedUntypedContainerEntryBuilder,
        SingletonContainerEntryBuilder, TransientContainerEntryBuilder,
        UntypedContainerEntryBuilder,
    },
    injectable::Injectable,
    injection_error::ConstructionResult,
    injector::Injector,
    service_provider::ServiceProvider,
};

/// Represents a service-collection that can be used to register and collection services. It acts as a factory
/// for a [`ServiceProvider`] that can be constructed via its [build] function.
pub struct ServiceCollection {
    root_builders: Vec<Box<dyn UntypedContainerEntryBuilder>>,
    scoped_builders: Vec<Box<dyn ScopedUntypedContainerEntryBuilder + Send + Sync>>,
}

impl ServiceCollection {
    /// Constructs a new empty service collection.
    #[must_use]
    pub fn new() -> Self {
        Self {
            root_builders: Vec::new(),
            scoped_builders: Vec::new(),
        }
    }

    /// Consumes the service collection and constructs a [`ServiceProvider`] that can be used to resolve the registered
    /// services.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexustack_inject::injectable;
    /// use nexustack_inject::ServiceCollection;
    ///
    /// #[derive(Clone)]
    /// struct MyService { }
    ///
    /// #[injectable]
    /// impl MyService {
    ///     pub fn new() -> Self {
    ///         Self { }
    ///     }
    /// }
    ///
    /// let service_provider = ServiceCollection::new()
    ///     .add_singleton::<MyService>()
    ///     .build();
    /// ```
    #[must_use]
    pub fn build(self) -> ServiceProvider {
        let container_builder = ContainerBuilder::new(
            self.root_builders,
            if self.scoped_builders.is_empty() {
                None
            } else {
                Some(self.scoped_builders)
            },
            None,
        );

        container_builder.build()
    }
}

impl Default for ServiceCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceCollection {
    // TODO: Add missing register functions

    /// Adds a value as singleton service to the service collection.
    ///
    /// # Type arguments
    ///
    /// * `TService` - The type of the service to register.
    ///
    /// # Arguments
    ///
    /// * `value` - The value of the service type to register as service.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexustack_inject::injectable;
    /// use nexustack_inject::ServiceCollection;
    ///
    /// #[derive(Clone)]
    /// struct MyService { }
    ///
    /// impl MyService {
    ///     pub fn new() -> Self {
    ///         Self { }
    ///     }
    /// }
    ///
    /// let service_provider = ServiceCollection::new()
    ///     .add_value(MyService::new())
    ///     .build();
    ///
    /// let my_service = service_provider.resolve::<MyService>().unwrap();
    /// ```
    #[must_use]
    pub fn add_value<TService: Clone + Send + Sync + 'static>(mut self, value: TService) -> Self {
        self.root_builders
            .push(Box::new(SingletonContainerEntryBuilder::new(|_| Ok(value))));

        self
    }

    /// Adds a singleton service to the service collection. The service must implement the [Injectable] trait.
    ///
    /// # Type arguments
    ///
    /// * `TService` - The type of the service to register.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::sync::atomic::Ordering;
    /// use std::sync::atomic::AtomicUsize;
    /// use nexustack_inject::injectable;
    /// use nexustack_inject::ServiceCollection;
    /// use nexustack_inject::ServiceScope;
    ///
    /// static NEXT_SEQ_NUM: AtomicUsize = AtomicUsize::new(1);
    ///
    /// #[derive(Clone)]
    /// struct MyService {
    ///     pub seq_num: usize
    /// }
    ///
    /// #[injectable]
    /// impl MyService {
    ///     pub fn new() -> Self {
    ///         Self { seq_num: NEXT_SEQ_NUM.fetch_add(1, Ordering::Relaxed) }
    ///     }
    /// }
    ///
    /// let service_provider = ServiceCollection::new()
    ///     .add_singleton::<MyService>()
    ///     .build();
    ///
    /// let my_service = service_provider.resolve::<MyService>().unwrap();
    ///
    /// assert_eq!(1, my_service.seq_num);
    ///
    /// let my_service = service_provider.resolve::<MyService>().unwrap();
    ///
    /// assert_eq!(1, my_service.seq_num);
    ///
    /// let service_scope = service_provider.resolve::<ServiceScope>().unwrap();
    /// let my_service = service_scope.service_provider().resolve::<MyService>().unwrap();
    ///
    /// assert_eq!(1, my_service.seq_num);
    /// ```
    #[must_use]
    pub fn add_singleton<TService: Clone + Send + Sync + Injectable + 'static>(mut self) -> Self {
        self.root_builders
            .push(Box::new(SingletonContainerEntryBuilder::new(
                TService::from_injector,
            )));

        self
    }

    /// Adds a singleton service to the service collection. The service must implement the [Injectable] trait.
    ///
    /// # Type arguments
    ///
    /// * `TService` - The type of the service to register.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::sync::atomic::Ordering;
    /// use std::sync::atomic::AtomicUsize;
    /// use nexustack_inject::injectable;
    /// use nexustack_inject::ServiceCollection;
    /// use nexustack_inject::ServiceScope;
    ///
    /// static NEXT_SEQ_NUM: AtomicUsize = AtomicUsize::new(1);
    ///
    /// #[derive(Clone)]
    /// struct MyService {
    ///     pub seq_num: usize
    /// }
    ///
    /// #[injectable]
    /// impl MyService {
    ///     pub fn new() -> Self {
    ///         Self { seq_num: NEXT_SEQ_NUM.fetch_add(1, Ordering::Relaxed) }
    ///     }
    /// }
    ///
    /// let service_provider = ServiceCollection::new()
    ///     .add_scoped::<MyService>()
    ///     .build();
    ///
    /// let my_service_result = service_provider.resolve::<MyService>();
    ///
    /// assert!(my_service_result.is_err());
    ///
    /// let service_scope_1 = service_provider.resolve::<ServiceScope>().unwrap();
    /// let my_service = service_scope_1.service_provider().resolve::<MyService>().unwrap();
    ///
    /// assert_eq!(1, my_service.seq_num);
    ///
    /// let my_service = service_scope_1.service_provider().resolve::<MyService>().unwrap();
    ///
    /// assert_eq!(1, my_service.seq_num);
    ///
    /// let service_scope_2 = service_provider.resolve::<ServiceScope>().unwrap();
    /// let my_service = service_scope_2.service_provider().resolve::<MyService>().unwrap();
    ///
    /// assert_eq!(2, my_service.seq_num);
    /// ```
    #[must_use]
    pub fn add_scoped<TService: Clone + Send + Sync + Injectable + 'static>(mut self) -> Self {
        self.scoped_builders
            .push(Box::new(ScopedContainerEntryBuilder::new(
                TService::from_injector,
            )));

        self
    }

    /// Adds a transient service to the service collection. The service must implement the [Injectable] trait.
    ///
    /// # Type arguments
    ///
    /// * `TService` - The type of the service to register.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::sync::atomic::Ordering;
    /// use std::sync::atomic::AtomicUsize;
    /// use nexustack_inject::injectable;
    /// use nexustack_inject::ServiceCollection;
    /// use nexustack_inject::ServiceScope;
    ///
    /// static NEXT_SEQ_NUM: AtomicUsize = AtomicUsize::new(1);
    ///
    /// #[derive(Clone)]
    /// struct MyService {
    ///     pub seq_num: usize
    /// }
    ///
    /// #[injectable]
    /// impl MyService {
    ///     pub fn new() -> Self {
    ///         Self { seq_num: NEXT_SEQ_NUM.fetch_add(1, Ordering::Relaxed) }
    ///     }
    /// }
    ///
    /// let service_provider = ServiceCollection::new()
    ///     .add_transient::<MyService>()
    ///     .build();
    ///
    /// let my_service = service_provider.resolve::<MyService>().unwrap();
    ///
    /// assert_eq!(1, my_service.seq_num);
    ///
    /// let my_service = service_provider.resolve::<MyService>().unwrap();
    ///
    /// assert_eq!(2, my_service.seq_num);
    ///
    /// let service_scope = service_provider.resolve::<ServiceScope>().unwrap();
    /// let my_service = service_scope.service_provider().resolve::<MyService>().unwrap();
    ///
    /// assert_eq!(3, my_service.seq_num);
    /// ```
    #[must_use]
    pub fn add_transient<TService: Send + Sync + Injectable + 'static>(mut self) -> Self {
        self.root_builders
            .push(Box::new(TransientContainerEntryBuilder::new(
                TService::from_injector,
            )));

        self
    }

    /// Adds a singleton service to the service collection via the provided factory function.
    ///
    /// # Type arguments
    ///
    /// * `TService` - The type of the service to register.
    ///
    /// # Argument
    ///
    /// * 'factory' - The factory function that constructs the service. It is passed an [Injector] that can be used to resolve
    ///   dependency services.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::sync::atomic::Ordering;
    /// use std::sync::atomic::AtomicUsize;
    /// use nexustack_inject::injectable;
    /// use nexustack_inject::ServiceCollection;
    /// use nexustack_inject::ServiceScope;
    ///
    /// static NEXT_SEQ_NUM: AtomicUsize = AtomicUsize::new(1);
    ///
    /// #[derive(Clone)]
    /// struct Dependency { }
    ///
    /// #[injectable]
    /// impl Dependency {
    ///      pub fn new() -> Self {
    ///         Self { }
    ///     }
    /// }
    ///
    /// #[derive(Clone)]
    /// struct MyService {
    ///     dependency: Dependency,
    ///     pub seq_num: usize
    /// }
    ///
    /// impl MyService {
    ///     pub fn new(dependency: Dependency) -> Self {
    ///         Self { dependency, seq_num: NEXT_SEQ_NUM.fetch_add(1, Ordering::Relaxed) }
    ///     }
    /// }
    ///
    /// let service_provider = ServiceCollection::new()
    ///     .add_singleton::<Dependency>()
    ///     .add_singleton_factory(|injector| Ok(MyService::new(injector.resolve::<Dependency>()?)))
    ///     .build();
    ///
    /// let my_service = service_provider.resolve::<MyService>().unwrap();
    ///
    /// assert_eq!(1, my_service.seq_num);
    ///
    /// let my_service = service_provider.resolve::<MyService>().unwrap();
    ///
    /// assert_eq!(1, my_service.seq_num);
    ///
    /// let service_scope = service_provider.resolve::<ServiceScope>().unwrap();
    /// let my_service = service_scope.service_provider().resolve::<MyService>().unwrap();
    ///
    /// assert_eq!(1, my_service.seq_num);
    /// ```
    #[must_use]
    pub fn add_singleton_factory<TService: Clone + Send + Sync + 'static>(
        mut self,
        factory: impl FnOnce(&Injector) -> ConstructionResult<TService> + 'static,
    ) -> Self {
        self.root_builders
            .push(Box::new(SingletonContainerEntryBuilder::new(factory)));

        self
    }

    /// Adds a scoped service to the service collection via the provided factory function.
    ///
    /// # Type arguments
    ///
    /// * `TService` - The type of the service to register.
    ///
    /// # Argument
    ///
    /// * 'factory' - The factory function that constructs the service. It is passed an [Injector] that can be used to resolve
    ///   dependency services.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::sync::atomic::Ordering;
    /// use std::sync::atomic::AtomicUsize;
    /// use nexustack_inject::injectable;
    /// use nexustack_inject::ServiceCollection;
    /// use nexustack_inject::ServiceScope;
    ///
    /// static NEXT_SEQ_NUM: AtomicUsize = AtomicUsize::new(1);
    ///
    /// #[derive(Clone)]
    /// struct Dependency { }
    ///
    /// #[injectable]
    /// impl Dependency {
    ///      pub fn new() -> Self {
    ///         Self { }
    ///     }
    /// }
    ///
    /// #[derive(Clone)]
    /// struct MyService {
    ///     dependency: Dependency,
    ///     pub seq_num: usize
    /// }
    ///
    /// impl MyService {
    ///     pub fn new(dependency: Dependency) -> Self {
    ///         Self { dependency, seq_num: NEXT_SEQ_NUM.fetch_add(1, Ordering::Relaxed) }
    ///     }
    /// }
    ///
    /// let service_provider = ServiceCollection::new()
    ///     .add_singleton::<Dependency>()
    ///     .add_scoped_factory(|injector| Ok(MyService::new(injector.resolve::<Dependency>()?)))
    ///     .build();
    ///
    /// let my_service_result = service_provider.resolve::<MyService>();
    ///
    /// assert!(my_service_result.is_err());
    ///
    /// let service_scope_1 = service_provider.resolve::<ServiceScope>().unwrap();
    /// let my_service = service_scope_1.service_provider().resolve::<MyService>().unwrap();
    ///
    /// assert_eq!(1, my_service.seq_num);
    ///
    /// let my_service = service_scope_1.service_provider().resolve::<MyService>().unwrap();
    ///
    /// assert_eq!(1, my_service.seq_num);
    ///
    /// let service_scope_2 = service_provider.resolve::<ServiceScope>().unwrap();
    /// let my_service = service_scope_2.service_provider().resolve::<MyService>().unwrap();
    ///
    /// assert_eq!(2, my_service.seq_num);
    /// ```
    #[must_use]
    pub fn add_scoped_factory<TService: Clone + Send + Sync + 'static>(
        mut self,
        factory: impl Fn(&Injector) -> ConstructionResult<TService> + Send + Sync + 'static,
    ) -> Self {
        self.scoped_builders
            .push(Box::new(ScopedContainerEntryBuilder::new(factory)));

        self
    }

    /// Adds a transient service to the service collection via the provided factory function.
    ///
    /// # Type arguments
    ///
    /// * `TService` - The type of the service to register.
    ///
    /// # Argument
    ///
    /// * 'factory' - The factory function that constructs the service. It is passed an [Injector] that can be used to resolve
    ///   dependency services.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::sync::atomic::Ordering;
    /// use std::sync::atomic::AtomicUsize;
    /// use nexustack_inject::injectable;
    /// use nexustack_inject::ServiceCollection;
    /// use nexustack_inject::ServiceScope;
    ///
    /// static NEXT_SEQ_NUM: AtomicUsize = AtomicUsize::new(1);
    ///
    /// #[derive(Clone)]
    /// struct Dependency { }
    ///
    /// #[injectable]
    /// impl Dependency {
    ///      pub fn new() -> Self {
    ///         Self { }
    ///     }
    /// }
    ///
    /// #[derive(Clone)]
    /// struct MyService {
    ///     dependency: Dependency,
    ///     pub seq_num: usize
    /// }
    ///
    /// impl MyService {
    ///     pub fn new(dependency: Dependency) -> Self {
    ///         Self { dependency, seq_num: NEXT_SEQ_NUM.fetch_add(1, Ordering::Relaxed) }
    ///     }
    /// }
    ///
    /// let service_provider = ServiceCollection::new()
    ///     .add_singleton::<Dependency>()
    ///     .add_transient_factory(|injector| Ok(MyService::new(injector.resolve::<Dependency>()?)))
    ///     .build();
    ///
    /// let my_service = service_provider.resolve::<MyService>().unwrap();
    ///
    /// assert_eq!(1, my_service.seq_num);
    ///
    /// let my_service = service_provider.resolve::<MyService>().unwrap();
    ///
    /// assert_eq!(2, my_service.seq_num);
    ///
    /// let service_scope = service_provider.resolve::<ServiceScope>().unwrap();
    /// let my_service = service_scope.service_provider().resolve::<MyService>().unwrap();
    ///
    /// assert_eq!(3, my_service.seq_num);
    /// ```
    #[must_use]
    pub fn add_transient_factory<TService: Send + Sync + 'static>(
        mut self,
        factory: impl Fn(&Injector) -> ConstructionResult<TService> + Send + Sync + 'static,
    ) -> Self {
        self.root_builders
            .push(Box::new(TransientContainerEntryBuilder::new(factory)));

        self
    }
}
