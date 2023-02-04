//! Processing requests.

use std::slice;
use std::borrow::Cow;
use std::collections::HashMap;
use hyper::{Body, Uri};
use hyper::header::{HeaderMap, HeaderValue};
use hyper::http::uri::PathAndQuery;
use url::form_urlencoded;
use url::percent_encoding::percent_decode;
use super::response::Response;


//------------ Request -------------------------------------------------------

/// An HTTP request.
#[derive(Debug)]
pub struct Request(hyper::Request<Body>);

impl Request {
    pub fn from_hyper(request: hyper::Request<Body>) -> Self {
        Request(request)
    }

    pub fn uri(&self) -> &Uri {
        self.0.uri()
    }

    pub fn path(&self) -> RequestPath {
        RequestPath::from_request(self)
    }

    pub fn path_str(&self) -> &str {
        self.uri().path()
    }

    pub fn query(&self) -> RequestQuery {
        RequestQuery::from_request(self)
    }

    pub fn headers(&self) -> &HeaderMap<HeaderValue> {
        self.0.headers()
    }
}

impl Request {
    pub fn require_get(&self) -> Result<(), Response> {
        if self.0.method() != hyper::Method::GET {
            Err(Response::method_not_allowed())
        }
        else {
            Ok(())
        }
    }
}

impl From<hyper::Request<Body>> for Request {
    fn from(src: hyper::Request<Body>) -> Self {
        Self::from_hyper(src)
    }
}


//------------ RequestPath ---------------------------------------------------

pub struct RequestPath {
    path: Result<PathAndQuery, String>,
    segment: (usize, usize),
}

impl RequestPath {
    fn from_request(request: &Request) -> Self {
        let path = if let Cow::Owned(some) = percent_decode(
            request.uri().path().as_bytes()
        ).decode_utf8_lossy() {
            Err(some)
        }
        else {
            Ok(request.uri().path_and_query().unwrap().clone())
        };
        let mut res = RequestPath {
            path,
            segment: (0, 0),
        };
        res.next_segment();
        res
    }

    pub fn full(&self) -> &str {
        match self.path.as_ref() {
            Ok(path) => path.path(),
            Err(path) => path.as_str()
        }
    }

    pub fn remaining(&self) -> &str {
        &self.full()[self.segment.0..]
    }

    pub fn segment(&self) -> &str {
        &self.full()[self.segment.0..self.segment.1]
    }

    fn next_segment(&mut self) -> bool {
        let mut start = self.segment.1;
        let path = self.full();
        // Start beyond the length of the path signals the end.
        if start >= path.len() {
            return false;
        }
        // Skip any leading slashes. There may be multiple which should be
        // folded into one (or at least that’s what we do).
        while path.split_at(start).1.starts_with('/') {
            start += 1
        }
        // Find the next slash. If we have one, that’s the end of
        // our segment, otherwise, we go all the way to the end of the path.
        let end = path[start..].find('/').map(|x| x + start)
                                         .unwrap_or(path.len());
        self.segment = (start, end);
        true 
    }

    pub fn next(&mut self) -> Option<&str> {
        if self.next_segment() {
            Some(self.segment())
        }
        else {
            None
        }
    }

    /// Returns the next segment if it is the final segment.
    ///
    /// If there are more segments after the next segment, returns the entire
    /// remaining path as an error.
    pub fn next_and_last(&mut self) -> Result<Option<&str>, &str> {
        if !self.next_segment() {
            return Ok(None)
        }
        let path = self.full();
        if self.segment.1 == path.len()
            || (self.segment.1 + 1 == path.len()
                    && path.as_bytes()[self.segment.1] == b'/'
                )
        {
            return Ok(Some(self.segment()))
        }
        else {
            Err(self.remaining())
        }
    }
}


//------------ RequestQuery -------------------------------------------------

#[derive(Default)]
pub struct RequestQuery {
    query: HashMap<String, Result<String, Vec<String>>>,
}

impl RequestQuery {
    fn from_request(request: &Request) -> Self {
        let mut res = Self::default();
        form_urlencoded::parse(
            request.uri().query().unwrap_or("").as_bytes()
        ).into_owned().for_each(|(key, value)| { res.insert(key, value); });
        res
    }

    fn insert(&mut self, key: String, value: String) {
        use std::collections::hash_map::Entry::*;

        match self.query.entry(key) {
            Occupied(mut entry) => {
                if let Err(entry) = entry.get_mut().as_mut() {
                    entry.push(value);
                }
                else {
                    *(entry.get_mut()) = Err(vec!(value));
                }
            }
            Vacant(entry) => {
                entry.insert(Ok(value));
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&[String]> {
        match self.query.get(key).as_ref()? {
            Ok(s) => Some(slice::from_ref(s)),
            Err(vec) => Some(vec.as_ref()),
        }
    }

    pub fn get_first(&self, key: &str) -> Option<&str> {
        match self.query.get(key).as_ref()? {
            Ok(s) => Some(s.as_str()),
            Err(vec) => vec.first().map(String::as_str)
        }
    }
}

