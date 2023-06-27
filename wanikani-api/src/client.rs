//! HTTP client implementation for consuming the WaniKani API

use std::{any::type_name, fmt::Debug};

use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::{header::HeaderMap, Client, RequestBuilder, Response, StatusCode};
use serde::Deserialize;
use url::Url;

use crate::{Error, Timestamp, WanikaniError, API_VERSION, URL_BASE};

const REVISION_HEADER: &str = "Wanikani-Revision";

pub(crate) trait Filter {
    fn apply_filters(&self, url: &mut Url);
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Filter parameters for voice actor collections
pub struct IdFilter {
    /// Only resources where `data.id` matches one of the array values are returned.
    pub ids: Option<Vec<u64>>,
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

#[cfg(feature = "study_material")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// The collection of study material records will be filtered on the parameters
/// provided.
pub struct StudyMaterialFilter {
    /// Return study materials with a matching value in the `hidden` attribute.
    pub hidden: Option<bool>,
    /// Only study material records where `data.id` matches one of the array
    /// values are returned.
    pub ids: Option<Vec<u64>>,
    /// Only study material records where `data.subject_id` matches one of the
    /// array values are returned.
    pub subject_ids: Option<Vec<u64>>,
    /// Only study material records where `data.subject_type` matches one of the
    /// array values are returned.
    pub subject_types: Option<Vec<crate::subject::SubjectType>>,
    /// Only study material records updated after this time are returned.
    pub updated_after: Option<Timestamp>,
}

#[cfg(feature = "study_material")]
impl Filter for StudyMaterialFilter {
    fn apply_filters(&self, url: &mut Url) {
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
        if let Some(ref ids) = self.subject_ids {
            query.append_pair(
                "subject_ids",
                ids.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
                    .as_str(),
            );
        }
        if let Some(ref types) = self.subject_types {
            query.append_pair(
                "subject_types",
                types
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
                    .as_str(),
            );
        }
        if let Some(hidden) = self.hidden {
            query.append_pair("hidden", hidden.to_string().as_str());
        }
        if let Some(updated_after) = self.updated_after {
            query.append_pair("updated_after", updated_after.to_rfc3339().as_str());
        }
    }
}

#[cfg(feature = "subject")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Filter parameters for subjects
pub struct SubjectFilter {
    /// Only subjects where `data.id` matches one of the array values are
    /// returned.
    pub ids: Option<Vec<u64>>,
    /// Return subjects of the specified types.
    pub types: Option<Vec<crate::subject::SubjectType>>,
    /// Return subjects of the specified slug.
    pub slugs: Option<Vec<String>>,
    /// Return subjects at the specified levels.
    pub levels: Option<Vec<u32>>,
    /// Return subjects which are or are not hidden from the user-facing
    /// application.
    pub hidden: Option<bool>,
    /// Only subjects updated after this time are returned.
    pub updated_after: Option<Timestamp>,
}

#[cfg(feature = "subject")]
impl Filter for SubjectFilter {
    fn apply_filters(&self, url: &mut Url) {
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
        if let Some(ref types) = self.types {
            query.append_pair(
                "types",
                types
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
                    .as_str(),
            );
        }
        if let Some(ref slugs) = self.slugs {
            query.append_pair("slugs", slugs.join(",").as_str());
        }
        if let Some(ref levels) = self.levels {
            query.append_pair(
                "levels",
                levels
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
                    .as_str(),
            );
        }
        if let Some(hidden) = self.hidden {
            query.append_pair("hidden", hidden.to_string().as_str());
        }
        if let Some(updated_after) = self.updated_after {
            query.append_pair("updated_after", updated_after.to_rfc3339().as_str());
        }
    }
}

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

    use super::{Filter, IdFilter, WKClient};

    const VO_PATH: &str = "voice_actors";

    impl WKClient {
        /// Returns a collection of all voice actors, ordered by ascending
        /// `created_at`, 500 at a time.
        pub async fn get_voice_actors(
            &self,
            filters: &IdFilter,
        ) -> Result<Collection<VoiceActor>, Error> {
            let mut url = self.base_url.clone();
            url.path_segments_mut().expect("Valid URL").push(VO_PATH);

            filters.apply_filters(&mut url);

            let req = self.client.get(url);

            self.do_request("get_voice_actors", req).await
        }

