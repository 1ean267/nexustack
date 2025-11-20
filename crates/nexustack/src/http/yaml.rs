/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

/*
 * Based on axum (https://github.com/tokio-rs/axum)
 *
 * MIT License
 * Copyright (c) 2019–2025 axum Contributors
 */

use axum::{
    extract::{FromRequest, OptionalFromRequest, Request, rejection::BytesRejection},
    http::{
        StatusCode,
        header::{self, HeaderMap, HeaderValue},
    },
    response::{IntoResponse, Response},
};
use serde::{Serialize, de::DeserializeOwned};
use tokio_util::bytes::{BufMut, Bytes, BytesMut};

// #[cfg(feature = "tracing")]
macro_rules! log_rejection {
    (
        rejection_type = $ty:ident,
        body_text = $body_text:expr,
        status = $status:expr,
    ) => {
        {
            tracing::event!(
                target: "axum::rejection", // TODO: change to "nexustack::http::yaml"?
                tracing::Level::TRACE,
                status = $status.as_u16(),
                body = $body_text,
                rejection_type = ::std::any::type_name::<$ty>(),
                "rejecting request",
            );
        }
    };
}

// #[cfg(not(feature = "tracing"))]
// macro_rules! log_rejection {
//     (
//         rejection_type = $ty:ident,
//         body_text = $body_text:expr,
//         status = $status:expr,
//     ) => {};
// }

macro_rules! define_rejection {
    (
        #[status = $status:ident]
        #[body = $body:literal]
        $(#[$m:meta])*
        pub struct $name:ident;
    ) => {
        $(#[$m])*
        #[derive(Debug)]
        #[non_exhaustive]
        pub struct $name;

        impl $name {
            /// Get the response body text used for this rejection.
            pub fn body_text(&self) -> String {
                self.to_string()
            }

            /// Get the status code used for this rejection.
            pub const fn status(&self) -> axum::http::StatusCode {
                axum::http::StatusCode::$status
            }
        }

        impl axum::response::IntoResponse for $name {
            fn into_response(self) -> axum::response::Response {
                let status = self.status();

                log_rejection!(
                    rejection_type = $name,
                    body_text = $body,
                    status = status,
                );
                (status, $body).into_response()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", $body)
            }
        }

        impl std::error::Error for $name {}

        impl Default for $name {
            fn default() -> Self {
                Self
            }
        }
    };

    (
        #[status = $status:ident]
        #[body = $body:literal]
        $(#[$m:meta])*
        pub struct $name:ident (Error);
    ) => {
        $(#[$m])*
        #[derive(Debug)]
        pub struct $name(pub(crate) axum::Error);

        impl $name {
            pub(crate) fn from_err<E>(err: E) -> Self
            where
                E: Into<axum::BoxError>,
            {
                Self(axum::Error::new(err))
            }

            /// Get the response body text used for this rejection.
            #[must_use]
            pub fn body_text(&self) -> String {
                self.to_string()
            }

            /// Get the status code used for this rejection.
            #[must_use]
            pub const fn status(&self) -> axum::http::StatusCode {
                axum::http::StatusCode::$status
            }
        }

        impl axum::response::IntoResponse for $name {
            fn into_response(self) -> axum::response::Response {
                let status = self.status();
                let body_text = self.body_text();

                log_rejection!(
                    rejection_type = $name,
                    body_text = body_text,
                    status = status,
                );
                (status, body_text).into_response()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str($body)?;
                f.write_str(": ")?;
                self.0.fmt(f)
            }
        }

        impl std::error::Error for $name {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                Some(&self.0)
            }
        }
    };
}

define_rejection! {
    #[status = BAD_REQUEST]
    #[body = "Failed to parse the request body as YAML or failed to deserialize the YAML body into the target type"]
    /// Rejection type for [`Yaml`].
    ///
    /// This rejection is used if the request body didn't contain syntactically valid YAML or the request body is syntactically valid YAML but couldn't be
    /// deserialized into the target type.
    pub struct YamlSyntaxOrDataError(Error);
}

define_rejection! {
    #[status = UNSUPPORTED_MEDIA_TYPE]
    #[body = "Expected request with `Content-Type: application/yaml`"]
    /// Rejection type for [`Yaml`] used if the `Content-Type`
    /// header is missing.
    pub struct MissingYamlContentType;
}

macro_rules! composite_rejection {
    (
        $(#[$m:meta])*
        pub enum $name:ident {
            $($variant:ident),+
            $(,)?
        }
    ) => {
        $(#[$m])*
        #[derive(Debug)]
        #[non_exhaustive]
        pub enum $name {
            $(
                #[allow(missing_docs)]
                $variant($variant)
            ),+
        }

        impl axum::response::IntoResponse for $name {
            fn into_response(self) -> axum::response::Response {
                match self {
                    $(
                        Self::$variant(inner) => inner.into_response(),
                    )+
                }
            }
        }

        impl $name {
            /// Get the response body text used for this rejection.
            #[must_use]
            pub fn body_text(&self) -> String {
                match self {
                    $(
                        Self::$variant(inner) => inner.body_text(),
                    )+
                }
            }

            /// Get the status code used for this rejection.
            #[must_use]
            pub fn status(&self) -> axum::http::StatusCode {
                match self {
                    $(
                        Self::$variant(inner) => inner.status(),
                    )+
                }
            }
        }

        $(
            impl From<$variant> for $name {
                fn from(inner: $variant) -> Self {
                    Self::$variant(inner)
                }
            }
        )+

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$variant(inner) => write!(f, "{inner}"),
                    )+
                }
            }
        }

        impl std::error::Error for $name {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match self {
                    $(
                        Self::$variant(inner) => inner.source(),
                    )+
                }
            }
        }
    };
}

