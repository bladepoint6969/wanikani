use url::Url;

use crate::{
    assignment::{Assignment, AssignmentStart},
    cross_feature::SubjectType,
    Collection, Error, Id, Resource, Timestamp,
};

use super::{Filter, WKClient};

const ASSIGNMENT_PATH: &str = "assignments";

impl WKClient {
    /// Returns a collection of all assignments, ordered by ascending
    /// `created_at`, 1000 at a time.
    pub async fn get_assignments(
        &self,
        filters: &AssignmentFilter,
    ) -> Result<Collection<Assignment>, Error> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("Valid URL")
            .push(ASSIGNMENT_PATH);

        filters.apply_filters(&mut url);

        let req = self.client.get(url);

        self.do_request("get_assignments", req).await
    }

    /// Retrieves a specific assignment by its `id`.
    pub async fn get_specific_assignment(&self, id: Id) -> Result<Resource<Assignment>, Error> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("Valid URL")
            .push(ASSIGNMENT_PATH)
            .push(&id.to_string());

        let req = self.client.get(url);

        self.do_request("get_specific_assignment", req).await
    }

    /// Mark the assignment as started, moving the assignment from the lessons
    /// queue to the review queue. Returns the updated assignment.
    ///
    /// ## Expected Starting State
    ///
    /// The assignment must be in the following valid state:
    ///
    /// Attribute     | State
    /// --------------|----------
    /// `level`       | Must be less than or equal to the lowest value of User's `level` and `subscription.max_level_granted`
    /// `srs_stage`   | Must be equal to `0`
    /// `started_at`  | Must be `null`
    /// `unlocked_at` | Must not be `null`
    pub async fn start_assignment(
        &self,
        id: Id,
        body: &AssignmentStart,
    ) -> Result<Resource<Assignment>, Error> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("Valid URL")
            .push(ASSIGNMENT_PATH)
            .push(&id.to_string())
            .push("start");

        let req = self.client.put(url).json(body);

        self.do_request("start_assignment", req).await
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Filter parameters for subjects
pub struct AssignmentFilter {
    /// Only assignments available at or after this time are returned.
    pub available_after: Option<Timestamp>,
    /// Only assignments available at or before this time are returned.
    pub available_before: Option<Timestamp>,
    /// When set to `true`, returns assignments that have a value in
    /// `data.burned_at`. Returns assignments with a `null` `data.burned_at` if
    /// `false`.
    pub burned: Option<bool>,
    /// Return assignments with a matching value in the `hidden` attribute
    pub hidden: Option<bool>,
    /// Only assignments where `data.id` matches one of the array values are
    /// returned.
    pub ids: Option<Vec<Id>>,
    /// Returns assignments which are immediately available for lessons
    pub immediately_available_for_lessons: bool,
    /// Returns assignments which are immediately available for review
    pub immediately_available_for_review: bool,
    /// Returns assignments which are in the review state
    pub in_review: bool,
    /// Only assignments where the associated subject level matches one of the
    /// array values are returned.
    pub levels: Option<Vec<u32>>,
    /// Only assignments where `data.srs_stage` matches one of the array values
    /// are returned.
    pub srs_stages: Option<Vec<u32>>,
    /// When set to `true`, returns assignments that have a value in
    /// `data.started_at`. Returns assignments with a `null` `data.started_at`
    /// if `false`.
    pub started: Option<bool>,
    /// Only assignments where `data.subject_id` matches one of the array values
    /// are returned.
    pub subject_ids: Option<Vec<Id>>,
    /// Only assignments where `data.subject_type` matches one of the array
    /// values are returned.
    pub subject_types: Option<Vec<SubjectType>>,
    /// When set to `true`, returns assignments that have a value in
    /// `data.unlocked_at`. Returns assignments with a `null` `data.unlocked_at`
    /// if `false`.
    pub unlocked: Option<bool>,
    /// Only assignments updated after this time are returned.
    pub updated_after: Option<Timestamp>,
}

impl Filter for AssignmentFilter {
    fn apply_filters(&self, url: &mut Url) {
        let mut query = url.query_pairs_mut();
        if let Some(ref value) = self.available_after {
            query.append_pair("available_after", value.to_rfc3339().as_str());
        }
        if let Some(ref value) = self.available_before {
            query.append_pair("available_before", value.to_rfc3339().as_str());
        }
        if let Some(ref value) = self.burned {
            query.append_pair("burned", value.to_string().as_str());
        }
        if let Some(ref value) = self.hidden {
            query.append_pair("hidden", value.to_string().as_str());
        }
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
        if self.immediately_available_for_lessons {
            query.append_key_only("immediately_available_for_lessons");
        }
        if self.immediately_available_for_review {
            query.append_key_only("immediately_available_for_review");
        }
        if self.in_review {
            query.append_key_only("in_review");
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
        if let Some(ref value) = self.srs_stages {
            query.append_pair(
                "srs_stages",
                value
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
                    .as_str(),
            );
        }
        if let Some(value) = self.started {
            query.append_pair("started", value.to_string().as_str());
        }
        if let Some(ref value) = self.subject_ids {
            query.append_pair(
                "subject_ids",
                value
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
                    .as_str(),
            );
        }
        if let Some(ref value) = self.subject_types {
            query.append_pair(
                "subject_types",
                value
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
                    .as_str(),
            );
        }
        if let Some(value) = self.unlocked {
            query.append_pair("unlocked", value.to_string().as_str());
        }
        if let Some(updated_after) = self.updated_after {
            query.append_pair("updated_after", updated_after.to_rfc3339().as_str());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::client::{create_client, init_tests};

    #[tokio::test]
    async fn test_get_assignments() {
        use super::AssignmentFilter;

        init_tests();

        let client = create_client();
        let filters = AssignmentFilter {
            levels: Some(vec![2]),
            ..AssignmentFilter::default()
        };
        assert!(client.get_assignments(&filters).await.is_ok());
    }
    #[tokio::test]
    async fn test_get_specific_assignment() {
        init_tests();

        let client = create_client();
        let assignments = client
            .get_assignments(&Default::default())
            .await
            .expect("Get all assignments");

        if let Some(assignment) = assignments.data.get(0) {
            assert!(client.get_specific_assignment(assignment.id).await.is_ok());
        } else {
            log::warn!("No assignments detected, this test should not be considered reliable");
        }
    }
}
