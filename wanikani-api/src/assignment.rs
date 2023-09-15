//! Assignments contain information about a user's progress on a particular
//! subject, including their current state and timestamps for various progress
//! milestones. Assignments are created when a user has passed all the
//! components of the given subject and the assignment is at or below their
//! current level for the first time.

use serde::{Deserialize, Serialize};

use crate::{cross_feature::SubjectType, Id, Timestamp};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
/// Assignments contain information about a user's progress on a particular
/// subject, including their current state and timestamps for various progress
/// milestones. Assignments are created when a user has passed all the
/// components of the given subject and the assignment is at or below their
/// current level for the first time.
pub struct Assignment {
    /// Timestamp when the related subject will be available in the user's
    /// review queue.
    pub available_at: Option<Timestamp>,
    /// Timestamp when the user reaches SRS stage `9` the first time.
    pub burned_at: Option<Timestamp>,
    /// Timestamp when the assignment was created.
    pub created_at: Timestamp,
    /// Indicates if the associated subject has been hidden, preventing it from
    /// appearing in lessons or reviews.
    pub hidden: bool,
    /// Timestamp when the user reaches SRS stage `5` for the first time.
    pub passed_at: Option<Timestamp>,
    /// Timestamp when the subject is resurrected and placed back in the user's
    /// review queue.
    pub resurrected_at: Option<Timestamp>,
    /// The current SRS stage interval. The interval range is determined by the
    /// related subject's spaced repetition system.
    pub srs_stage: u32,
    /// Timestamp when the user completes the lesson for the related subject.
    pub started_at: Option<Timestamp>,
    /// Unique identifier of the associated subject.
    pub subject_id: Id,
    /// The type of the associated subject.
    pub subject_type: SubjectType,
    /// The timestamp when the related subject has its prerequisites satisfied
    /// and is made available in lessons.
    ///
    /// Prerequisites are:
    ///
    /// - The subject components have reached SRS stage `5` once (they have been
    /// "passed").
    /// - The user's level is equal to or greater than the level of the
    /// assignment’s subject.
    pub unlocked_at: Option<Timestamp>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
/// Mark the assignment as started, moving the assignment from the lessons queue
/// to the review queue. Returns the updated assignment.
pub struct AssignmentStart {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// If not set, `started_at` will default to the time the request is made.
    ///
    /// `started_at` must be greater than or equal to `unlocked_at`.
    pub started_at: Option<Timestamp>,
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use crate::{cross_feature::SubjectType, Resource, ResourceCommon, ResourceType};

    use super::Assignment;

    #[test]
    fn test_deserialize_assignment() {
        let json = include_str!("../test_files/assignment.json");

        let assignment: Resource<Assignment> = serde_json::from_str(json).expect("Deserialize");

        assert_eq!(assignment.id, 80463006);

        let common = assignment.common;
        assert_eq!(common.object, ResourceType::Assignment);
        assert_eq!(
            common.url,
            "https://api.wanikani.com/v2/assignments/80463006"
                .parse()
                .expect("URL")
        );
        assert_eq!(
            common.data_updated_at.expect("Timestamp"),
            DateTime::parse_from_rfc3339("2017-10-30T01:51:10.438432Z").expect("Timestamp")
        );

        let data = assignment.data;
        assert_eq!(
            data.created_at,
            DateTime::parse_from_rfc3339("2017-09-05T23:38:10.695133Z").expect("Timestamp")
        );
        assert_eq!(data.subject_id, 8761);
        assert_eq!(data.subject_type, SubjectType::Radical);
        assert_eq!(data.srs_stage, 8);
        assert_eq!(
            data.unlocked_at.expect("Timestamp"),
            DateTime::parse_from_rfc3339("2017-09-05T23:38:10.695133Z").expect("Timestamp")
        );
        assert_eq!(
            data.started_at.expect("Timestamp"),
            DateTime::parse_from_rfc3339("2017-09-05T23:41:28.980679Z").expect("Timestamp")
        );
        assert_eq!(
            data.passed_at.expect("Timestamp"),
            DateTime::parse_from_rfc3339("2017-09-07T17:14:14.491889Z").expect("Timestamp")
        );
        assert!(data.burned_at.is_none());
        assert_eq!(
            data.available_at.expect("Timestamp"),
            DateTime::parse_from_rfc3339("2018-02-27T00:00:00.000000Z").expect("Timestamp")
        );
        assert!(data.resurrected_at.is_none());
        assert!(!data.hidden);
    }

    #[test]
    fn test_serialize_assignment() {
        let data = Assignment {
            available_at: Some(Utc::now()),
            burned_at: None,
            created_at: Utc::now(),
            hidden: true,
            passed_at: Some(Utc::now()),
            resurrected_at: None,
            srs_stage: 2,
            started_at: Some(Utc::now()),
            subject_id: 6969,
            subject_type: SubjectType::KanaVocabulary,
            unlocked_at: None,
        };
        let common = ResourceCommon {
            data_updated_at: None,
            object: ResourceType::Assignment,
            url: "https://some.url/assignment".parse().expect("URL"),
        };
        let assignment = Resource {
            common,
            data,
            id: 69420,
        };

        let json = serde_json::to_string(&assignment).expect("Serialize");

        let new_assignment: Resource<Assignment> =
            serde_json::from_str(&json).expect("Deserialize");

        assert_eq!(assignment, new_assignment);
    }
}
