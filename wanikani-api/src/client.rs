//! HTTP client implementation for consuming the Wanikani API

use std::fmt::Debug;

use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::{
    header::{HeaderMap},
    Client, RequestBuilder, Response, StatusCode,
};
use url::Url;

use crate::{Error, Resource, Timestamp, WanikaniError, API_VERSION, URL_BASE};

const REVISION_HEADER: &str = "Wanikani-Revision";

/// The Wanikani client struct performs requests to the API.
pub struct WKClient {
    base_url: Url,
    token: String,
    client: Client,
    version: &'static str,
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
        }
    }

    fn add_required_headers(&self, req: RequestBuilder) -> RequestBuilder {
        req.bearer_auth(&self.token)
            .header(REVISION_HEADER, self.version)
    }

    fn rate_limit_reset(&self, headers: &HeaderMap) -> Timestamp {
        let header_val = headers.get("Ratelimit-Reset");
        let reset = match header_val {
            Some(header_val) => {
                let reset_str = header_val.to_str().expect("Header should be string");
                reset_str.parse().unwrap_or_else(|_| {
                    log::warn!("Ratelimit-Reset header is not a number, is \"{reset_str}\"");
                    0
                })
            }
            None => {
                log::warn!("Ratelimit-Reset header not found");
                0
            }
        };

        let naive_datetime =
            NaiveDateTime::from_timestamp_millis(reset * 1000).expect("Valid range");
        DateTime::from_utc(naive_datetime, Utc)
    }

    async fn handle_error(&self, response: Response) -> Error {
        let status = response.status();
        let headers = response.headers().to_owned();
        log::error!("Status code {status} received");
        match response.json::<WanikaniError>().await {
            Ok(error) => {
                if status == StatusCode::TOO_MANY_REQUESTS {
                    Error::RateLimit {
                        error,
                        reset_time: self.rate_limit_reset(&headers),
                    }
                } else {
                    error.into()
                }
            }
            Err(e) => e.into(),
        }
    }

    #[cfg(feature = "summary")]
    /// Get a summary report of available and upcoming lessons and reviews.
    pub async fn get_summary(&self) -> Result<Resource, Error> {
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
            _ => Err(self.handle_error(resp).await),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use chrono::{Duration, Utc};
    use reqwest::Client;

    use super::WKClient;
    use crate::{init_tests, Resource, Timestamp};

    fn create_client() -> WKClient {
        WKClient::new(
            env::var("API_KEY").expect("API_KEY provided"),
            Client::default(),
        )
    }

    #[test]
    fn test_duration_calculation() {
        let duration = Duration::seconds(10);
        let now = Utc::now();
        let reset_time: Timestamp = now + duration;

        let new_duration = (reset_time - now).to_std().expect("In range");
        assert_eq!(new_duration.as_secs(), 10)
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

    #[cfg(feature = "summary")]
    #[tokio::test]
    #[ignore]
    async fn test_rate_limiting() {
        use crate::Error;

        init_tests();

        let client = create_client();

        let error = loop {
            if let Err(e) = client.get_summary().await {
                break e;
            }
        };

        let Error::RateLimit { error, reset_time } = error else {
            panic!("Didn't get rate-limited");
        };

        let wait_period = reset_time - Utc::now();

        assert_eq!(error.code, 429);
        assert_eq!(error.error.expect("Some message"), "Rate limit exceeded");
        assert!(wait_period.num_seconds() < 60);
    }
}
