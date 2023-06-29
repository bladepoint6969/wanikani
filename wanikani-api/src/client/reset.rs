use crate::{reset::Reset, Collection, Error, Resource};

use super::{Filter, IdFilter, WKClient};

const RESET_PATH: &str = "resets";

impl WKClient {
    /// Returns a collection of all resets, ordered by ascending
    /// `created_at`, 500 at a time.
    pub async fn get_resets(&self, filters: &IdFilter) -> Result<Collection<Reset>, Error> {
        let mut url = self.base_url.clone();
        url.path_segments_mut().expect("Valid URL").push(RESET_PATH);

        filters.apply_filters(&mut url);

        let req = self.client.get(url);

        self.do_request("get_resets", req).await
    }

    /// Retrieves a specific reset by its `id`.
    pub async fn get_specific_reset(&self, id: u64) -> Result<Resource<Reset>, Error> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("Valid URL")
            .push(RESET_PATH)
            .push(&id.to_string());

        let req = self.client.get(url);

        self.do_request("get_specific_reset", req).await
    }
}

#[cfg(test)]
mod tests {
    use crate::client::{create_client, init_tests};

    #[tokio::test]
    async fn test_get_resets() {
        init_tests();

        let client = create_client();

        assert!(client.get_resets(&Default::default()).await.is_ok());
    }

    #[cfg(feature = "reset")]
    #[tokio::test]
    async fn test_get_specific_reset() {
        init_tests();

        let client = create_client();

        let resets = client
            .get_resets(&Default::default())
            .await
            .expect("Get all resets");

        if let Some(reset) = resets.data.get(0) {
            assert!(client.get_specific_reset(reset.id).await.is_ok());
        } else {
            log::warn!("No resets detected, this test should not be considered reliable");
        }
    }
}
