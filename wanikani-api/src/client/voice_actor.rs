use crate::{voice_actor::VoiceActor, Collection, Error, Resource, Id};

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
    pub async fn get_specific_voice_actor(&self, id: Id) -> Result<Resource<VoiceActor>, Error> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("Valid URL")
            .push(VO_PATH)
            .push(&id.to_string());

        let req = self.client.get(url);

        self.do_request("get_specific_voice_actor", req).await
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use crate::{
        client::{create_client, init_tests, IdFilter},
        Timestamp,
    };

    fn get_timestamp() -> Timestamp {
        Utc::now() - Duration::seconds(10)
    }

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
    #[tokio::test]
    async fn test_get_specific_voice_actor() {
        init_tests();

        let client = create_client();

        assert!(client.get_specific_voice_actor(1).await.is_ok());
    }
}
