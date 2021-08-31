//! Serializer and deserializer for the [VCR 6.0 Cassette
//! format](https://relishapp.com/vcr/vcr/v/6-0-0/docs/cassettes/cassette-format).
//!
//! # Examples
//!
//! Given the following `.json` VCR Cassette recording:
//! ```json
//! {
//!     "http_interactions": [
//!         {
//!             "request": {
//!                 "uri": "http://localhost:7777/foo",
//!                 "body": "",
//!                 "method": "get",
//!                 "headers": { "Accept-Encoding": [ "identity" ] }
//!             },
//!             "response": {
//!                 "body": "Hello foo",
//!                 "http_version": "1.1",
//!                 "status": { "code": 200, "message": "OK" },
//!                 "headers": {
//!                     "Date": [ "Thu, 27 Oct 2011 06:16:31 GMT" ],
//!                     "Content-Type": [ "text/html;charset=utf-8" ],
//!                     "Content-Length": [ "9" ],
//!                 }
//!             },
//!             "recorded_at": "Tue, 01 Nov 2011 04:58:44 GMT"
//!         },
//!     ],
//!     "recorded_with": "VCR 2.0.0"
//! }
//! ```
//!
//! We can deserialize it using [`serde_json`](https://docs.rs/serde-json):
//!
//! ```rust
//! # #![allow(unused)]
//! use std::fs;
//! use vcr_cassette::Cassette;
//!
//! let example = fs::read_to_string("tests/fixtures/example.json").unwrap();
//! let cassette: Cassette = serde_json::from_str(&example).unwrap();
//! ```
//!
//! To deserialize `.yaml` Cassette files use
//! [`serde_yaml`](https://docs.rs/serde-yaml) instead.

#![forbid(unsafe_code, future_incompatible)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, unreachable_pub)]

use std::fmt;
use std::marker::PhantomData;
use std::{collections::HashMap, str::FromStr};

use chrono::{offset::FixedOffset, DateTime};
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use void::Void;

mod datetime;

/// An HTTP Headers type.
pub type Headers = HashMap<String, Vec<String>>;

/// An identifier of the library which created the recording.
///
/// # Examples
///
/// ```
/// # #![allow(unused)]
/// use vcr_cassette::RecorderId;
///
/// let id: RecorderId = String::from("VCR 2.0.0");
/// ```
pub type RecorderId = String;

/// A sequence of recorded HTTP Request/Response pairs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cassette {
    /// A sequence of recorded HTTP Request/Response pairs.
    pub http_interactions: Vec<HttpInteraction>,

    /// An identifier of the library which created the recording.
    pub recorded_with: RecorderId,
}

/// A single HTTP Request/Response pair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpInteraction {
    /// An HTTP response.
    pub response: Response,
    /// An HTTP request.
    pub request: Request,

    /// An [RFC
    /// 2822](https://docs.rs/chrono/0.4.19/chrono/struct.DateTime.html#method.parse_from_rfc2822)
    /// formatted timestamp.
    ///
    /// # Examples
    ///
    /// ```json
    /// { "recorded_at": "Tue, 01 Nov 2011 04:58:44 GMT" }
    /// ```
    #[serde(with = "datetime")]
    pub recorded_at: DateTime<FixedOffset>,
}

/// A recorded HTTP Response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    /// An HTTP Body.
    #[serde(deserialize_with = "string_or_struct")]
    pub body: Body,
    /// The version of the HTTP Response.
    pub http_version: Option<Version>,
    /// The Response status
    pub status: Status,
    /// The Response headers
    pub headers: Headers,
}

/// A recorded HTTP Body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Body {
    /// The encoding of the HTTP body.
    pub encoding: Option<String>,
    /// The HTTP body encoded as a string.
    pub string: String,
}

impl FromStr for Body {
    // This implementation of `from_str` can never fail, so use the impossible
    // `Void` type as the error type.
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Body {
            encoding: None,
            string: s.to_string(),
        })
    }
}

/// A recorded HTTP Status Code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Status {
    /// The HTTP status code.
    pub code: u16,
    /// The HTTP status message.
    pub message: String,
}

/// A recorded HTTP Request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    /// The Request URI.
    pub uri: String,
    /// The Request body.
    #[serde(deserialize_with = "string_or_struct")]
    pub body: Body,
    /// The Request method.
    pub method: Method,
    /// The Request headers.
    pub headers: Headers,
}

/// An HTTP method.
///
/// WebDAV and custom methods can be created by passing a static string to the
/// `Other` member.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    /// An HTTP `CONNECT` method.
    Connect,
    /// An HTTP `DELETE` method.
    Delete,
    /// An HTTP `GET` method.
    Get,
    /// An HTTP `HEAD` method.
    Head,
    /// An HTTP `OPTIONS` method.
    Options,
    /// An HTTP `PATCH` method.
    Patch,
    /// An HTTP `POST` method.
    Post,
    /// An HTTP `PUT` method.
    Put,
    /// An HTTP `TRACE` method.
    Trace,
    /// Any other HTTP method.
    Other(String),
}

impl Method {
    /// Convert the HTTP method to its string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Method::Connect => "CONNECT",
            Method::Delete => "DELETE",
            Method::Get => "GET",
            Method::Head => "HEAD",
            Method::Options => "OPTIONS",
            Method::Patch => "PATCH",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Trace => "TRACE",
            Method::Other(s) => &s,
        }
    }
}

/// The version of the HTTP protocol in use.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[non_exhaustive]
pub enum Version {
    /// HTTP/0.9
    #[serde(rename = "0.9")]
    Http0_9,

    /// HTTP/1.0
    #[serde(rename = "1.0")]
    Http1_0,

    /// HTTP/1.1
    #[serde(rename = "1.1")]
    Http1_1,

    /// HTTP/2.0
    #[serde(rename = "2")]
    Http2_0,

    /// HTTP/3.0
    #[serde(rename = "3")]
    Http3_0,
}

// Copied from: https://serde.rs/string-or-struct.html
fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Void>,
    D: Deserializer<'de>,
{
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = Void>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}
