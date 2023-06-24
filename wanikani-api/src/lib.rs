#![forbid(unsafe_code)]
#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(missing_docs, clippy::todo, clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../intro.md")]

use std::fmt::Display;

use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "summary")]
pub mod summary;

const SUMMARY_PATH: &str = "summary";

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Struct that contains fields common to all resources
pub struct ResourceCommon {
    /// The URL of the request. For collections, that will contain all the
    /// filters and options you've passed to the API. Resources have a single
    /// URL and don't need to be filtered, so the URL will be the same in both
    /// resource and collection responses.
    pub url: Url,
    /// For collections, this is the timestamp of the most recently updated
    /// resource in the [specified scope](index.html#filters) and is not limited by
    /// pagination. If no resources were returned for the specified scope, then
    /// this will be `null`. For a resource, then this is the last time that
    /// particular resource was updated.
    pub data_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Pagination information for collections
pub struct Pages {
    /// The URL for the next page of resources, if one exists.
    pub next_url: Option<Url>,
    /// The URL for the previous page of resources, if one exists.
    pub previous_url: Option<Url>,
    /// The number of resources per page.
    pub per_page: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// A collection of resources
pub struct Collection {
    #[serde(flatten)]
    /// Common resource data
    pub common: ResourceCommon,
    /// Pagination data for the collection
    pub pages: Pages,
    /// The total count of resources in the collection
    pub total_count: u64,
    /// The collection's data
    pub data: Vec<Resource>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// The Wanikani API error object
pub struct WanikaniError {
    /// The numeric error code. This is likely going to match the HTTP status
    /// code.
    pub code: i32,
    /// The returned error message, if one was sent by the API
    pub error: Option<String>,
}

impl Display for WanikaniError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.error {
            None => write!(f, "Error code {} received", self.code),
            Some(ref message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for WanikaniError {}

#[derive(Debug, Error)]
/// Possible error conditions
pub enum Error {
    #[error("Wanikani error: {0}")]
    /// An error was returned by Wanikani.
    WanikaniError(#[from] WanikaniError),
    #[error("HTTP client error: {0}")]
    /// There was some error in the HTTP client.
    Client(#[from] reqwest::Error),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "object")]
/// Possible resource types
pub enum Resource {
    /// Placeholder value for when no resource features are selected.
    ///
    /// This should never actually show up in reality.
    None,
    #[cfg(feature = "summary")]
    /// A summary report
    Report(summary::Summary),
}

/// The version of the API supported by this library
pub const API_VERSION: &str = "20170710";

/// The base URL of the Wanikani V2 API
pub const URL_BASE: &str = "https://api.wanikani.com/v2";

#[cfg(test)]
static INIT: std::sync::OnceLock<()> = std::sync::OnceLock::new();

#[cfg(test)]
fn init_tests() {
    INIT.get_or_init(|| {
        dotenvy::dotenv().ok();
        env_logger::init()
    });
}
