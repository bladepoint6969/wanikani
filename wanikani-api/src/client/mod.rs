//! HTTP client implementation for consuming the WaniKani API

use std::{any::type_name, fmt::Debug};

use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::{header::HeaderMap, Client, RequestBuilder, Response, StatusCode};
use serde::Deserialize;
use url::Url;

use crate::{Error, Timestamp, WanikaniError, API_VERSION, URL_BASE, Id};

const REVISION_HEADER: &str = "Wanikani-Revision";

pub(crate) trait Filter {
    fn apply_filters(&self, url: &mut Url);
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Filter parameters for voice actor collections
pub struct IdFilter {
    /// Only resources where `data.id` matches one of the array values are returned.
    pub ids: Option<Vec<Id>>,
    /// Only resources updated after this time are returned.
    pub updated_after: Option<Timestamp>,
}

impl Filter for IdFilter {
    fn apply_filters(&self, url: &mut url::Url) {
        let mut query = url.query_pairs_mut();
        if let Some(ref ids) = self.ids {
            query.append_pair(
                "ids",
                ids.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
                    .as_str(),
            );
        }
        if let Some(updated_after) = self.updated_after {
            query.append_pair("updated_after", updated_after.to_rfc3339().as_str());
        }
    }
}

#[cfg(feature = "level_progression")]
mod level_progression;

#[cfg(feature = "reset")]
mod reset;

#[cfg(feature = "review_statistic")]
mod review_statistic;

#[cfg(feature = "study_material")]
mod study_material;

#[cfg(feature = "study_material")]
pub use study_material::StudyMaterialFilter;

#[cfg(feature = "subject")]
mod subject;

#[cfg(feature = "subject")]
pub use subject::SubjectFilter;

#[cfg(feature = "summary")]
mod summary;

#[cfg(feature = "user")]
mod user;

#[cfg(feature = "voice_actor")]
mod voice_actor;

/// The WaniKani client struct performs requests to the API.
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
        const MILLIS_IN_SECOND: i64 = 1000;

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
            NaiveDateTime::from_timestamp_millis(reset * MILLIS_IN_SECOND).expect("Valid range");
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

    /// Fetch a resource by its URL.
    ///
    /// This can be used for easily following the `next_url` trail of
    /// collections, or for refreshing a resource by its `url`.
    ///
    /// ### Example
    /// ```rust
    /// # use wanikani_api::{Collection, Pages, Error, client::WKClient};
    /// # type VoiceActor = serde_json::Value;
    /// # let client = WKClient::new("MY_TOKEN".to_string(), reqwest::Client::default());
    /// # async move {
    /// # let pages = Pages {
    /// #     next_url: Some("https://api.wanikani.com/v2/level_progressions".parse().unwrap()),
    /// #     previous_url: None, per_page: 500};
    /// // let collection: Collection<VoiceActor> = ...;
    /// // let pages = collection.pages;
    ///
    /// if let Some(ref url) = pages.next_url {
    ///     let next_collection: Collection<VoiceActor> = client
    ///         .get_resource_by_url(url)
    ///         .await
    ///         .unwrap();
    /// }
    /// # };
    pub async fn get_resource_by_url<T>(&self, url: &Url) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let fn_signature = format!("get_resource_by_url<{}>", type_name::<T>());

        let req = self.client.get(url.to_owned());

        self.do_request(&fn_signature, req).await
    }

    async fn do_request<T>(&self, caller: &str, req: RequestBuilder) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let req = self.add_required_headers(req);

        log::debug!("{caller} request: {req:?}");

        let resp = req.send().await?;

        log::debug!("{caller} response: {resp:?}");

        match resp.status() {
            StatusCode::OK => Ok(resp.json().await?),
            _ => Err(self.handle_error(resp).await),
        }
    }
}

#[cfg(test)]
static INIT: std::sync::OnceLock<()> = std::sync::OnceLock::new();

#[cfg(test)]
fn init_tests() {
    INIT.get_or_init(|| {
        dotenvy::dotenv().ok();
        env_logger::init()
    });
}

#[cfg(test)]
fn create_client() -> WKClient {
    WKClient::new(
        std::env::var("API_KEY").expect("API_KEY provided"),
        Client::default(),
    )
}

#[cfg(test)]
mod tests {
    use crate::{Collection, URL_BASE};

    use super::{create_client, init_tests};

    #[tokio::test]
    #[ignore]
    async fn test_rate_limiting() {
        use chrono::{DateTime, Duration, Local, Utc};
        use tokio::time::Instant;

        use crate::Error;

        init_tests();

        let client = create_client();

        let url = format!("{}/subjects?levels=5000", URL_BASE)
            .parse()
            .expect("URL");

        let error = loop {
            if let Err(e) = client.get_resource_by_url::<Collection<()>>(&url).await {
                break e;
            }
        };

        let Error::RateLimit { error, reset_time } = error else {
            panic!("Didn't get rate-limited");
        };

        let wait_period = reset_time - Utc::now();

        log::info!(
            "Reset time is {} Wait period is {wait_period}",
            DateTime::<Local>::from(reset_time)
        );

        assert_eq!(error.code, 429);
        assert_eq!(error.error.expect("Some message"), "Rate limit exceeded");
        assert!(wait_period.num_seconds() < 60);
        assert!(wait_period.num_milliseconds() > 0);

        tokio::time::sleep_until(
            Instant::now()
                + (wait_period + Duration::seconds(1))
                    .to_std()
                    .expect("Should be short"),
        )
        .await;

        assert!(client
            .get_resource_by_url::<Collection<()>>(&url)
            .await
            .is_ok())
    }
}
