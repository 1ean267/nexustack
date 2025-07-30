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

pub struct ServiceCollection {
    root_builders: Vec<Box<dyn UntypedContainerEntryBuilder>>,
    scoped_builders: Vec<Box<dyn ScopedUntypedContainerEntryBuilder + Send + Sync>>,
}

impl ServiceCollection {
    pub fn new() -> Self {
        Self {
            root_builders: Vec::new(),
            scoped_builders: Vec::new(),
        }
    }

    pub fn build(self) -> ServiceProvider {
        let container_builder = ContainerBuilder::new(
            self.root_builders,
            if !self.scoped_builders.is_empty() {
                Some(self.scoped_builders)
            } else {
                None
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

    pub fn add_value<TService: Clone + Send + Sync + 'static>(mut self, value: TService) -> Self {
        self.root_builders
            .push(Box::new(SingletonContainerEntryBuilder::new(|_| Ok(value))));

        self
    }

    pub fn add_singleton<TService: Clone + Send + Sync + Injectable + 'static>(mut self) -> Self {
        self.root_builders
            .push(Box::new(SingletonContainerEntryBuilder::new(
                TService::from_injector,
            )));

        self
    }

    pub fn add_scoped<TService: Clone + Send + Sync + Injectable + 'static>(mut self) -> Self {
        self.scoped_builders
            .push(Box::new(ScopedContainerEntryBuilder::new(
                TService::from_injector,
            )));

        self
    }

    pub fn add_transient<TService: Send + Sync + Injectable + 'static>(mut self) -> Self {
        self.root_builders
            .push(Box::new(TransientContainerEntryBuilder::new(
                TService::from_injector,
            )));

        self
    }

    pub fn add_singleton_factory<TService: Clone + Send + Sync + 'static>(
        mut self,
        factory: impl FnOnce(&Injector) -> ConstructionResult<TService> + 'static,
    ) -> Self {
        self.root_builders
            .push(Box::new(SingletonContainerEntryBuilder::new(factory)));

        self
    }

    pub fn add_scoped_factory<TService: Clone + Send + Sync + 'static>(
        mut self,
        factory: impl Fn(&Injector) -> ConstructionResult<TService> + Send + Sync + 'static,
    ) -> Self {
        self.scoped_builders
            .push(Box::new(ScopedContainerEntryBuilder::new(factory)));

        self
    }

    pub fn add_transient_factory<TService: Send + Sync + 'static>(
        mut self,
        factory: impl Fn(&Injector) -> ConstructionResult<TService> + Send + Sync + 'static,
    ) -> Self {
        self.root_builders
            .push(Box::new(TransientContainerEntryBuilder::new(factory)));

        self
    }
}
