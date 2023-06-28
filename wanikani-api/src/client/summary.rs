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
    use crate::client::{init_tests, create_client};

    #[tokio::test]
    async fn test_get_summary() {
        init_tests();

        let client = create_client();

        assert!(client.get_summary().await.is_ok());
    }

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