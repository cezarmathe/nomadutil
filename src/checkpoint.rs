//! Module for interacting with APIs.

use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::header::ACCEPT;

use serde::Deserialize;

// CheckResponse is the response for a check request.
#[derive(Clone, Debug, Deserialize)]
pub struct CheckResponse {
    product: String,
    current_version: String,
    current_release: i32,
    current_download_url: String,
    current_changelog_url: String,
    project_website: String,
    #[serde(default)]
    outdated: bool,
    alerts: Vec<CheckAlert>,
}

impl CheckResponse {
    #[allow(missing_docs)]
    #[inline]
    pub fn product(&self) -> &str {
        self.product.as_str()
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn current_version(&self) -> &str {
        self.current_version.as_str()
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn current_release(&self) -> i32 {
        self.current_release
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn current_download_url(&self) -> &str {
        self.current_download_url.as_str()
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn current_changelog_url(&self) -> &str {
        self.current_changelog_url.as_str()
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn project_website(&self) -> &str {
        self.project_website.as_str()
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn outdated(&self) -> bool {
        self.outdated
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn alerts(&self) -> &[CheckAlert] {
        self.alerts.as_slice()
    }
}

// CheckAlert is a single alert message from a check request.
//
// These never have to be manually constructed, and are typically populated
// into a CheckResponse as a result of the Check request.
#[derive(Clone, Debug, Deserialize)]
pub struct CheckAlert {
    id: i32,
    date: i32,
    message: String,
    url: String,
    level: String,
}

impl CheckAlert {
    #[allow(missing_docs)]
    #[inline]
    pub fn id(&self) -> i32 {
        self.id
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn date(&self) -> i32 {
        self.date
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn level(&self) -> &str {
        self.level.as_str()
    }
}

/// Make a check using the hashicorp checkpoint api.
pub fn check(version: Option<&str>) -> anyhow::Result<CheckResponse> {
    let client = Client::builder()
        // https://github.com/hashicorp/go-checkpoint/blob/bbe6c410aa4be4194cb490a2bde8c3c33f295541/check.go#L101-L102
        .timeout(Duration::from_secs(3))
        .user_agent("github.com/cezarmathe/nomadutil")
        .build()?;

    let queries: Vec<(&str, &str)> = {
        let mut queries = Vec::new();

        queries.push(("arch", crate::ARCH));
        queries.push(("os", "linux"));

        if let Some(value) = version {
            queries.push(("version", value));
        }

        queries
    };

    Ok(client
        .get("https://checkpoint-api.hashicorp.com/v1/check/nomad")
        .header(ACCEPT, "application/json")
        .query(queries.as_slice())
        .send()?
        .json()?)
}
