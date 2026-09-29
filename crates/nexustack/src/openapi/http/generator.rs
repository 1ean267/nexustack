/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::openapi::{
    Optional, SchemaGenerationError, SpecificationVersion,
    http::{
        CookieSameSite, HttpContentTypeBuilder, HttpDocumentGenerationError, HttpOperation,
        HttpOperationBuilder, HttpOperationId, HttpResponseBuilder, HttpSecurityRequirementBuilder,
        response::HttpStatusResponseBuilder,
    },
    schema::{
        generator::{JsonSchemaBuilder, SchemaCollection},
        post_process::{PostProcessSchemaBuilder, Transform},
        string::{StringSchema, StringSchemaBuilder},
    },
    spec,
};
use serde_json::Value as JsonValue;
use std::{borrow::Cow, cell::RefCell, collections::HashMap, fmt::Write, rc::Rc};

pub struct KeyedOperationObject {
    method: &'static str,
    path: &'static str,
    operation: spec::OperationObject,
}

pub fn add_http_operation_to_paths(
    paths: &mut spec::PathsObject,
    operation: KeyedOperationObject,
) -> Result<(), HttpDocumentGenerationError> {
    let path_item = paths
        .0
        .entry(Cow::Borrowed(operation.path))
        .or_insert_with(|| spec::PathItemObject {
            r#ref: None,
            summary: None,
            description: None,
            get: None,
            put: None,
            post: None,
            delete: None,
            options: None,
            head: None,
            patch: None,
            trace: None,
            servers: None,
            parameters: None,
        });

    let target_operation = if operation.method.eq_ignore_ascii_case("get") {
        &mut path_item.get
    } else if operation.method.eq_ignore_ascii_case("put") {
        &mut path_item.put
    } else if operation.method.eq_ignore_ascii_case("post") {
        &mut path_item.post
    } else if operation.method.eq_ignore_ascii_case("delete") {
        &mut path_item.delete
    } else if operation.method.eq_ignore_ascii_case("options") {
        &mut path_item.options
    } else if operation.method.eq_ignore_ascii_case("head") {
        &mut path_item.head
    } else if operation.method.eq_ignore_ascii_case("patch") {
        &mut path_item.patch
    } else if operation.method.eq_ignore_ascii_case("trace") {
        &mut path_item.trace
    } else {
        return Err(HttpDocumentGenerationError::UnsupportedHttpMethod {
            method: operation.method.into(),
        });
    };

    if target_operation.is_some() {
        return Err(HttpDocumentGenerationError::DuplicateOperation {
            method: operation.method.into(),
            path: operation.path.into(),
        });
    }

    *target_operation = Some(Box::new(operation.operation));
    Ok(())
}

#[cfg(any())]
pub fn build_http_operation<T: HttpOperation>(
    specification: SpecificationVersion,
    context: <T as HttpOperation>::Context,
) -> Result<KeyedOperationObject, HttpDocumentGenerationError> {
    let operation_builder = JsonOperationBuilder::new(specification, None);
    T::describe(operation_builder, context)
}

pub fn build_http_operation_with_collection<T: HttpOperation>(
    specification: SpecificationVersion,
    schema_collection: Rc<RefCell<SchemaCollection>>,
    context: <T as HttpOperation>::Context,
) -> Result<KeyedOperationObject, HttpDocumentGenerationError> {
    let operation_builder = JsonOperationBuilder::new(specification, Some(schema_collection));
    T::describe(operation_builder, context)
}

struct JsonResponseBuilder {
    specification: SpecificationVersion,
    schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
    result: HashMap<u16, spec::ResponseObject>,
}

impl JsonResponseBuilder {
    fn new(
        specification: SpecificationVersion,
        schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
    ) -> Self {
        Self {
            specification,
            schema_collection,
            result: HashMap::new(),
        }
    }
}

impl HttpResponseBuilder for JsonResponseBuilder {
    type Ok = HashMap<u16, spec::ResponseObject>;
    type Error = HttpDocumentGenerationError;

    type StatusResponseBuilder<'a> = JsonStatusResponseBuilder<'a>;

    fn describe_status_response<'a>(
        &'a mut self,
        status_code: u16,
        description: Option<&'static str>,
        _deprecated: bool, // TODO: Use me
    ) -> Result<Self::StatusResponseBuilder<'a>, Self::Error> {
        if self.result.contains_key(&status_code) {
            return Err(HttpDocumentGenerationError::DuplicateResponseDefinition { status_code });
        }

        Ok(JsonStatusResponseBuilder {
            parent: self,
            status_code,
            description,
            content: HashMap::new(),
            headers: HashMap::new(),
            cookies: HashMap::new(),
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }
}

