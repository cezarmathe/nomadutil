//! Module for common stuff.

use reqwest::blocking::Client;

/// Get an http client.
pub fn get_http_client() -> Client {
    Client::builder()
        .user_agent("github.com/cezarmathe/nomadutil")
        .build()
        .expect("failed to create the http client")
}
