/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::openapi::{
    HttpContentTypeBuilder, HttpOperation, HttpOperationBuilder, HttpOperationId,
    HttpResponseBuilder, HttpSecurityRequirementBuilder, Optional, SpecificationVersion,
    error::DocumentGenerationError,
    schema::{
        generator::{JsonSchemaBuilder, SchemaCollection},
        post_process::{PostProcessSchemaBuilder, Transform},
    },
    spec,
};
use std::{borrow::Cow, cell::RefCell, collections::HashMap, rc::Rc};

pub struct KeyedOperationObject {
    method: &'static str,
    path: &'static str,
    operation: spec::OperationObject,
}

pub fn add_http_operation_to_paths(
    paths: &mut spec::PathsObject,
    operation: KeyedOperationObject,
) -> Result<(), DocumentGenerationError> {
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
        return Err(DocumentGenerationError::UnsupportedHttpMethod {
            method: operation.method,
        });
    };

    if target_operation.is_some() {
        return Err(DocumentGenerationError::DuplicateOperation {
            method: operation.method,
            path: operation.path,
        });
    }

    *target_operation = Some(Box::new(operation.operation));
    Ok(())
}

#[cfg(any())]
pub fn build_http_operation<T: HttpOperation>(
    specification: SpecificationVersion,
) -> Result<KeyedOperationObject, DocumentGenerationError> {
    let operation_builder = JsonOperationBuilder::new(specification, None);
    T::describe(operation_builder)
}

pub fn build_http_operation_with_collection<T: HttpOperation>(
    specification: SpecificationVersion,
    schema_collection: Rc<RefCell<SchemaCollection>>,
) -> Result<KeyedOperationObject, DocumentGenerationError> {
    let operation_builder = JsonOperationBuilder::new(specification, Some(schema_collection));
    T::describe(operation_builder)
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
    type Error = DocumentGenerationError;

    type ContentTypeBuilder<'a> = JsonResponseContentTypeBuilder<'a>;

    fn describe_response<'a>(
        &'a mut self,
        status_code: u16,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::ContentTypeBuilder<'a>, Self::Error> {
        if self.result.contains_key(&status_code) {
            return Err(DocumentGenerationError::DuplicateResponseDefinition { status_code });
        }

        Ok(JsonResponseContentTypeBuilder {
            parent: self,
            status_code,
            description,
            deprecated,
            content: HashMap::new(),
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }
}

struct JsonResponseContentTypeBuilder<'a> {
    parent: &'a mut JsonResponseBuilder,
    status_code: u16,
    description: Option<&'static str>,
    deprecated: bool,
    content: HashMap<Cow<'static, str>, spec::MediaTypeObject>,
}

impl<'b> HttpContentTypeBuilder for JsonResponseContentTypeBuilder<'b> {
    type Ok = ();
    type Error = DocumentGenerationError;

    type SchemaBuilder<'a>
        = PostProcessSchemaBuilder<DescribeResponseContentType<'a, 'b>, JsonSchemaBuilder>
    where
        Self: 'a;

    fn describe_content_type<'a>(
        &'a mut self,
        content_type: &'static str,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::SchemaBuilder<'a>, Self::Error> {
        if self.content.contains_key(content_type) {
            return Err(DocumentGenerationError::DuplicateContentType { content_type });
        }

        let specification = self.parent.specification;
        let schema_collection = self.parent.schema_collection.clone();

        Ok(PostProcessSchemaBuilder::new(
            DescribeResponseContentType {
                parent: self,
                content_type,
                description,
                deprecated,
            },
            JsonSchemaBuilder::new(specification, schema_collection),
        ))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let response_object = spec::ResponseObject {
            description: self.description.unwrap_or_default().into(),
            headers: None, // TODO: Support headers
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
    parent: &'a mut JsonResponseContentTypeBuilder<'b>,
    content_type: &'static str,
    description: Option<&'static str>,
    deprecated: bool,
}

impl Transform<spec::SchemaOrReferenceObject> for DescribeResponseContentType<'_, '_> {
    type Output = ();
    type Error = DocumentGenerationError;

    fn transform(
        self,
        schema: spec::SchemaOrReferenceObject,
    ) -> Result<Self::Output, DocumentGenerationError> {
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
    type Error = DocumentGenerationError;

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
        deprecated: bool,
        required: Option<bool>,
    ) -> Result<Self::RequestBodySchemaBuilder<'a>, Self::Error> {
        Ok(JsonRequestBodyContentTypeBuilder {
            parent: self,
            description,
            deprecated,
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

    fn describe_operation<T>(
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
    type Error = DocumentGenerationError;

    fn transform(
        self,
        i: (bool, spec::SchemaOrReferenceObject),
    ) -> Result<Self::Output, DocumentGenerationError> {
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
    deprecated: bool,
    required: Option<bool>,
    content: HashMap<Cow<'static, str>, spec::MediaTypeObject>,
}

impl<'b> HttpContentTypeBuilder for JsonRequestBodyContentTypeBuilder<'b> {
    type Ok = ();
    type Error = DocumentGenerationError;

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
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::SchemaBuilder<'a>, Self::Error> {
        if self.content.contains_key(content_type) {
            return Err(DocumentGenerationError::DuplicateContentType { content_type });
        }

        let specification = self.parent.specification;
        let schema_collection = self.parent.schema_collection.clone();

        Ok(PostProcessSchemaBuilder::new(
            DescribeRequestBodyContentType {
                parent: self,
                content_type,
                description,
                deprecated,
            },
            Optional::new(JsonSchemaBuilder::new(specification, schema_collection)),
        ))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let request_body_object = spec::RequestBodyObject {
            description: self.description.map(Cow::Borrowed),
            content: if self.content.is_empty() {
                return Err(DocumentGenerationError::RequestBodyMustHaveContentType);
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
    description: Option<&'static str>,
    deprecated: bool,
}

impl Transform<(bool, spec::SchemaOrReferenceObject)> for DescribeRequestBodyContentType<'_, '_> {
    type Output = ();
    type Error = DocumentGenerationError;

    fn transform(
        self,
        i: (bool, spec::SchemaOrReferenceObject),
    ) -> Result<Self::Output, DocumentGenerationError> {
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
    type Error = DocumentGenerationError;

    fn describe_requirement<S>(
        &mut self,
        name: &'static str,
        scopes: Option<S>,
    ) -> Result<(), Self::Error>
    where
        S: IntoIterator<Item = &'static str>,
    {
        if self.requirements.contains_key(name) {
            return Err(DocumentGenerationError::DuplicateSecurityRequirement { name });
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
    type Error = DocumentGenerationError;

    type ContentTypeBuilder<'a>
        = <JsonResponseBuilder as HttpResponseBuilder>::ContentTypeBuilder<'a>
    where
        Self: 'a;

    fn describe_response<'a>(
        &'a mut self,
        status_code: u16,
        description: Option<&'static str>,
        deprecated: bool,
    ) -> Result<Self::ContentTypeBuilder<'a>, Self::Error> {
        self.inner
            .describe_response(status_code, description, deprecated)
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
