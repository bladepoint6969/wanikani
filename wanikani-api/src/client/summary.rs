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

#[cfg(test)]
mod tests {
    use crate::client::{create_client, init_tests};

    #[tokio::test]
    async fn test_get_summary() {
        init_tests();

        let client = create_client();

        assert!(client.get_summary().await.is_ok());
    }
}