composite_rejection! {
    /// Rejection used for [`Yaml`].
    ///
    /// Contains one variant for each way the [`Yaml`] extractor
    /// can fail.
    pub enum YamlRejection {
        YamlSyntaxOrDataError,
        MissingYamlContentType,
        BytesRejection,
    }
}

/// YAML Extractor / Response.
///
/// When used as an extractor, it can deserialize request bodies into some type that
/// implements [`serde::de::DeserializeOwned`]. The request will be rejected (and a [`YamlRejection`] will
/// be returned) if:
///
/// - The request doesn't have a `Content-Type: application/yaml` (or similar) header.
/// - The body doesn't contain syntactically valid YAML.
/// - The body contains syntactically valid YAML, but it couldn't be deserialized into the target type.
/// - Buffering the request body fails.
///
/// See [`YamlRejection`] for more details.
///
/// # Extractor example
///
/// ```rust,no_run
/// use axum::{
///     routing::post,
///     Router,
/// };
/// use serde::Deserialize;
/// use nexustack::http::yaml::Yaml;
///
/// #[derive(Deserialize)]
/// struct CreateUser {
///     email: String,
///     password: String,
/// }
///
/// async fn create_user(Yaml(payload): Yaml<CreateUser>) {
///     // payload is a `CreateUser`
/// }
///
/// let app = Router::new().route("/users", post(create_user));
/// # let _: Router = app;
/// ```
///
/// When used as a response, it can serialize any type that implements [`serde::Serialize`] to
/// `YAML`, and will automatically set `Content-Type: application/yaml` header.
///
/// If the [`Serialize`] implementation decides to fail
/// or if a map with non-string keys is used,
/// a 500 response will be issued
/// whose body is the error message in UTF-8.
///
/// # Response example
///
/// ```
/// use axum::{
///     extract::Path,
///     routing::get,
///     Router,
/// };
/// use serde::Serialize;
/// use uuid::Uuid;
/// use nexustack::http::yaml::Yaml;
///
/// #[derive(Serialize)]
/// struct User {
///     id: Uuid,
///     username: String,
/// }
///
/// async fn get_user(Path(user_id) : Path<Uuid>) -> Yaml<User> {
///     let user = find_user(user_id).await;
///     Yaml(user)
/// }
///
/// async fn find_user(user_id: Uuid) -> User {
///     // ...
///     # unimplemented!()
/// }
///
/// let app = Router::new().route("/users/{id}", get(get_user));
/// # let _: Router = app;
/// ```
#[derive(Debug, Clone, Copy, Default)]
#[must_use]
pub struct Yaml<T>(pub T);