struct CookieAttributes {
    domain: Option<Cow<'static, str>>,
    path: Option<Cow<'static, str>>,
    max_age: Option<std::time::Duration>,
    same_site: CookieSameSite,
    http_only: bool,
    secure: bool,
    partitioned: bool,
}

struct Cookie {
    attributes: CookieAttributes,
    description: Option<&'static str>,
    pattern: Option<Cow<'static, str>>,
    examples: Vec<String>,
    deprecated: bool,
    required: bool,
}

struct JsonStatusResponseBuilder<'a> {
    parent: &'a mut JsonResponseBuilder,
    status_code: u16,
    description: Option<&'static str>,
    content: HashMap<Cow<'static, str>, spec::MediaTypeObject>,
    headers: HashMap<http::header::HeaderName, spec::HeaderOrReferenceObject>,
    cookies: HashMap<Cow<'static, str>, Cookie>,
}

impl<'b> HttpContentTypeBuilder for JsonStatusResponseBuilder<'b> {
    type Ok = ();
    type SchemaBuilderError = SchemaGenerationError;
    type Error = HttpDocumentGenerationError;

    type SchemaBuilder<'a>
        = PostProcessSchemaBuilder<DescribeResponseContentType<'a, 'b>, JsonSchemaBuilder>
    where
        Self: 'a;

