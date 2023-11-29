//! Processing requests.

use std::{mem, slice};
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

    pub fn path(&self) -> Result<RequestPath, InvalidPath> {
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

#[derive(Debug)]
pub struct RequestPath {
    path: Result<PathAndQuery, String>,
}

impl RequestPath {
    fn from_request(request: &Request) -> Result<Self, InvalidPath> {
        let path = if let Cow::Owned(some) = percent_decode(
            request.uri().path().as_bytes()
        ).decode_utf8().map_err(|_| InvalidPath)? {
            Err(some)
        }
        else {
            Ok(request.uri().path_and_query().ok_or(InvalidPath)?.clone())
        };
        Ok(Self { path })
    }

    pub fn as_str(&self) -> &str {
        match self.path.as_ref() {
            Ok(path) => path.path(),
            Err(path) => path.as_str()
        }
    }

    pub fn iter(&self) -> PathIter {
        PathIter::new(self.as_str())
    }
}

impl AsRef<str> for RequestPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}


//------------ PathIter -------------------------------------------------

#[derive(Debug)]
pub struct PathIter<'a> {
    full: &'a str,
    remaining: &'a str,
}

impl<'a> PathIter<'a> {
    fn new(path: &'a str) -> Self {
        let remaining = if path.starts_with('/') {
            &path[1..]
        }
        else {
            path
        };
        Self { full: path, remaining }
    }

    pub fn full(&self) -> &str {
        self.full
    }

    pub fn remaining(&self) -> &str {
        self.remaining
    }

    pub fn len(&self) -> usize {
        self.remaining.len()
    }

    pub fn is_empty(&self) -> bool {
        self.remaining.is_empty()
    }
}

impl<'a> Iterator for PathIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None
        }
        let slash = match self.remaining.find('/') {
            Some(pos) => pos,
            None => {
                let res = self.remaining;
                self.remaining = "";
                return Some(res)
            }
        };
        let res = &self.remaining[..slash];
        self.remaining = &self.remaining[slash + 1..];
        Some(res)
    }
}


//------------ RequestQuery -------------------------------------------------

#[derive(Default)]
pub struct RequestQuery {
    query: HashMap<String, QueryValue>,
}

impl RequestQuery {
    fn from_request(request: &Request) -> Self {
        Self::from_uri(request.uri())
    }

    fn from_uri(uri: &Uri) -> Self {
        let mut res = Self::default();
        form_urlencoded::parse(
            uri.query().unwrap_or("").as_bytes()
        ).into_owned().for_each(|(key, value)| { res.insert(key, value); });
        res
    }

    fn insert(&mut self, key: String, value: String) {
        use std::collections::hash_map::Entry::*;

        match self.query.entry(key) {
            Occupied(mut entry) => {
                entry.get_mut().into_multi().push(value)
            }
            Vacant(entry) => {
                entry.insert(QueryValue::Single(value));
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&[String]> {
        match self.query.get(key)? {
            QueryValue::Single(s) => Some(slice::from_ref(s)),
            QueryValue::Multi(vec) => Some(vec.as_ref()),
        }
    }

    pub fn take(&mut self, key: &str) -> Option<QueryValue> {
        self.query.remove(key)
    }

    pub fn get_first(&self, key: &str) -> Option<&str> {
        match self.query.get(key)? {
            QueryValue::Single(s) => Some(s.as_str()),
            QueryValue::Multi(vec) => vec.first().map(String::as_str)
        }
    }
}


//------------ QueryValue ----------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryValue {
    Single(String),
    Multi(Vec<String>),
}

impl QueryValue {
    fn into_multi(&mut self) -> &mut Vec<String> {
        if let Self::Multi(ref mut vec) = self {
            return vec
        }
        let old = mem::replace(self, Self::Multi(Vec::new()));
        let vec = match self {
            Self::Multi(ref mut vec) => vec,
            _ => unreachable!()
        };
        match old {
            Self::Single(old) => vec.push(old),
            _ => unreachable!()
        }
        vec
    }
}


//------------ InvalidPath ---------------------------------------------------

#[derive(Debug)]
pub struct InvalidPath;


//============ Tests =========================================================

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use super::*;

    #[test]
    fn request_query() {
        let query = RequestQuery::from_uri(
            &Uri::from_str("http://foo/bar?a=b&c=d&e=f&c=g").unwrap()
        );
        let mut vec = query.query.clone().into_iter().collect::<Vec<_>>();
        vec.sort_by(|l, r| l.0.cmp(&r.0));

        assert_eq!(
            vec.as_slice(),
            &[
                ("a".into(), QueryValue::Single("b".into())),
                ("c".into(), QueryValue::Multi(
                    vec!["d".into(), "g".into()]
                )),
                ("e".into(), QueryValue::Single("f".into())),
            ]
        );

        assert_eq!(
            query.get("a"),
            Some(["b".into()].as_slice())
        );
        assert_eq!(
            query.get("c"),
            Some(["d".into(), "g".into()].as_slice())
        );
        assert_eq!(
            query.get("e"),
            Some(["f".into()].as_slice())
        );

        assert_eq!(query.get_first("a"), Some("b"));
        assert_eq!(query.get_first("c"), Some("d"));
        assert_eq!(query.get_first("e"), Some("f"));
    }
}

