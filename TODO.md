# TODO

* openapi_schema: Exhaustiveness of struct should match the exhaustiveness of the rust type (controllable via the #[non_exhaustive] attribute)
* openapi_schema: Rename all
* openapi_schema: Examples cross product
* openapi_schema: discriminator
* openapi_schema: Named primitive schema
* injection: DI container owned services (internal owned type -> copy/transform function -> external type)
* injection: DI decorated services
* openapi_schema: SchemaCollection -> RefCell internal
* cron: schedule_with should be able to accept a wider range of function, also function service injection and anything that is transformable to `CronResult<Schedule>`
* cron: cron_jobs should not be forced to return `Result<(), CronError>` it should be possible to return `()` or `Result<(), {SomeError}>` where `{SomeError}` implemented `std::error::Error` such that it can be transformed into a `CronError`
* cron: Ugly naming when registering the cron feature to the application builder and adding cron_jobs to it.
* cron: Review error type
* cron: Should not require cron-jobs to be an async fn
* ci: Build check with all feature combinations
* http: Specialization on the decode error concrete type when implementing an http response from impl is ugly
* http: We need to be able to transform the openapi builder when adding custom layers to the router
* http: Can we add a way to centrally configure (per decoder) the serialization and the HTTP error codes of the error (i.e. impl IntoReponse) and the openapi description and auto apply both instead of declaring a custom transform to the HTTP response type?
* http: We need to be able to add custom cookie/header/etc. reads (and possible transforms) to the openapi descriptor when declaring decoders/encoders (possibly also other components). For example: The DefaultDecoder reads the ContentType header
