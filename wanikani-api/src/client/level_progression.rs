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

#[cfg(test)]
mod tests {
    use crate::client::{init_tests, create_client, IdFilter};

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
}