//! Building responses.

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};
use hyper::{Body, StatusCode};
use hyper::header::HeaderValue;
use hyper::http::response::Builder;
#[cfg(feature = "chrono")]
use crate::request::Request;


//------------ Response -----------------------------------------------------

#[derive(Debug)]
pub struct Response(hyper::Response<Body>);

impl Response {
    pub fn ok(content_type: ContentType, body: impl Into<Body>) -> Self {
        ResponseBuilder::new().ok()
            .content_type(content_type)
            .body(body)
    }

    /// Returns a Bad Request response.
    pub fn bad_request() -> Self {
        ResponseBuilder::new().bad_request()
            .content_type(ContentType::TEXT)
            .body("Bad Request")
    }

    /// Returns a Not Modified response.
    pub fn not_found() -> Self {
        ResponseBuilder::new().not_found()
            .content_type(ContentType::TEXT)
            .body("Not Found")
    }

    /// Returns a Not Modified response.
    #[cfg(feature = "chrono")]
    pub fn not_modified(etag: &str, done: DateTime<Utc>) -> Self {
        ResponseBuilder::new().not_modified()
            .etag(etag).last_modified(done)
            .empty()
    }

    /// Returns a Method Not Allowed response.
    pub fn method_not_allowed() -> Self {
        ResponseBuilder::new().method_not_allowed()
            .content_type(ContentType::TEXT)
            .body("Method not allowed.")
    }

    /// Returns a Moved Permanently response pointing to the given location.
    pub fn moved_permanently(location: &str) -> Self {
        ResponseBuilder::new().moved_permanently()
            .content_type(ContentType::TEXT)
            .location(location)
            .body(format!("Moved permanently to {}", location))
    }

    /// Returns a 304 Not Modified response if appropriate.
    ///
    /// If either the etag or the completion time are referred to by the
    /// request, returns the reponse. If a new response needs to be generated,
    /// returns `None`.
    #[cfg(feature = "chrono")]
    pub fn maybe_not_modified(
        req: &Request,
        etag: &str,
        done: DateTime<Utc>,
    ) -> Option<Response> {
        use crate::date::parse_http_date;

        // First, check If-None-Match.
        for value in req.headers().get_all("If-None-Match").iter() {
            // Skip ill-formatted values. By being lazy here we may falsely
            // return a full response, so this should be fine.
            let value = match value.to_str() {
                Ok(value) => value,
                Err(_) => continue
            };
            let value = value.trim();
            if value == "*" {
                return Some(Self::not_modified(etag, done))
            }
            for tag in EtagsIter(value) {
                if tag.trim() == etag {
                    return Some(Self::not_modified(etag, done))
                }
            }
        }

        // Now, the If-Modified-Since header.
        if let Some(value) = req.headers().get("If-Modified-Since") {
            if let Some(date) = parse_http_date(value.to_str().ok()?) {
                if date >= done {
                    return Some(Self::not_modified(etag, done))
                }
            }
        }

        None
    }

    /// Converts the response into a hyper response.
    pub fn into_hyper(self) -> hyper::Response<Body> {
        self.0
    }
}


//------------ ResponseBuilder ----------------------------------------------

#[derive(Debug)]
pub struct ResponseBuilder {
    builder: Builder,
}

impl ResponseBuilder {
    fn with(builder: Builder) -> Self {
        Self { builder }
    }

    /// Creates a new builder.
    pub fn new() -> Self {
        Self::with(Builder::new())
    }

    pub fn status(self, status: StatusCode) -> Self {
        Self::with(self.builder.status(status))
    }

    /// Creates a new builder for a 200 OK response.
    pub fn ok(self) -> Self {
        self.status(StatusCode::OK)
    }

    /// Creates a new builder for a Service Unavailable response.
    pub fn service_unavailable(self) -> Self {
        self.status(StatusCode::SERVICE_UNAVAILABLE)
    }

    /// Creates a new builder for a Bad Request response.
    pub fn bad_request(self) -> Self {
        self.status(StatusCode::BAD_REQUEST)
    }

    /// Creates a new builder for a Not Found response.
    pub fn not_found(self) -> Self {
        self.status(StatusCode::NOT_FOUND)
    }

    /// Creates a new builder for a Not Modified response.
    pub fn not_modified(self) -> Self {
        self.status(StatusCode::NOT_MODIFIED)
    }

    /// Creates a new builder for a Method Not Allowed response.
    pub fn method_not_allowed(self) -> Self {
        self.status(StatusCode::METHOD_NOT_ALLOWED)
    }

