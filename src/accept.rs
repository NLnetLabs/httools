//! Handling of Accept headers and content types.

use hyper::header::HeaderValue;


//------------ Accept --------------------------------------------------------

pub struct Accept {
    value: HeaderValue,
}

impl Accept {
    fn get_serializer(content_type: ContentType) -> Option<Serializer> {
    }
}

