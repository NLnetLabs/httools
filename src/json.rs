//! Building JSON on the fly.
#![cfg(feature = "json")]

use std::fmt;
use crate::response::{ContentType, Response};


//------------ BuildJson -----------------------------------------------------

pub trait BuildJson {
    fn build_json(&self, builder: &mut JsonValue);
}

impl<F: Fn(&mut JsonValue)> BuildJson for F {
    fn build_json(&self, builder: &mut JsonValue) {
        (self)(builder)
    }
}


//------------ JsonBuilder ---------------------------------------------------

/// A helper type for building a JSON-encoded string on the fly.
///
/// Note that the builder only supports strings without control characters.
pub struct JsonBuilder {
    target: String,
}

impl JsonBuilder {
    pub fn build<F: FnOnce(&mut JsonBuilder)>(op: F) -> String {
        let mut builder = JsonBuilder { target: String::new() };
        op(&mut builder);
        builder.target
    }

    pub fn ok<F: FnOnce(&mut JsonBuilder)>(op: F) -> Response {
        Response::ok(ContentType::JSON, Self::build(op))
    }

    pub fn ok_object<F: FnOnce(&mut JsonObject)>(op: F) -> Response {
        Response::ok(ContentType::JSON, Self::build(|json| json.object(op)))
    }
}

impl JsonBuilder {
    pub fn value(&mut self, op: impl FnOnce(&mut JsonValue)) {
        op(&mut JsonValue {
            target: &mut self.target,
            indent: 1,
        });
    }

    pub fn object<F: FnOnce(&mut JsonObject)>(
        &mut self, op: F
    ) {
        self.value(|json| json.object(op));
        /*
        self.target.push_str("{\n");
        op(&mut JsonObject {
            target: &mut self.target,
            indent: 1,
            empty: true
        });
        self.target.push_str("\n}");
        */
    }

    pub fn array<F: FnOnce(&mut JsonArray)>(
        &mut self, op: F
    ) {
        self.value(|json| json.array(op));
        /*
        self.target.push_str("[\n");
        op(&mut JsonArray {
            target: &mut self.target,
            indent: 1,
            empty: true
        });
        self.target.push_str("\n]");
        */
    }

    pub fn string(
        &mut self, value: impl fmt::Display
    ) {
        self.value(|json| json.string(value));
        /*
        self.target.push('"');
        write!(self.target, "{}", json_str(value));
        self.target.push('"');
        */
    }

    pub fn raw(
        &mut self, value: impl fmt::Display
    ) {
        self.value(|json| json.raw(value));
        //write!(self.target, "{}", json_str(value));
    }
}


//------------ JsonObject ---------------------------------------------------

pub struct JsonObject<'a> {
    target: &'a mut String,
    indent: usize,
    empty: bool,
}
    
impl<'a> JsonObject<'a> {
    pub fn value(
        &mut self,
        key: impl fmt::Display,
        op: impl FnOnce(&mut JsonValue)
    ) {
        self.append_key(key);
        op(&mut JsonValue {
            target: self.target,
            indent: self.indent + 1,
        });
    }

    pub fn object<F: FnOnce(&mut JsonObject)>(
        &mut self, key: impl fmt::Display, op: F
    ) {
        self.value(key, |json| json.object(op))
        /*
        self.append_key(key);
        self.target.push_str("{\n");
        op(&mut JsonObject {
            target: self.target,
            indent: self.indent + 1,
            empty: true
        });
        self.target.push('\n');
        self.append_indent();
        self.target.push('}');
        */
    }

    pub fn array<F: FnOnce(&mut JsonArray)>(
        &mut self, key: impl fmt::Display, op: F
    ) {
        self.value(key, |json| json.array(op))
        /*
        self.append_key(key);
        self.target.push_str("[\n");
        op(&mut JsonArray {
            target: self.target,
            indent: self.indent + 1,
            empty: true
        });
        self.target.push('\n');
        self.append_indent();
        self.target.push(']');
        */
    }

    pub fn string(
        &mut self, key: impl fmt::Display, value: impl fmt::Display
    ) {
        self.value(key, |json| json.string(value))
        /*
        self.append_key(key);
        self.target.push('"');
        write!(self.target, "{}", json_str(value));
        self.target.push('"');
        */
    }

    pub fn raw(
        &mut self, key: impl fmt::Display, value: impl fmt::Display
    ) {
        self.value(key, |json| json.raw(value))
        /*
        self.append_key(key);
        write!(self.target, "{}", json_str(value));
        */
    }

    fn append_key(&mut self, key: impl fmt::Display) {
        if self.empty {
            self.empty = false
        }
        else {
            self.target.push_str(",\n");
        }
        self.append_indent();
        self.target.push('"');
        write!(self.target, "{}", json_str(key));
        self.target.push('"');
        self.target.push_str(": ");
    }

    fn append_indent(&mut self) {
        for _ in 0..self.indent {
            self.target.push_str("   ");
        }
    }
}


//------------ JsonArray ----------------------------------------------------

pub struct JsonArray<'a> {
    target: &'a mut String,
    indent: usize,
    empty: bool,
}

impl<'a> JsonArray<'a> {
    pub fn value(&mut self, op: impl FnOnce(&mut JsonValue)) {
        self.append_array_head();
        self.append_indent();
        op(&mut JsonValue {
            target: self.target,
            indent: self.indent + 1,
        })
    }