    fn describe_content_type<'a>(
        &'a mut self,
        content_type: &'static str,
        _description: Option<&'static str>, // TODO: Use me
        _deprecated: bool,                  // TODO: Use me
    ) -> Result<Self::SchemaBuilder<'a>, Self::Error> {
        if self.content.contains_key(content_type) {
            return Err(HttpDocumentGenerationError::DuplicateContentType {
                content_type: content_type.into(),
            });
        }

        let specification = self.parent.specification;
        let schema_collection = self.parent.schema_collection.clone();

        Ok(PostProcessSchemaBuilder::new(
            DescribeResponseContentType {
                parent: self,
                content_type,
            },
            JsonSchemaBuilder::new(specification, schema_collection),
        ))
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        fn fmt_serialized_value(
            cookie_attrs: &CookieAttributes,
            name: &str,
            example: &str,
        ) -> Result<String, HttpDocumentGenerationError> {
            let mut result = format!("{name}={example}");

            if cookie_attrs.http_only {
                result.push_str("; HttpOnly");
            }

            if cookie_attrs.partitioned {
                result.push_str("; Partitioned");
            }

            if cookie_attrs.secure {
                result.push_str("; Secure");
            }

            if cookie_attrs.partitioned {
                result.push_str("; Partitioned");
            }

            if let Some(domain) = &cookie_attrs.domain {
                result.push_str("; Domain=");
                result.push_str(domain);
            }

            if let Some(path) = &cookie_attrs.path {
                result.push_str("; Path=");
                result.push_str(path);
            }

            if let Some(max_age) = cookie_attrs.max_age {
                write!(result, "; Max-Age={}", max_age.as_secs_f64()).expect("displaying an f64 only forwards errors of the underlying buffer, String as a buffer never errors.");
            }

            match cookie_attrs.same_site {
                CookieSameSite::None if cookie_attrs.secure => result.push_str("; SameSite=None"),
                CookieSameSite::Strict => result.push_str("; SameSite=Strict"),
                _ => {}
            }

            Ok(result)
        }

        let specification = self.parent.specification;

        if self.cookies.len() == 1 {
            let (cookie_name, cookie) = self
                .cookies
                .into_iter()
                .next()
                .expect("Checked len, there must be one entry");

            let Cookie {
                examples: cookie_examples,
                attributes: cookie_attrs,
                ..
            } = cookie;

            let examples = cookie_examples
                .into_iter()
                .enumerate()
                .map(|(num, example)| {
                    Ok((
                        format!("example_{}", num + 1).into(),
                        match specification {
                            SpecificationVersion::OpenAPI3_0 | SpecificationVersion::OpenAPI3_1 => {
                                spec::ExampleObject::Value {
                                    summary: None,
                                    description: None,
                                    value: JsonValue::String(example),
                                }
                            }
                            SpecificationVersion::OpenAPI3_2 => {
                                let serialized_value =
                                    fmt_serialized_value(&cookie_attrs, &cookie_name, &example)?
                                        .into();

                                spec::ExampleObject::SerializedValue {
                                    summary: None,
                                    description: None,
                                    data_value: JsonValue::String(example),
                                    serialized_value,
                                }
                            }
                        }
                        .into(),
                    ))
                })
                .collect::<Result<
                    HashMap<Cow<'static, str>, spec::ExampleOrReferenceObject>,
                    HttpDocumentGenerationError,
                >>()?;

            let mut schema = spec::schema! {
                r#type: "string".into(),
            };

            schema.pattern = cookie.pattern;

            self.headers
                .insert(
                    http::header::SET_COOKIE,
                    spec::HeaderObject::Schema {
                        description: cookie.description.map(Into::into),
                        required: cookie.required,
                        deprecated: cookie.deprecated,
                        style: None,
                        explode: None,
                        schema: schema.into(),
                        example: None,
                        examples: examples,
                    }
                    .into(),
                )
                .expect("We did not allow set-cookie header to be set via describe_header");
        } else if self.cookies.len() > 1 {
            let required = self.cookies.values().any(|cookie| cookie.required);
            let deprecated = self.cookies.values().all(|cookie| cookie.deprecated);
            let prefix_items = self
                .cookies
                .values()
                .map(|cookie| {
                    let mut schema = spec::schema! {
                        r#type: "string".into(),
                    };

                    schema.pattern = cookie.pattern.clone();

                    spec::BoxSchemaOrReferenceObject::from(schema)
                })
                .collect::<Vec<_>>();

            let schema = match specification {
                SpecificationVersion::OpenAPI3_0 => {
                    spec::schema! {
                        r#type: "array".into(),
                        items: spec::schema! {
                            one_of: prefix_items
                        }.into(),
                        min_items: self.cookies.len().into(),
                        max_items: self.cookies.len().into(),
                    }
                }
                SpecificationVersion::OpenAPI3_1 | SpecificationVersion::OpenAPI3_2 => {
                    spec::schema! {
                        r#type: "array".into(),
                        prefix_items: prefix_items,
                        min_items: self.cookies.len().into(),
                        max_items: self.cookies.len().into(),
                    }
                }
            };

            let examples = self
                .cookies
                .into_iter()
                .flat_map(|(cookie_name, cookie)| {
                    let Cookie {
                        examples: cookie_examples,
                        attributes: cookie_attrs,
                        ..
                    } = cookie;

                    cookie_examples
                        .into_iter()
                        .enumerate()
                        .map(move |(num, example)| {
                            Ok((
                                format!("example_{}_{}", cookie_name, num + 1).into(),
                                match specification {
                                    SpecificationVersion::OpenAPI3_0
                                    | SpecificationVersion::OpenAPI3_1 => {
                                        spec::ExampleObject::Value {
                                            summary: None,
                                            description: None,
                                            value: JsonValue::String(example),
                                        }
                                    }
                                    SpecificationVersion::OpenAPI3_2 => {
                                        let serialized_value = fmt_serialized_value(
                                            &cookie_attrs,
                                            &cookie_name,
                                            &example,
                                        )?
                                        .into();

                                        spec::ExampleObject::SerializedValue {
                                            summary: None,
                                            description: None,
                                            data_value: JsonValue::String(example),
                                            serialized_value,
                                        }
                                    }
                                }
                                .into(),
                            ))
                        })
                })
                .collect::<Result<
                    HashMap<Cow<'static, str>, spec::ExampleOrReferenceObject>,
                    HttpDocumentGenerationError,
                >>()?;

            self.headers
                .insert(
                    http::header::SET_COOKIE,
                    spec::HeaderObject::Schema {
                        description: Some("Note: The server will issue multiple physical Set-Cookie headers on the wire (one for each cookie object listed in the example data).".into()),
                        required,
                        deprecated,
                        style: None,
                        explode: None,
                        schema: schema.into(),
                        example: None,
                        examples,
                    }
                    .into(),
                )
                .expect("We did not allow set-cookie header to be set via describe_header");
        }

        let response_object = spec::ResponseObject {
            description: self.description.unwrap_or_default().into(),
            headers: self.headers,
            content: if self.content.is_empty() {
                None
            } else {
                Some(self.content)
            },
            links: None,
            // TODO
            // deprecated: if self.deprecated { Some(true) } else { None },
        };

        self.parent.result.insert(self.status_code, response_object);

        Ok(())
    }
}

