//! HTTP client implementation for consuming the Wanikani API

use std::fmt::Debug;

use async_recursion::async_recursion;
use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::{header::HeaderValue, Client, RequestBuilder, Response};
use tokio::{sync::Mutex, time::Instant};
use url::Url;

use crate::{Error, Resource, WanikaniError, API_VERSION, URL_BASE};

const REVISION_HEADER: &str = "Wanikani-Revision";

#[derive(Debug, Clone)]
struct RateLimit {
    limit: u32,
    remaining: u32,
    reset: DateTime<Utc>,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            limit: 60,
            remaining: 60,
            reset: Utc::now(),
        }
    }
}

impl RateLimit {
    pub async fn wait_for_reset(&self) {
        if self.remaining > 0 {
            return;
        }
        log::info!("Rate limit reached, waiting for reset");
        let duration = (Utc::now() - self.reset)
            .to_std()
            .expect("Duration should be a matter of minutes");

        tokio::time::sleep_until(Instant::now() + duration).await
    }

    pub fn update_data(&mut self, limit: u32, remaining: u32, reset: i64) {
        let naive_datetime =
            NaiveDateTime::from_timestamp_millis(reset * 1000).expect("Valid range");
        let reset = DateTime::from_utc(naive_datetime, Utc);
        self.limit = limit;
        self.remaining = remaining;
        self.reset = reset;

        log::debug!("New rate limit: {self:?}");
    }
}

/// The Wanikani client struct performs requests to the API.
///
/// Collection requests will be aggregated, and rate limiting will be respected.
pub struct WKClient {
    base_url: Url,
    token: String,
    client: Client,
    version: &'static str,
    rate_limit: Mutex<RateLimit>,
}

impl Debug for WKClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WKClient")
            .field("base_url", &self.base_url)
            .field("client", &self.client)
            .field("version", &self.version)
            .field("token", &"*snip*")
            .finish()
    }
}

impl WKClient {
    /// Create a new client.
    pub fn new(token: String, client: Client) -> Self {
        let base_url = URL_BASE.parse().expect("Valid URL");
        Self {
            base_url,
            token,
            client,
            version: API_VERSION,
            rate_limit: Mutex::new(RateLimit::default()),
        }
    }

    fn add_required_headers(&self, req: RequestBuilder) -> RequestBuilder {
        req.bearer_auth(&self.token)
            .header(REVISION_HEADER, self.version)
    }

    async fn wait_for_reset(&self) {
        self.rate_limit.lock().await.wait_for_reset().await
    }

    async fn update_rate_limit(&self, resp: &Response) {
        let headers = resp.headers();
        let limit: u32 = headers
            .get("RateLimit-Limit")
            .unwrap_or(
                &HeaderValue::from_str({
                    log::warn!("RateLimit-Limit header not found");
                    "60"
                })
                .expect("Valid Header"),
            )
            .to_str()
            .expect("Header should be string")
            .parse()
            .unwrap_or(60);
        let remaining: u32 = headers
            .get("RateLimit-Remaining")
            .unwrap_or(
                &HeaderValue::from_str({
                    log::warn!("RateLimit-Remaining header not found");
                    "60"
                })
                .expect("Valid Header"),
            )
            .to_str()
            .expect("Header should be string")
            .parse()
            .unwrap_or(60);
        let reset: i64 = headers
            .get("RateLimit-Reset")
            .unwrap_or(
                &HeaderValue::from_str({
                    log::warn!("RateLimit-Reset header not found");
                    "0"
                })
                .expect("Valid Header"),
            )
            .to_str()
            .expect("Header should be string")
            .parse()
            .unwrap_or(0);
        let mut guard = self.rate_limit.lock().await;
        guard.update_data(limit, remaining, reset);
    }

    async fn handle_error(&self, response: Response) -> Error {
        log::error!("Status code {} received", response.status());
        match response.json::<WanikaniError>().await {
            Ok(error) => error.into(),
            Err(e) => e.into(),
        }
    }

    async fn handle_rate_limiting(&self, resp: &Response) {
        self.update_rate_limit(resp).await;
        self.wait_for_reset().await;
    }

    #[cfg(feature = "summary")]
    #[async_recursion]
    /// Get a summary report of available and upcoming lessons and reviews.
    pub async fn get_summary(&self) -> Result<Resource, Error> {
        use reqwest::StatusCode;

        use crate::SUMMARY_PATH;

        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("Valid URL")
            .push(SUMMARY_PATH);

        let req = self.add_required_headers(self.client.get(url));

        log::debug!("get_summary request: {req:?}");

        let resp = req.send().await?;

        log::debug!("get_summary response: {resp:?}");

        match resp.status() {
            StatusCode::OK => Ok(resp.json().await?),
            StatusCode::TOO_MANY_REQUESTS => {
                self.handle_rate_limiting(&resp).await;
                self.get_summary().await
            }
            _ => Err(self.handle_error(resp).await),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use reqwest::Client;

    use super::WKClient;
    use crate::{init_tests, Resource};

    fn create_client() -> WKClient {
        WKClient::new(
            env::var("API_KEY").expect("API_KEY provided"),
            Client::default(),
        )
    }

    #[cfg(feature = "summary")]
    #[tokio::test]
    async fn test_get_summary() {
        init_tests();

        let client = create_client();

        let Resource::Report(_summary) = client.get_summary().await.expect("Success") else {
            panic!("Incorrect Resource received")
        };
    }
}
