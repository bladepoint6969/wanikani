//! HTTP client implementation for consuming the WaniKani API

use std::{any::type_name, fmt::Debug};

use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::{header::HeaderMap, Client, RequestBuilder, Response, StatusCode};
use serde::Deserialize;
use url::Url;

use crate::{Error, Timestamp, WanikaniError, API_VERSION, URL_BASE};

const REVISION_HEADER: &str = "Wanikani-Revision";

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
    /// # use wanikani_api::{Collection, Error, voice_actor::VoiceActor, client::WKClient};
    /// # let client = WKClient::new("MY_TOKEN".to_string(), reqwest::Client::default());
    /// # async move {
    /// let collection: Collection<VoiceActor> = client.get_voice_actors().await.unwrap();
    ///
    /// if let Some(ref url) = collection.pages.next_url {
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

#[cfg(feature = "summary")]
mod summary {
    use crate::{summary::Summary, Error};

    use super::WKClient;

    const SUMMARY_PATH: &str = "summary";

    impl WKClient {
        /// Get a summary report of available and upcoming lessons and reviews.
        pub async fn get_summary(&self) -> Result<Summary, Error> {
            let mut url = self.base_url.clone();
            url.path_segments_mut()
                .expect("Valid URL")
                .push(SUMMARY_PATH);

            let req = self.client.get(url);

            self.do_request("get_summary", req).await
        }
    }
}

#[cfg(feature = "user")]
mod user {
    use crate::{
        user::{UpdateUser, User},
        Error,
    };

    use super::WKClient;

    const USER_PATH: &str = "user";

    impl WKClient {
        /// Returns a summary of user information.
        pub async fn get_user_information(&self) -> Result<User, Error> {
            let mut url = self.base_url.clone();
            url.path_segments_mut().expect("Valid URL").push(USER_PATH);

            let req = self.client.get(url);

            self.do_request("get_user_information", req).await
        }

        /// Returns an updated summary of user information.
        pub async fn update_user_information(&self, user: &UpdateUser) -> Result<User, Error> {
            let mut url = self.base_url.clone();
            url.path_segments_mut().expect("Valid URL").push(USER_PATH);

            let req = self.client.put(url).json(user);

            self.do_request("update_user_information", req).await
        }
    }
}

#[cfg(feature = "voice_actor")]
mod voice_actor {
    use crate::{voice_actor::VoiceActor, Collection, Error, Resource};

    use super::WKClient;

    const VO_PATH: &str = "voice_actors";

    impl WKClient {
        /// Returns a collection of all voice actors, ordered by ascending
        /// `created_at`, 500 at a time.
        pub async fn get_voice_actors(&self) -> Result<Collection<VoiceActor>, Error> {
            let mut url = self.base_url.clone();
            url.path_segments_mut().expect("Valid URL").push(VO_PATH);

            let req = self.client.get(url);

            self.do_request("get_voice_actors", req).await
        }

        /// Retrieves a specific voice_actor by its `id`.
        pub async fn get_specific_voice_actor(
            &self,
            id: u32,
        ) -> Result<Resource<VoiceActor>, Error> {
            let mut url = self.base_url.clone();
            url.path_segments_mut()
                .expect("Valid URL")
                .push(VO_PATH)
                .push(&id.to_string());

            let req = self.client.get(url);

            self.do_request("get_specific_voice_actor", req).await
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use chrono::{Duration, Utc};
    use reqwest::Client;

    use super::WKClient;

    static INIT: std::sync::OnceLock<()> = std::sync::OnceLock::new();

    fn init_tests() {
        INIT.get_or_init(|| {
            dotenvy::dotenv().ok();
            env_logger::init()
        });
    }

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

        assert!(client.get_summary().await.is_ok());
    }

    #[cfg(feature = "user")]
    #[tokio::test]
    async fn test_get_user_information() {
        init_tests();

        let client = create_client();

        assert!(client.get_user_information().await.is_ok());
    }

    #[cfg(feature = "user")]
    #[tokio::test]
    async fn test_update_user_information() {
        use crate::user::{UpdatePreferences, UpdateUser};

        init_tests();

        let client = create_client();

        let user = client.get_user_information().await.expect("Success");

        let preferences = UpdatePreferences {
            default_voice_actor_id: Some(2),
            ..user.data.preferences.into()
        };
        let mut update = UpdateUser { preferences };

        let updated_user = client
            .update_user_information(&update)
            .await
            .expect("Success");

        assert_ne!(updated_user, user);
        assert_eq!(updated_user.data.preferences.default_voice_actor_id, 2);
        assert!(
            updated_user.common.data_updated_at.expect("Timestamp")
                > user.common.data_updated_at.expect("Timestamp")
        );

        update.preferences = user.data.preferences.into();
        let reset_user = client
            .update_user_information(&update)
            .await
            .expect("Success");

        assert_eq!(reset_user.data, user.data);
        assert!(
            reset_user.common.data_updated_at.expect("Timestamp")
                > updated_user.common.data_updated_at.expect("Timestamp")
        );
    }

    #[cfg(feature = "voice_actor")]
    #[tokio::test]
    async fn test_get_voice_actors() {
        init_tests();

        let client = create_client();

        assert!(client.get_voice_actors().await.is_ok());
    }

    #[cfg(feature = "voice_actor")]
    #[tokio::test]
    async fn test_get_specific_voice_actor() {
        init_tests();

        let client = create_client();

        assert!(client.get_specific_voice_actor(1).await.is_ok());
    }

    #[cfg(feature = "summary")]
    #[tokio::test]
    #[ignore]
    async fn test_rate_limiting() {
        use chrono::{DateTime, Local};
        use tokio::time::Instant;

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

        assert!(client.get_summary().await.is_ok())
    }
}