impl<T, S> FromRequest<S> for Yaml<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = YamlRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        if !yaml_content_type(req.headers()) {
            return Err(MissingYamlContentType.into());
        }

        let bytes = Bytes::from_request(req, state).await?;
        Self::from_bytes(&bytes)
    }
}

impl<T, S> OptionalFromRequest<S> for Yaml<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = YamlRejection;

    async fn from_request(req: Request, state: &S) -> Result<Option<Self>, Self::Rejection> {
        let headers = req.headers();
        if headers.get(header::CONTENT_TYPE).is_some() {
            if yaml_content_type(headers) {
                let bytes = Bytes::from_request(req, state).await?;
                Ok(Some(Self::from_bytes(&bytes)?))
            } else {
                Err(MissingYamlContentType.into())
            }
        } else {
            Ok(None)
        }
    }
}

fn yaml_content_type(headers: &HeaderMap) -> bool {
    let Some(content_type) = headers.get(header::CONTENT_TYPE) else {
        return false;
    };

    let Ok(content_type) = content_type.to_str() else {
        return false;
    };

    let Ok(mime) = content_type.parse::<mime::Mime>() else {
        return false;
    };

    mime.type_() == "application"
        && (mime.subtype() == "yaml" || mime.suffix().is_some_and(|name| name == "yaml"))
}

