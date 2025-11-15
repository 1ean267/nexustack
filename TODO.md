# TODO

* Exhaustiveness of struct should match the exhaustiveness of the rust type (controllable via the #[non_exhaustive] attribute)
* Rename all
* Examples cross product
* discriminator
* Named primitive schema
* DI container owned services (internal owned type -> copy/transform function -> external type)
* DI decorated services
* SchemaCollection -> RefCell intern
* Restructure json submodule: Merge with specification and rename to specification. Move the build function to the openapi module??
* cron: schedule_with should be able to accept a wider range of function, also function service injection and anything that is transformable to `CronResult<Schedule>`
* cron: cron_jobs should not be forced to return `Result<(), CronError>` it should be possible to return `()` or `Result<(), {SomeError}>` where `{SomeError}` implemented `std::error::Error` such that it can be transformed into a `CronError`
* cron: Rename `CronRunner` to `Cron` as it is the feature
* cron: Ugly naming when registering the cron feature to the application builder and adding cron_jobs to it.
* cron: Review error type
* ci: Build check with all feature combinations