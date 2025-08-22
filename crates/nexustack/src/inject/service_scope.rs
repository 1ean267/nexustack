/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::inject::service_provider::ServiceProvider;

/// Represents a service-scope within the global service provider. A service-scope
/// can be used to access scoped services.
///
/// # Example
///
/// ```rust
/// use nexustack::inject::injectable;
/// use nexustack::inject::ServiceCollection;
/// use nexustack::inject::ServiceScope;
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
///     .add_scoped::<MyService>()
///     .build();
///
/// let service_scope = service_provider.resolve::<ServiceScope>().unwrap();
/// let my_service = service_scope.service_provider().resolve::<MyService>().unwrap();
/// ```
pub struct ServiceScope {
    service_provider: ServiceProvider,
}

impl ServiceScope {
    pub(crate) const fn new(service_provider: ServiceProvider) -> Self {
        Self { service_provider }
    }

    /// A reference to the scoped service provider.
    #[must_use]
    pub const fn service_provider(&self) -> &ServiceProvider {
        &self.service_provider
    }
}