struct DescribeResponseContentType<'a, 'b> {
    parent: &'a mut JsonStatusResponseBuilder<'b>,
    content_type: &'static str,
}

impl Transform<spec::SchemaOrReferenceObject> for DescribeResponseContentType<'_, '_> {
    type Output = ();
    type Error = SchemaGenerationError;

    fn transform(
        self,
        schema: spec::SchemaOrReferenceObject,
    ) -> Result<Self::Output, SchemaGenerationError> {
        let media_type_object = spec::MediaTypeObject {
            schema: Some(schema),
            example: None,
            examples: None,
            encoding: None,
        };

        self.parent
            .content
            .insert(Cow::Borrowed(self.content_type), media_type_object);

        Ok(())
    }
}

impl<'b> HttpStatusResponseBuilder for JsonStatusResponseBuilder<'b> {
    type ResponseCookieSchemaBuilder<'a>
        = PostProcessSchemaBuilder<DescribeResponseCookie<'a, 'b>, StringSchemaBuilder>
    where
        Self: 'a;

    type ResponseHeaderSchemaBuilder<'a>
        = PostProcessSchemaBuilder<DescribeResponseHeader<'a, 'b>, StringSchemaBuilder>
    where
        Self: 'a;

    fn describe_response_cookie<'a>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        domain: Option<Cow<'static, str>>,
        path: Option<Cow<'static, str>>,
        max_age: Option<std::time::Duration>,
        same_site: CookieSameSite,
        deprecated: bool,
        required: bool,
        http_only: bool,
        secure: bool,
        partitioned: bool,
    ) -> Result<Self::ResponseCookieSchemaBuilder<'a>, Self::Error> {
        if self.cookies.contains_key(&Cow::Borrowed(name)) {
            return Err(HttpDocumentGenerationError::DuplicateResponseCookie { name: name.into() });
        }

        Ok(PostProcessSchemaBuilder::new(
            DescribeResponseCookie {
                parent: self,
                name,
                description,
                domain,
                path,
                max_age,
                same_site,
                deprecated,
                required,
                http_only,
                secure,
                partitioned,
            },
            StringSchemaBuilder,
        ))
    }

    fn describe_response_header<'a>(
        &'a mut self,
        name: http::header::HeaderName,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
    ) -> Result<Self::ResponseHeaderSchemaBuilder<'a>, Self::Error> {
        if name == http::header::CONTENT_TYPE || name == http::header::SET_COOKIE {
            return Err(HttpDocumentGenerationError::IllegalResponseHeader { name });
        }

        if self.headers.contains_key(&name) {
            return Err(HttpDocumentGenerationError::DuplicateResponseHeader { name });
        }

        Ok(PostProcessSchemaBuilder::new(
            DescribeResponseHeader {
                parent: self,
                name,
                description,
                deprecated,
                required,
            },
            StringSchemaBuilder,
        ))
    }
}

struct DescribeResponseCookie<'a, 'b> {
    parent: &'a mut JsonStatusResponseBuilder<'b>,
    name: &'static str,
    description: Option<&'static str>,
    domain: Option<Cow<'static, str>>,
    path: Option<Cow<'static, str>>,
    max_age: Option<std::time::Duration>,
    same_site: super::CookieSameSite,
    deprecated: bool,
    required: bool,
    http_only: bool,
    secure: bool,
    partitioned: bool,
}

impl Transform<StringSchema> for DescribeResponseCookie<'_, '_> {
    type Output = ();
    type Error = SchemaGenerationError;

    fn transform(self, schema: StringSchema) -> Result<Self::Output, Self::Error> {
        let StringSchema { pattern, examples } = schema;

        let cookie_object = Cookie {
            attributes: CookieAttributes {
                domain: self.domain,
                path: self.path,
                max_age: self.max_age,
                same_site: self.same_site,
                http_only: self.http_only,
                secure: self.secure,
                partitioned: self.partitioned,
            },
            description: self.description,
            pattern,
            examples,
            deprecated: self.deprecated,
            required: self.required,
        };

        self.parent.cookies.insert(self.name.into(), cookie_object);

        Ok(())
    }
}

struct DescribeResponseHeader<'a, 'b> {
    parent: &'a mut JsonStatusResponseBuilder<'b>,
    name: http::header::HeaderName,
    description: Option<&'static str>,
    deprecated: bool,
    required: Option<bool>,
}

