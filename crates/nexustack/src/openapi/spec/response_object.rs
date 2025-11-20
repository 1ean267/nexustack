/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use super::{HeaderOrReferenceObject, LinkOrReferenceObject, MediaTypeObject, ReferenceObject};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

/// Describes a single response from an API Operation, including design-time, static links to operations based on the response.
/// See <https://swagger.io/specification/#response-object>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseObject {
    /// REQUIRED. A description of the response.
    /// `CommonMark` syntax MAY be used for rich text representation.
    #[serde(rename = "description")]
    pub description: Cow<'static, str>,

    /// Maps a header name to its definition. RFC7230 states header names are case insensitive.
    /// If a response header is defined with the name "Content-Type", it SHALL be ignored.
    #[serde(
        rename = "headers",
        default,
        skip_serializing_if = "HashMap::is_empty",
        with = "serde_headers"
    )]
    pub headers: HashMap<http::header::HeaderName, HeaderOrReferenceObject>,

    /// A map containing descriptions of potential response payloads.
    /// The key is a media type or media type range and the value describes it.
    /// For responses that match multiple keys, only the most specific key is applicable.
    /// e.g. text/plain overrides text/*
    #[serde(rename = "content", default, skip_serializing_if = "Option::is_none")]
    pub content: Option<HashMap<Cow<'static, str>, MediaTypeObject>>,

    /// A map of operations links that can be followed from the response.
    /// The key of the map is a short name for the link, following the naming constraints
    /// of the names for Component Objects.
    #[serde(rename = "links", default, skip_serializing_if = "Option::is_none")]
    pub links: Option<HashMap<Cow<'static, str>, LinkOrReferenceObject>>,
}

mod serde_headers {
    use serde::ser::SerializeMap;
    use std::{collections::HashMap, marker::PhantomData, str::FromStr};

    #[repr(C)]
    struct HeaderNameWrapper(http::header::HeaderName);

    impl serde::Serialize for HeaderNameWrapper {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(self.0.as_str())
        }
    }

    struct HeaderNameVisitor;

    impl<'de> serde::Deserialize<'de> for HeaderNameWrapper {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_str(HeaderNameVisitor)
        }
    }

    impl<'de> serde::de::Visitor<'de> for HeaderNameVisitor {
        type Value = HeaderNameWrapper;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a header name")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(HeaderNameWrapper(
                http::header::HeaderName::from_str(v).map_err(|_| {
                    serde::de::Error::invalid_value(serde::de::Unexpected::Str(v), &self)
                })?,
            ))
        }
    }

    pub fn serialize<V: serde::Serialize, S: serde::Serializer>(
        val: &HashMap<http::header::HeaderName, V>,
        ser: S,
    ) -> Result<S::Ok, S::Error> {
        let mut map_ser = ser.serialize_map(Some(val.len()))?;

        for (key, val) in val.iter() {
            let wrapper: &HeaderNameWrapper =
                // SAFETY: HeaderNameWrapper is a wrapper around HeaderName with the same size and alignment
                unsafe { *(&key as *const _ as *const &HeaderNameWrapper) };
            map_ser.serialize_entry(wrapper, val)?;
        }

        map_ser.end()
    }

    struct HeadersVisitor<V> {
        _v: PhantomData<V>,
    }

    impl<'de, V: serde::de::Deserialize<'de>> serde::de::Visitor<'de> for HeadersVisitor<V> {
        type Value = HashMap<http::header::HeaderName, V>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a header map")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut result = if let Some(size_hint) = map.size_hint() {
                HashMap::with_capacity(size_hint)
            } else {
                HashMap::new()
            };

            while let Some((HeaderNameWrapper(key), val)) =
                map.next_entry::<HeaderNameWrapper, V>()?
            {
                result.insert(key, val);
            }

            Ok(result)
        }
    }

    pub fn deserialize<'de, D, V: for<'a> serde::Deserialize<'a>>(
        de: D,
    ) -> Result<HashMap<http::header::HeaderName, V>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        de.deserialize_map(HeadersVisitor::<V> { _v: PhantomData })
    }
}

/// Represents either an [`ResponseObject`] or [`ReferenceObject`] object.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ResponseOrReferenceObject {
    /// An inline response object.
    Response(ResponseObject),
    /// A reference to a response object.
    Reference(ReferenceObject),
}

impl From<ResponseObject> for ResponseOrReferenceObject {
    fn from(value: ResponseObject) -> Self {
        Self::Response(value)
    }
}

impl From<ReferenceObject> for ResponseOrReferenceObject {
    fn from(value: ReferenceObject) -> Self {
        Self::Reference(value)
    }
}