        /// Retrieves a specific voice_actor by its `id`.
        pub async fn get_specific_voice_actor(
            &self,
            id: u64,
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

#[cfg(feature = "level_progression")]
mod level_progression {
    use crate::{level_progression::LevelProgression, Collection, Error, Resource};

    use super::{Filter, IdFilter, WKClient};

    const PROG_PATH: &str = "level_progressions";

    impl WKClient {
        /// Returns a collection of all level progressions, ordered by ascending
        /// `created_at`, 500 at a time.
        pub async fn get_level_progressions(
            &self,
            filters: &IdFilter,
        ) -> Result<Collection<LevelProgression>, Error> {
            let mut url = self.base_url.clone();
            url.path_segments_mut().expect("Valid URL").push(PROG_PATH);

            filters.apply_filters(&mut url);

            let req = self.client.get(url);

            self.do_request("get_level_progressions", req).await
        }

        /// Retrieves a specific level progression by its id.
        pub async fn get_specific_level_progression(
            &self,
            id: u64,
        ) -> Result<Resource<LevelProgression>, Error> {
            let mut url = self.base_url.clone();
            url.path_segments_mut()
                .expect("Valid URL")
                .push(PROG_PATH)
                .push(&id.to_string());

            let req = self.client.get(url);

            self.do_request("get_specific_level_progression", req).await
        }
    }
}

#[cfg(feature = "subject")]
mod subject {
    use crate::{
        subject::{Subject, WaniKaniSubject},
        Collection, Error, Resource,
    };

    use super::{Filter, SubjectFilter, WKClient};

    const SUBJECT_PATH: &str = "subjects";

    impl WKClient {
        /// Returns a collection of all subjects, ordered by ascending
        /// `created_at`, 1000 at a time.
        pub async fn get_subjects(
            &self,
            filters: &SubjectFilter,
        ) -> Result<Collection<Subject>, Error> {
            let mut url = self.base_url.clone();
            url.path_segments_mut()
                .expect("Valid URL")
                .push(SUBJECT_PATH);

            filters.apply_filters(&mut url);

            let req = self.client.get(url);

            self.do_request("get_subjects", req).await
        }

        /// Retrieves a specific subject by its `id`. The structure of the
        /// response depends on the subject type.
        pub async fn get_specific_subject<T: WaniKaniSubject>(
            &self,
            id: u64,
        ) -> Result<Resource<T>, Error> {
            let mut url = self.base_url.clone();
            url.path_segments_mut()
                .expect("Valid URL")
                .push(SUBJECT_PATH)
                .push(&id.to_string());

            let req = self.client.get(url);

            self.do_request("get_specific_subject", req).await
        }
    }
}

#[cfg(feature = "study_material")]
mod study_material {
    use crate::{study_material::StudyMaterial, Collection, Error, Resource};

    use super::{Filter, StudyMaterialFilter, WKClient};

    const STUDY_MATERIAL_PATH: &str = "study_materials";

    impl WKClient {
        /// Returns a collection of all study material, ordered by ascending
        /// `created_at`, 500 at a time.
        pub async fn get_study_materials(
            &self,
            filters: &StudyMaterialFilter,
        ) -> Result<Collection<StudyMaterial>, Error> {
            let mut url = self.base_url.clone();
            url.path_segments_mut()
                .expect("Valid URL")
                .push(STUDY_MATERIAL_PATH);

            filters.apply_filters(&mut url);

            let req = self.client.get(url);

            self.do_request("get_subjects", req).await
        }

        /// Retrieves a specific study material by its `id`.
        pub async fn get_specific_study_material(
            &self,
            id: u64,
        ) -> Result<Resource<StudyMaterial>, Error> {
            let mut url = self.base_url.clone();
            url.path_segments_mut()
                .expect("Valid URL")
                .push(STUDY_MATERIAL_PATH)
                .push(&id.to_string());

            let req = self.client.get(url);

            self.do_request("get_specific_subject", req).await
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use reqwest::Client;
    use std::env;

    use crate::Timestamp;

    use super::{IdFilter, WKClient};

    static INIT: std::sync::OnceLock<()> = std::sync::OnceLock::new();

    fn get_timestamp() -> Timestamp {
        Utc::now() - Duration::seconds(10)
    }

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

        let mut voice_actors = client
            .get_voice_actors(&IdFilter::default())
            .await
            .expect("VOs returned");

        assert_eq!(voice_actors.total_count, 2);
        assert_eq!(voice_actors.data.len(), 2);

        voice_actors = client
            .get_voice_actors(&IdFilter {
                ids: Some(vec![1]),
                ..IdFilter::default()
            })
            .await
            .expect("VO 1");

        assert_eq!(voice_actors.total_count, 1);
        assert_eq!(voice_actors.data.len(), 1);

        voice_actors = client
            .get_voice_actors(&IdFilter {
                updated_after: Some(get_timestamp()),
                ..IdFilter::default()
            })
            .await
            .expect("No VO");

        assert_eq!(voice_actors.total_count, 0);
        assert!(voice_actors.data.is_empty());
    }

    #[cfg(feature = "voice_actor")]
    #[tokio::test]
    async fn test_get_specific_voice_actor() {
        init_tests();

        let client = create_client();

        assert!(client.get_specific_voice_actor(1).await.is_ok());
    }

    #[cfg(feature = "level_progression")]
    #[tokio::test]
    async fn test_get_level_progressions() {
        init_tests();

        let client = create_client();

        assert!(client
            .get_level_progressions(&IdFilter::default())
            .await
            .is_ok());
    }

    #[cfg(feature = "level_progression")]
    #[tokio::test]
    async fn test_get_specific_level_progression() {
        init_tests();

        let client = create_client();
        let progressions = client
            .get_level_progressions(&Default::default())
            .await
            .expect("Get all progs");

        if let Some(prog) = progressions.data.get(0) {
            assert!(client.get_specific_level_progression(prog.id).await.is_ok());
        } else {
            log::warn!(
                "No level progressions detected, this test should not be considered reliable"
            );
        }
    }

    #[cfg(feature = "subject")]
    #[tokio::test]
    async fn test_get_subjects() {
        use super::SubjectFilter;

        init_tests();

        let client = create_client();
        let filters = SubjectFilter {
            levels: Some(vec![2]),
            ..SubjectFilter::default()
        };
        assert!(client.get_subjects(&filters).await.is_ok());
    }

    #[cfg(feature = "subject")]
    #[tokio::test]
    async fn test_get_specific_subject() {
        use crate::{
            subject::{KanaVocabulary, Kanji, Radical, Subject, Vocabulary},
            Resource,
        };

        init_tests();

        let client = create_client();
        let mut subject: Resource<Subject> =
            client.get_specific_subject(1).await.expect("Get subject");
        let radical: Resource<Radical> = client.get_specific_subject(1).await.expect("Get radical");

        let Subject::Radical(subject_inner) = subject.data else {
            panic!("Incorrect type (Should be radical)");
        };

        assert_eq!(subject.id, radical.id);
        assert_eq!(subject.common, radical.common);
        assert_eq!(subject_inner, radical.data);

        subject = client.get_specific_subject(440).await.expect("Get subject");
        let kanji: Resource<Kanji> = client.get_specific_subject(440).await.expect("Get kanji");

        let Subject::Kanji(subject_inner) = subject.data else {
            panic!("Incorrect type (Should be kanji)");
        };

        assert_eq!(subject.id, kanji.id);
        assert_eq!(subject.common, kanji.common);
        assert_eq!(subject_inner, kanji.data);

        subject = client
            .get_specific_subject(2467)
            .await
            .expect("Get subject");
        let vocab: Resource<Vocabulary> =
            client.get_specific_subject(2467).await.expect("Get vocab");

        let Subject::Vocabulary(subject_inner) = subject.data else {
            panic!("Incorrect type (Should be kanji)");
        };

        assert_eq!(subject.id, vocab.id);
        assert_eq!(subject.common, vocab.common);
        assert_eq!(subject_inner, vocab.data);

        subject = client
            .get_specific_subject(9177)
            .await
            .expect("Get subject");
        let vocab: Resource<KanaVocabulary> = client
            .get_specific_subject(9177)
            .await
            .expect("Get kana vocab");

        let Subject::KanaVocabulary(subject_inner) = subject.data else {
            panic!("Incorrect type (Should be kanji)");
        };

        assert_eq!(subject.id, vocab.id);
        assert_eq!(subject.common, vocab.common);
        assert_eq!(subject_inner, vocab.data);
    }

    #[cfg(feature = "study_material")]
    #[tokio::test]
    async fn test_get_study_materials() {
        init_tests();

        let client = create_client();

        assert!(client
            .get_study_materials(&Default::default())
            .await
            .is_ok());
    }

    #[cfg(feature = "study_material")]
    #[tokio::test]
    async fn test_get_specific_study_material() {
        init_tests();

        let client = create_client();

        let study_materials = client
            .get_study_materials(&Default::default())
            .await
            .expect("Get all study_materials");

        if let Some(prog) = study_materials.data.get(0) {
            assert!(client.get_specific_study_material(prog.id).await.is_ok());
        } else {
            log::warn!(
                "No study materials detected, this test should not be considered reliable"
            );
        }
    }

    #[cfg(feature = "summary")]
    #[tokio::test]
    #[ignore]
    async fn test_rate_limiting() {
        use chrono::{DateTime, Duration, Local, Utc};
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
