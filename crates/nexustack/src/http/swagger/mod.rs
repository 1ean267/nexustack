/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::http::ConfigureRouter;
use axum::response::Redirect;
use include_directory::{Dir, include_directory};
use std::{borrow::Cow, sync::Arc};

static SWAGGER_DIST: Dir = include_directory!("$CARGO_MANIFEST_DIR/src/http/swagger/dist");

static INDEX_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8">
    <title>{title}</title>
    <link rel="stylesheet" type="text/css" href="./swagger-ui.css" />
    <link rel="stylesheet" type="text/css" href="index.css" />
    <link rel="icon" type="image/png" href="./favicon-32x32.png" sizes="32x32" />
    <link rel="icon" type="image/png" href="./favicon-16x16.png" sizes="16x16" />
  </head>

  <body>
    <div id="swagger-ui"></div>
    <script src="./swagger-ui-bundle.js" charset="UTF-8"> </script>
    <script src="./swagger-ui-standalone-preset.js" charset="UTF-8"> </script>
    <script src="./swagger-initializer.js" charset="UTF-8"> </script>
  </body>
</html>
"#;

static SWAGGER_INITIALIZER_JS: &str = r#"
window.onload = function() {
  //<editor-fold desc="Changeable Configuration Block">

  // the following lines will be replaced by docker/configurator, when it runs in a docker-container
  window.ui = SwaggerUIBundle({
    url: "{spec_url}",
    dom_id: '#swagger-ui',
    deepLinking: true,
    presets: [
      SwaggerUIBundle.presets.apis,
      SwaggerUIStandalonePreset
    ],
    plugins: [
      SwaggerUIBundle.plugins.DownloadUrl
    ],
    layout: "StandaloneLayout"
  });

  //</editor-fold>
};
"#;

pub(crate) trait SwaggerRouter {
    fn serve_swagger_ui(self, path: &str, spec_url: &str, title: &str) -> Self;
}

impl<L> SwaggerRouter for axum::Router<L>
where
    L: Clone + Send + Sync + 'static,
{
    fn serve_swagger_ui(mut self, path: &str, spec_url: &str, title: &str) -> Self {
        for entry in SWAGGER_DIST.files() {
            self = self.route(
                // TODO: Is entry.path relative or absolute to the include_directory! macro invocation?
                &format!("{path}/{}", entry.path().display()),
                axum::routing::get({
                    let data: &'static [u8] = entry.contents();
                    let mime = mime_guess::from_path(entry.path())
                        .first_or_octet_stream()
                        .essence_str()
                        .to_string();
                    async move || {
                        (
                            axum::http::StatusCode::OK,
                            [(axum::http::header::CONTENT_TYPE, mime.clone())],
                            data,
                        )
                    }
                }),
            );
        }

        self.route(
            path,
            axum::routing::get({
                let path = path.to_string();
                async move || Redirect::temporary(&format!("{path}/index.html"))
            }),
        )
        .route(
            &format!("{path}/"),
            axum::routing::get({
                let path = path.to_string();
                async move || Redirect::temporary(&format!("{path}/index.html"))
            }),
        )
        .route(
            &format!("{path}/index.html"),
            axum::routing::get({
                let title = title.to_string();
                let path = path.to_string();
                async move || {
                    (
                        axum::http::StatusCode::OK,
                        [(
                            axum::http::header::CONTENT_TYPE,
                            mime::TEXT_HTML.essence_str(),
                        )],
                        #[allow(clippy::literal_string_with_formatting_args)]
                        INDEX_HTML
                            .replace("{title}", &title)
                            .replace("{base}", &path),
                    )
                }
            }),
        )
        .route(
            &format!("{path}/swagger-initializer.js"),
            axum::routing::get({
                let spec_url = spec_url.to_string();
                async move || {
                    (
                        axum::http::StatusCode::OK,
                        [(
                            axum::http::header::CONTENT_TYPE,
                            mime::APPLICATION_JAVASCRIPT.essence_str(),
                        )],
                        #[allow(clippy::literal_string_with_formatting_args)]
                        SWAGGER_INITIALIZER_JS.replace("{spec_url}", &spec_url),
                    )
                }
            }),
        )
    }
}

pub(crate) struct OpenApiConfigurationBuilder {
    /// The `OpenAPI` document builder.
    openapi_document_builder: Option<crate::openapi::http::HttpDocumentBuilder>,
    /// The path to serve the `OpenAPI` document and Swagger UI at.
    openapi_path: Cow<'static, str>,
}

impl OpenApiConfigurationBuilder {
    pub(crate) fn new(
        openapi_document_builder: Option<crate::openapi::http::HttpDocumentBuilder>,
    ) -> Self {
        Self {
            openapi_document_builder,
            openapi_path: Cow::Borrowed("/api"),
        }
    }

    pub(crate) fn with_openapi_path(&mut self, openapi_path: Cow<'static, str>) -> &mut Self {
        self.openapi_path = openapi_path;
        self
    }

    pub(crate) fn configure_openapi_document<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::openapi::http::HttpDocumentBuilder),
    {
        if let Some(openapi_document_builder) = self.openapi_document_builder.as_mut() {
            f(openapi_document_builder);
        }

        self
    }

    pub(crate) fn with_openapi_document_builder(
        &mut self,
        openapi_document_builder: Option<crate::openapi::http::HttpDocumentBuilder>,
    ) -> &mut Self {
        self.openapi_document_builder = openapi_document_builder;
        self
    }

    pub(crate) fn build(
        self,
    ) -> Result<OpenApiConfiguration, crate::openapi::http::HttpDocumentGenerationError> {
        let openapi_document = self
            .openapi_document_builder
            .map(|openapi_document_builder| {
                crate::openapi::http::HttpDocumentBuilder::build(openapi_document_builder)
                    .map(Arc::new)
            })
            .transpose()?;

        Ok(OpenApiConfiguration {
            openapi_document,
            openapi_path: self.openapi_path,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct OpenApiConfiguration {
    /// The `OpenAPI` document.
    openapi_document: Option<Arc<crate::openapi::http::HttpDocument>>,
    /// The path to serve the `OpenAPI` document and Swagger UI at.
    openapi_path: Cow<'static, str>,
}

impl ConfigureRouter for OpenApiConfiguration {
    fn configure_router(&self, mut router: axum::Router) -> axum::Router {
        if let Some(openapi_document) = &self.openapi_document {
            let title = &openapi_document.title();
            let openapi_path = self.openapi_path.as_ref();

            router = router.route(
                &format!("{openapi_path}-json"),
                axum::routing::get({
                    let openapi_document = openapi_document.clone();
                    async move || axum::Json(openapi_document)
                }),
            );

            #[cfg(feature = "yaml")]
            {
                router = router.route(
                    &format!("{openapi_path}-yaml"),
                    axum::routing::get({
                        let openapi_document = openapi_document.clone();
                        async move || crate::http::yaml::Yaml(openapi_document)
                    }),
                );
            }

            router = router.serve_swagger_ui(openapi_path, &format!("{openapi_path}-json"), title);
        }

        router
    }
}