impl Transform<StringSchema> for DescribeResponseHeader<'_, '_> {
    type Output = ();
    type Error = SchemaGenerationError;

    fn transform(self, schema: StringSchema) -> Result<Self::Output, Self::Error> {
        let StringSchema { pattern, examples } = schema;

        let mut schema = spec::schema! { r#type: "string".into() };
        schema.pattern = pattern;

        let examples: HashMap<Cow<'static, str>, spec::ExampleOrReferenceObject> = examples
            .into_iter()
            .enumerate()
            .map(|(num, example)| {
                (
                    format!("example_{}", num + 1).into(),
                    match self.parent.parent.specification {
                        SpecificationVersion::OpenAPI3_0 | SpecificationVersion::OpenAPI3_1 => {
                            spec::ExampleObject::Value {
                                summary: None,
                                description: None,
                                value: JsonValue::String(example),
                            }
                        }
                        SpecificationVersion::OpenAPI3_2 => spec::ExampleObject::SerializedValue {
                            summary: None,
                            description: None,
                            data_value: JsonValue::String(example.clone()),
                            serialized_value: example.into(),
                        },
                    }
                    .into(),
                )
            })
            .collect();

        let header_object = spec::HeaderObject::Schema {
            description: self.description.map(Into::into),
            required: self.required.unwrap_or(false), // TODO: Is this correct?
            deprecated: self.deprecated,
            style: None,
            explode: None,
            schema: schema.into(),
            example: None,
            examples,
        };

        self.parent
            .headers
            .insert(self.name.into(), header_object.into());

        Ok(())
    }
}

struct JsonOperationBuilder {
    specification: SpecificationVersion,
    schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
    parameters: Option<Vec<spec::ParameterOrReferenceObject>>,
    request_body: Option<spec::RequestBodyOrReferenceObject>,
    security: Option<spec::SecurityRequirements>,
}

impl JsonOperationBuilder {
    const fn new(
        specification: SpecificationVersion,
        schema_collection: Option<Rc<RefCell<SchemaCollection>>>,
    ) -> Self {
        Self {
            specification,
            schema_collection,
            parameters: None,
            request_body: None,
            security: None,
        }
    }
}

impl HttpOperationBuilder for JsonOperationBuilder {
    type Ok = KeyedOperationObject;
    type SchemaBuilderError = SchemaGenerationError;
    type Error = HttpDocumentGenerationError;

    type ParameterSchemaBuilder<'a>
        = PostProcessSchemaBuilder<DescribeParameter<'a>, Optional<JsonSchemaBuilder>>
    where
        Self: 'a;

    type RequestBodySchemaBuilder<'a>
        = JsonRequestBodyContentTypeBuilder<'a>
    where
        Self: 'a;

    type SecurityRequirementBuilder<'a>
        = JsonSecurityRequirementBuilder<'a>
    where
        Self: 'a;

    type HttpResponseBuilder = DescribeOperation;

    fn describe_query_parameter<'a>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
    ) -> Result<Self::ParameterSchemaBuilder<'a>, Self::Error> {
        let specification = self.specification;
        let schema_collection = self.schema_collection.clone();

