//! Module for common stuff.

use reqwest::blocking::Client;

/// Get an http client.
pub fn get_http_client() -> Client {
    Client::builder()
        .user_agent("github.com/cezarmathe/nomadutil")
        .build()
        .expect("failed to create the http client")
}

/// Convert an Option<String> to an Option<&str>
#[inline]
pub fn opt_string_to_opt_str(src: &Option<String>) -> Option<&str> {
    if let Some(value) = src {
        Some(value.as_str())
    } else {
        None
    }
}