macro_rules! impl_deref {
    ($ident:ident) => {
        impl<T> std::ops::Deref for $ident<T> {
            type Target = T;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<T> std::ops::DerefMut for $ident<T> {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };

    ($ident:ident: $ty:ty) => {
        impl std::ops::Deref for $ident {
            type Target = $ty;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $ident {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

impl_deref!(Yaml);

impl<T> From<T> for Yaml<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> Yaml<T>
where
    T: DeserializeOwned,
{
    /// Construct a `Yaml<T>` from a byte slice. Most users should prefer to use the `FromRequest` impl
    /// but special cases may require first extracting a `Request` into `Bytes` then optionally
    /// constructing a `Yaml<T>`.
    ///
    /// # Errors
    ///
    /// This function returns a `YamlRejection` in the following cases:
    ///
    /// - `YamlSyntaxOrDataError`: If the input bytes contain syntactically invalid YAML or the YAML cannot be deserialized into the target type.
    /// - `MissingYamlContentType`: If the `Content-Type` header is missing or does not indicate `application/yaml`.
    /// - `BytesRejection`: If there is an error extracting the request body as bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, YamlRejection> {
        let deserializer = serde_yaml_bw::Deserializer::from_slice(bytes);

        serde_path_to_error::deserialize(deserializer)
            .map_err(|err| YamlSyntaxOrDataError::from_err(err).into())
            .map(|value| Self(value))
    }
}

impl<T> IntoResponse for Yaml<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        // Extracted into separate fn so it's only compiled once for all T.
        fn make_response(buf: BytesMut, ser_result: serde_yaml_bw::Result<()>) -> Response {
            match ser_result {
                Ok(()) => (
                    [(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static("application/yaml"),
                    )],
                    buf.freeze(),
                )
                    .into_response(),
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                    )],
                    err.to_string(),
                )
                    .into_response(),
            }
        }

        // Use a small initial capacity of 128 bytes like serde_json::to_vec
        // https://docs.rs/serde_json/1.0.82/src/serde_json/ser.rs.html#2189
        let mut buf = BytesMut::with_capacity(128).writer();
        let res = serde_yaml_bw::to_writer(&mut buf, &self.0);
        make_response(buf.into_inner(), res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        routing::post,
        test_helpers::{RequestBuilder, TestClient},
    };
    use serde::Deserialize;
    use serde_yaml_bw::{Mapping, Value};

    trait YamlRequestBuilder {
        type RequestBuilder;

        fn yaml<T>(self, yaml: &T) -> Self::RequestBuilder
        where
            T: serde::Serialize;
    }

    impl YamlRequestBuilder for RequestBuilder {
        type RequestBuilder = Self;

        fn yaml<T>(mut self, yaml: &T) -> Self::RequestBuilder
        where
            T: serde::Serialize,
        {
            let mut body = Vec::with_capacity(128);
            serde_yaml_bw::to_writer(&mut body, yaml).unwrap();

            self = self.header("content-type", "application/yaml");
            self = self.body(body);
            self
        }
    }

    #[tokio::test]
    async fn deserialize_body() {
        #[derive(Debug, Deserialize)]
        struct Input {
            foo: String,
        }

        let app = Router::new().route("/", post(|input: Yaml<Input>| async { input.0.foo }));

        let client = TestClient::new(app);
        let res = client
            .post("/")
            .yaml(&Value::Mapping({
                let mut mapping = Mapping::new();
                mapping.insert("foo".into(), "bar".into());
                mapping
            }))
            .await;
        let body = res.text().await;

        pretty_assertions::assert_eq!(body, "bar");
    }

    #[tokio::test]
    async fn consume_body_to_yaml_requires_yaml_content_type() {
        #[derive(Debug, Deserialize)]
        struct Input {
            foo: String,
        }

        let app = Router::new().route("/", post(|input: Yaml<Input>| async { input.0.foo }));

        let client = TestClient::new(app);
        let res = client.post("/").body(r#"{ "foo": "bar" }"#).await;

        let status = res.status();

        pretty_assertions::assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn yaml_content_types() {
        async fn valid_yaml_content_type(content_type: &str) -> bool {
            println!("testing {content_type:?}");

            let app = Router::new().route("/", post(|_input: Yaml<Value>| async {}));

            let res = TestClient::new(app)
                .post("/")
                .header("content-type", content_type)
                .body("{}")
                .await;

            res.status() == StatusCode::OK
        }

        assert!(valid_yaml_content_type("application/yaml").await);
        assert!(valid_yaml_content_type("application/yaml; charset=utf-8").await);
        assert!(valid_yaml_content_type("application/yaml;charset=utf-8").await);
        assert!(valid_yaml_content_type("application/cloudevents+yaml").await);
        assert!(!valid_yaml_content_type("text/yaml").await);
    }

    #[tokio::test]
    async fn invalid_yaml_syntax() {
        let app = Router::new().route("/", post(|_: Yaml<serde_yaml_bw::Value>| async {}));

        let client = TestClient::new(app);
        let res = client
            .post("/")
            .body("{")
            .header("content-type", "application/yaml")
            .await;

        pretty_assertions::assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn extra_chars_after_valid_yaml_syntax() {
        #[derive(Debug, Deserialize)]
        struct Input {
            foo: String,
        }

        let app = Router::new().route("/", post(|input: Yaml<Input>| async { input.0.foo }));

        let client = TestClient::new(app);
        let res = client
            .post("/")
            .body(r#"{ "foo": "bar" } baz "#)
            .header("content-type", "application/yaml")
            .await;

        pretty_assertions::assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let body_text = res.text().await;
        pretty_assertions::assert_eq!(
            body_text,
            "Failed to parse the request body as YAML: trailing characters at line 1 column 18"
        );
    }

    #[derive(Deserialize)]
    struct Foo {
        #[allow(dead_code)]
        a: i32,
        #[allow(dead_code)]
        b: Vec<Bar>,
    }

    #[derive(Deserialize)]
    struct Bar {
        #[allow(dead_code)]
        x: i32,
        #[allow(dead_code)]
        y: i32,
    }

    #[tokio::test]
    async fn invalid_yaml_data() {
        let app = Router::new().route("/", post(|_: Yaml<Foo>| async {}));

        let client = TestClient::new(app);
        let res = client
            .post("/")
            .body("{\"a\": 1, \"b\": [{\"x\": 2}]}")
            .header("content-type", "application/yaml")
            .await;

        pretty_assertions::assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body_text = res.text().await;
        pretty_assertions::assert_eq!(
            body_text,
            "Failed to deserialize the YAML body into the target type: b[0]: missing field `y` at line 1 column 23"
        );
    }
}