        Ok(PostProcessSchemaBuilder::new(
            DescribeParameter {
                parent: self,
                name,
                location: spec::ParameterLocation::Query,
                description,
                deprecated,
                required,
            },
            Optional::new(JsonSchemaBuilder::new(specification, schema_collection)),
        ))
    }

    fn describe_header_parameter<'a>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
    ) -> Result<Self::ParameterSchemaBuilder<'a>, Self::Error> {
        let specification = self.specification;
        let schema_collection = self.schema_collection.clone();

        Ok(PostProcessSchemaBuilder::new(
            DescribeParameter {
                parent: self,
                name,
                location: spec::ParameterLocation::Header,
                description,
                deprecated,
                required,
            },
            Optional::new(JsonSchemaBuilder::new(specification, schema_collection)),
        ))
    }

    fn describe_path_parameter<'a>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::ParameterSchemaBuilder<'a>, Self::Error> {
        let specification = self.specification;
        let schema_collection = self.schema_collection.clone();

        Ok(PostProcessSchemaBuilder::new(
            DescribeParameter {
                parent: self,
                name,
                location: spec::ParameterLocation::Path,
                description,
                deprecated,
                required: Some(true),
            },
            Optional::new(JsonSchemaBuilder::new(specification, schema_collection)),
        ))
    }

    fn describe_cookie_parameter<'a>(
        &'a mut self,
        name: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
        required: Option<bool>,
    ) -> Result<Self::ParameterSchemaBuilder<'a>, Self::Error> {
        let specification = self.specification;
        let schema_collection = self.schema_collection.clone();

        Ok(PostProcessSchemaBuilder::new(
            DescribeParameter {
                parent: self,
                name,
                location: spec::ParameterLocation::Cookie,
                description,
                deprecated,
                required,
            },
            Optional::new(JsonSchemaBuilder::new(specification, schema_collection)),
        ))
    }

    fn describe_request_body<'a>(
        &'a mut self,
        description: Option<&'static str>,
        _deprecated: bool, // TODO: Use me
        required: Option<bool>,
    ) -> Result<Self::RequestBodySchemaBuilder<'a>, Self::Error> {
        Ok(JsonRequestBodyContentTypeBuilder {
            parent: self,
            description,
            required,
            content: HashMap::new(),
        })
    }

    fn describe_security_requirement(
        &mut self,
    ) -> Result<Self::SecurityRequirementBuilder<'_>, Self::Error> {
        Ok(JsonSecurityRequirementBuilder {
            parent: self,
            requirements: HashMap::new(),
        })
    }

    fn describe_response<T>(
        self,
        id: HttpOperationId,
        method: &'static str,
        path: &'static str,
        tags: Option<T>,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::HttpResponseBuilder, Self::Error>
    where
        T: IntoIterator<Item = &'static str>,
    {
        let specification = self.specification;
        let schema_collection = self.schema_collection.clone();

        Ok(DescribeOperation {
            parent: self,
            inner: JsonResponseBuilder::new(specification, schema_collection),
            id,
            method,
            path,
            tags: tags.map(|t| t.into_iter().map(Cow::Borrowed).collect()),
            description,
            deprecated,
        })
    }
}

struct DescribeParameter<'a> {
    parent: &'a mut JsonOperationBuilder,
    name: &'static str,
    location: spec::ParameterLocation,
    description: Option<&'static str>,
    deprecated: bool,
    required: Option<bool>,
}

impl Transform<(bool, spec::SchemaOrReferenceObject)> for DescribeParameter<'_> {
    type Output = ();
    type Error = SchemaGenerationError;

    fn transform(
        self,
        i: (bool, spec::SchemaOrReferenceObject),
    ) -> Result<Self::Output, SchemaGenerationError> {
        let (is_optional, schema) = i;

        let parameter_object = spec::ParameterObject::Schema {
            name: Cow::Borrowed(self.name),
            r#in: self.location,
            description: self.description.map(Cow::Borrowed),
            required: self.required.unwrap_or(!is_optional),
            deprecated: self.deprecated,
            allow_empty_value: None,
            style: None,
            explode: None,
            allow_reserved: None,
            schema: Some(schema.into()),
            example: None,
            examples: None,
        };

        if let Some(params) = &mut self.parent.parameters {
            params.push(spec::ParameterOrReferenceObject::Parameter(
                parameter_object,
            ));
        } else {
            self.parent.parameters = Some(vec![spec::ParameterOrReferenceObject::Parameter(
                parameter_object,
            )]);
        }

        Ok(())
    }
}

struct JsonRequestBodyContentTypeBuilder<'a> {
    parent: &'a mut JsonOperationBuilder,
    description: Option<&'static str>,
    required: Option<bool>,
    content: HashMap<Cow<'static, str>, spec::MediaTypeObject>,
}

impl<'b> HttpContentTypeBuilder for JsonRequestBodyContentTypeBuilder<'b> {
    type Ok = ();
    type Error = HttpDocumentGenerationError;
    type SchemaBuilderError = SchemaGenerationError;

    type SchemaBuilder<'a>
        = PostProcessSchemaBuilder<
        DescribeRequestBodyContentType<'a, 'b>,
        Optional<JsonSchemaBuilder>,
    >
    where
        Self: 'a;