    /// Creates a new builder for a Moved Permanently response.
    pub fn moved_permanently(self) -> Self {
        self.status(StatusCode::MOVED_PERMANENTLY)
    }

    /// Adds the content type header.
    pub fn content_type(self, content_type: ContentType) -> Self {
        Self::with(self.builder.header("Content-Type", content_type.0))
    }

    /// Adds the ETag header.
    pub fn etag(self, etag: &str) -> Self {
        ResponseBuilder {
            builder: self.builder.header("ETag", etag)
        }
    }

    /// Adds the Last-Modified header.
    #[cfg(feature = "chrono")]
    pub fn last_modified(self, last_modified: DateTime<Utc>) -> Self {
        ResponseBuilder {
            builder: self.builder.header(
                "Last-Modified",
                crate::date::format_http_date(last_modified)
            )
        }
    }

    /// Adds the Location header.
    pub fn location(self, location: &str) -> Self {
        ResponseBuilder {
            builder: self.builder.header(
                "Location",
                location
            )
        }
    }

    /// Adds a Set-Cookie header using a static str as the value.
    pub fn set_static_cookie(mut self, value: &'static str) -> Self {
        self.builder.headers_mut().unwrap().append(
            "Set-Cookie", HeaderValue::from_static(value)
        );
        Self::with(self.builder)
    }

    /// Finalizes the response by adding a body.
    pub fn body(self, body: impl Into<Body>) -> Response {
        Response(
            self.builder.body(body.into())
                .expect("broken HTTP response builder")
        )
    }

    /// Finalies the response by adding an empty body.
    pub fn empty(self) -> Response {
        self.body(Body::empty())
    }
}


//------------ ContentType ---------------------------------------------------

#[derive(Clone, Debug)]
pub struct ContentType(HeaderValue);

impl ContentType {
    pub const CSS: ContentType = ContentType::external("text/css");
    pub const CSV: ContentType = ContentType::external(
        "text/csv;charset=utf-8;header=present"
    );
    pub const HTML: ContentType = ContentType::external(
        "text/html;charset=utf-8"
    );
    pub const JS: ContentType = ContentType::external(
        "text/javascript"
    );
    pub const JSON: ContentType = ContentType::external(
        "application/json"
    );
    pub const SVG: ContentType = ContentType::external(
        "image/svg+xml"
    );
    pub const TEXT: ContentType = ContentType::external(
        "text/plain;charset=utf-8"
    );
    pub const PROMETHEUS: ContentType = ContentType::external(
        "text/plain; version=0.0.4"
    );

    pub const fn external(value: &'static str) -> Self {
        ContentType(HeaderValue::from_static(value))
    }
}


//------------ Parsing Etags -------------------------------------------------

/// An iterator over the etags in an If-Not-Match header value.
///
/// This does not handle the "*" value.
///
/// One caveat: The iterator stops when it encounters bad formatting which
/// makes this indistinguishable from reaching the end of a correctly
/// formatted value. As a consequence, we will 304 a request that has the
/// right tag followed by garbage.
struct EtagsIter<'a>(&'a str);

impl<'a> Iterator for EtagsIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip white space and check if we are done.
        self.0 = self.0.trim_start();
        if self.0.is_empty() {
            return None
        }

        // We either have to have a lone DQUOTE or one prefixed by W/
        let prefix_len = if self.0.starts_with('"') {
            1
        }
        else if self.0.starts_with("W/\"") {
            3
        }
        else {
            return None
        };

        // Find the end of the tag which is after the next DQUOTE.
        let end = match self.0[prefix_len..].find('"') {
            Some(index) => index + prefix_len + 1,
            None => return None
        };

        let res = &self.0[0..end];

        // Move past the second DQUOTE and any space.
        self.0 = self.0[end..].trim_start();

        // If we have a comma, skip over that and any space.
        if self.0.starts_with(',') {
            self.0 = self.0[1..].trim_start();
        }

        Some(res)
    }
}


//============ Tests =========================================================

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn etags_iter() {
        assert_eq!(
            EtagsIter("\"foo\", \"bar\", \"ba,zz\"").collect::<Vec<_>>(),
            ["\"foo\"", "\"bar\"", "\"ba,zz\""]
        );
        assert_eq!(
            EtagsIter("\"foo\", W/\"bar\" , \"ba,zz\", ").collect::<Vec<_>>(),
            ["\"foo\"", "W/\"bar\"", "\"ba,zz\""]
        );
    }
}

