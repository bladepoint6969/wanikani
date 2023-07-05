use crate::{
    cross_feature::SubjectType, review_statistic::ReviewStatistic, Collection, Error, Id, Resource,
    Timestamp,
};

use super::{Filter, WKClient};

const STAT_PATH: &str = "review_statistics";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// The collection of review statistics will be filtered on the parameters provided.
pub struct ReviewStatisticFilter {
    pub hidden: Option<bool>,
    pub ids: Option<Vec<Id>>,
    pub percentages_greater_than: Option<u32>,
    pub percentages_less_than: Option<u32>,
    pub subject_ids: Option<Vec<Id>>,
    pub subject_types: Option<Vec<SubjectType>>,
    pub updated_after: Option<Timestamp>,
}

impl Filter for ReviewStatisticFilter {
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
        if let Some(ref percentages) = self.percentages_greater_than {
            query.append_pair("percentages_greater_than", percentages.to_string().as_str());
        }
        if let Some(ref percentages) = self.percentages_less_than {
            query.append_pair("percentages_less_than", percentages.to_string().as_str());
        }
    }
}

impl WKClient {
    /// Returns a collection of all review statistics, ordered by ascending
    /// `created_at`, 500 at a time.
    pub async fn get_review_statistics(
        &self,
        filters: &ReviewStatisticFilter,
    ) -> Result<Collection<ReviewStatistic>, Error> {
        let mut url = self.base_url.clone();
        url.path_segments_mut().expect("Valid URL").push(STAT_PATH);

        filters.apply_filters(&mut url);

        let req = self.client.get(url);

        self.do_request("get_resets", req).await
    }

    /// Retrieves a specific review statistic by its `id`.
    pub async fn get_specific_review_statistic(
        &self,
        id: Id,
    ) -> Result<Resource<ReviewStatistic>, Error> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("Valid URL")
            .push(STAT_PATH)
            .push(&id.to_string());

        let req = self.client.get(url);

        self.do_request("get_specific_reset", req).await
    }
}

#[cfg(test)]
mod tests {
    use crate::client::{create_client, init_tests};

    #[tokio::test]
    async fn test_get_review_statistics() {
        init_tests();

        let client = create_client();

        assert!(client
            .get_review_statistics(&Default::default())
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_get_specific_review_statistic() {
        init_tests();

        let client = create_client();

        let resets = client
            .get_review_statistics(&Default::default())
            .await
            .expect("Get all review_statistics");

        if let Some(reset) = resets.data.get(0) {
            assert!(client.get_specific_review_statistic(reset.id).await.is_ok());
        } else {
            log::warn!(
                "No review statistics detected, this test should not be considered reliable"
            );
        }
    }
}