    fn describe_content_type<'a>(
        &'a mut self,
        content_type: &'static str,
        _description: Option<&'static str>, // TODO: Use me
        _deprecated: bool,                  // TODO: Use me
    ) -> Result<Self::SchemaBuilder<'a>, Self::Error> {
        if self.content.contains_key(content_type) {
            return Err(HttpDocumentGenerationError::DuplicateContentType {
                content_type: content_type.into(),
            });
        }

        let specification = self.parent.specification;
        let schema_collection = self.parent.schema_collection.clone();

        Ok(PostProcessSchemaBuilder::new(
            DescribeRequestBodyContentType {
                parent: self,
                content_type,
            },
            Optional::new(JsonSchemaBuilder::new(specification, schema_collection)),
        ))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let request_body_object = spec::RequestBodyObject {
            description: self.description.map(Cow::Borrowed),
            content: if self.content.is_empty() {
                return Err(HttpDocumentGenerationError::RequestBodyMustHaveContentType);
            } else {
                self.content
            },
            required: self.required.unwrap_or(false),
        };

        self.parent.request_body = Some(spec::RequestBodyOrReferenceObject::RequestBody(
            request_body_object,
        ));

        Ok(())
    }
}

struct DescribeRequestBodyContentType<'a, 'b> {
    parent: &'a mut JsonRequestBodyContentTypeBuilder<'b>,
    content_type: &'static str,
}

impl Transform<(bool, spec::SchemaOrReferenceObject)> for DescribeRequestBodyContentType<'_, '_> {
    type Output = ();
    type Error = SchemaGenerationError;

    fn transform(
        self,
        i: (bool, spec::SchemaOrReferenceObject),
    ) -> Result<Self::Output, SchemaGenerationError> {
        let (is_optional, schema) = i;
        let media_type_object = spec::MediaTypeObject {
            schema: Some(schema),
            example: None,
            examples: None,
            encoding: None,
        };

        self.parent
            .content
            .insert(Cow::Borrowed(self.content_type), media_type_object);

        // This is a workaround to set the request body as required if any of its content types are required
        // if not specified explicitly.
        if !is_optional && self.parent.required.is_none() {
            self.parent.required = Some(true);
        }

        Ok(())
    }
}

struct JsonSecurityRequirementBuilder<'a> {
    parent: &'a mut JsonOperationBuilder,
    requirements: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
}

impl HttpSecurityRequirementBuilder for JsonSecurityRequirementBuilder<'_> {
    type Ok = ();
    type Error = HttpDocumentGenerationError;

    fn describe_requirement<S>(
        &mut self,
        name: &'static str,
        scopes: Option<S>,
    ) -> Result<(), Self::Error>
    where
        S: IntoIterator<Item = &'static str>,
    {
        if self.requirements.contains_key(name) {
            return Err(HttpDocumentGenerationError::DuplicateSecurityRequirement {
                name: name.into(),
            });
        }

        let scopes = scopes.map_or_else(Vec::new, |s| s.into_iter().map(Cow::Borrowed).collect());

        self.requirements.insert(Cow::Borrowed(name), scopes);

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if let Some(security) = &mut self.parent.security {
            security.push(self.requirements);
        } else {
            self.parent.security = Some(vec![self.requirements]);
        }

        Ok(())
    }
}

struct DescribeOperation {
    parent: JsonOperationBuilder,
    inner: JsonResponseBuilder,
    id: HttpOperationId,
    method: &'static str,
    path: &'static str,
    tags: Option<Vec<Cow<'static, str>>>,
    description: Option<&'static str>,
    deprecated: bool,
}

impl HttpResponseBuilder for DescribeOperation {
    type Ok = KeyedOperationObject;
    type Error = HttpDocumentGenerationError;

    type StatusResponseBuilder<'a>
        = <JsonResponseBuilder as HttpResponseBuilder>::StatusResponseBuilder<'a>
    where
        Self: 'a;

    fn describe_status_response<'a>(
        &'a mut self,
        status_code: u16,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::StatusResponseBuilder<'a>, Self::Error> {
        self.inner
            .describe_status_response(status_code, description, deprecated)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let responses = self.inner.end()?;

        let operation = spec::OperationObject {
            tags: self.tags,
            summary: None,
            description: self.description.map(Cow::Borrowed),
            external_docs: None,
            operation_id: Some(std::borrow::Cow::Borrowed(self.id.name())),
            parameters: self.parent.parameters,
            request_body: self.parent.request_body,
            responses,
            callbacks: None,
            deprecated: self.deprecated,
            security: self.parent.security,
            servers: None,
        };

        Ok(KeyedOperationObject {
            operation,
            method: self.method,
            path: self.path,
        })
    }
}
