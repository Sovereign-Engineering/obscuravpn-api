pub struct Response<T> {
    body: Option<T>,
    etag: Option<http::HeaderValue>,
}

impl<T> Response<T> {
    pub(crate) fn new(body: Option<T>, etag: Option<http::HeaderValue>) -> Self {
        Response { body, etag }
    }

    pub fn etag(&self) -> Option<&[u8]> {
        self.etag.as_ref().map(|h| h.as_bytes())
    }

    /// Get the response body.
    ///
    /// This will be None only if the response was a 304 Not Modified.
    pub fn into_body(self) -> Option<T> {
        self.body
    }
}