    pub fn object<F: FnOnce(&mut JsonObject)>(&mut self, op: F) {
        self.value(|json| json.object(op))
        /*
        self.append_array_head();
        self.append_indent();
        self.target.push_str("{\n");
        op(&mut JsonObject {
            target: self.target,
            indent: self.indent + 1,
            empty: true
        });
        self.target.push('\n');
        self.append_indent();
        self.target.push('}');
        */
    }

    pub fn array<F: FnOnce(&mut JsonArray)>(&mut self, op: F) {
        self.value(|json| json.array(op))
        /*
        self.append_array_head();
        self.append_indent();
        self.target.push_str("[\n");
        op(&mut JsonArray {
            target: self.target,
            indent: self.indent + 1,
            empty: true
        });
        self.target.push('\n');
        self.append_indent();
        self.target.push(']');
        */
    }

    pub fn string(&mut self, value: impl fmt::Display) {
        self.value(|json| json.string(value))
        /*
        self.append_array_head();
        self.append_indent();
        self.target.push('"');
        write!(self.target, "{}", json_str(value));
        self.target.push('"');
        */
    }

    pub fn raw(&mut self, value: impl fmt::Display) {
        self.value(|json| json.raw(value))
        /*
        self.append_array_head();
        self.append_indent();
        write!(self.target, "{}", json_str(value));
        */
    }

    fn append_array_head(&mut self) {
        if self.empty {
            self.empty = false
        }
        else {
            self.target.push_str(",\n");
        }
    }

    fn append_indent(&mut self) {
        for _ in 0..self.indent {
            self.target.push_str("   ");
        }
    }
}


//------------ JsonValue ----------------------------------------------------

pub struct JsonValue<'a> {
    target: &'a mut String,
    indent: usize,
}

impl<'a> JsonValue<'a> {
    pub fn object<F: FnOnce(&mut JsonObject)>(&mut self, op: F) {
        self.target.push_str("{\n");
        op(&mut JsonObject {
            target: self.target,
            indent: self.indent + 1,
            empty: true
        });
        self.target.push('\n');
        self.append_indent();
        self.target.push('}');
    }

    pub fn array<F: FnOnce(&mut JsonArray)>(&mut self, op: F) {
        self.target.push_str("[\n");
        op(&mut JsonArray {
            target: self.target,
            indent: self.indent + 1,
            empty: true
        });
        self.target.push('\n');
        self.append_indent();
        self.target.push(']');
    }

    pub fn string(&mut self, value: impl fmt::Display) {
        self.target.push('"');
        write!(self.target, "{}", json_str(value));
        self.target.push('"');
    }

    pub fn raw(&mut self, value: impl fmt::Display) {
        write!(self.target, "{}", json_str(value));
    }

    fn append_indent(&mut self) {
        for _ in 0..self.indent {
            self.target.push_str("   ");
        }
    }
}


//------------ json_str -----------------------------------------------------

pub fn json_str(val: impl fmt::Display) -> impl fmt::Display {
    struct WriteJsonStr<'a, 'f>(&'a mut fmt::Formatter<'f>);

    impl<'a, 'f> fmt::Write for WriteJsonStr<'a, 'f> {
        fn write_str(&mut self, mut s: &str) -> fmt::Result {
            while let Some(idx) = s.find(|ch| ch == '"' || ch == '\\') {
                self.0.write_str(&s[..idx])?;
                self.0.write_str("\\")?;
                write!(self.0, "{}", char::from(s.as_bytes()[idx]))?;
                s = &s[idx + 1..];
            }
            self.0.write_str(s)
        }
    }

    struct JsonStr<T>(T);

    impl<T: fmt::Display> fmt::Display for JsonStr<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            use std::fmt::Write;

            write!(&mut WriteJsonStr(f), "{}", self.0)
        }
    }

    JsonStr(val)
}


//------------ WriteOrPanic --------------------------------------------------

/// A target for writing formatted data into without error.
///
/// This provides a method `write_fmt` for use with the `write!` macro and
/// friends that does not return a result. Rather, it panics if an error
/// occurs.
pub trait WriteOrPanic {
    fn write_fmt(&mut self, args: fmt::Arguments);
}

impl WriteOrPanic for Vec<u8> {
    fn write_fmt(&mut self, args: fmt::Arguments) {
        std::io::Write::write_fmt(self, args).expect("formatting failed");
    }
}

impl WriteOrPanic for String {
    fn write_fmt(&mut self, args: fmt::Arguments) {
        std::fmt::Write::write_fmt(self, args).expect("formatting failed");
    }
}


//============ Tests =========================================================

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_json_str() {
        assert_eq!(
            format!("{}", json_str("foo")).as_str(),
            "foo"
        );
        assert_eq!(
            format!("{}", json_str("f\"oo")).as_str(),
            "f\\\"oo"
        );
        assert_eq!(
            format!("{}", json_str("f\\oo")).as_str(),
            "f\\\\oo"
        );
        assert_eq!(
            format!("{}", json_str("\\oo")).as_str(),
            "\\\\oo"
        );
        assert_eq!(
            format!("{}", json_str("foo\\")).as_str(),
            "foo\\\\"
        );
    }
}

