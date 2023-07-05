//! Review statistics summarize the activity recorded in reviews. They contain
//! sum the number of correct and incorrect answers for both meaning and
//! reading. They track current and maximum streaks of correct answers. They
//! store the overall percentage of correct answers versus total answers.
//!
//! A review statistic is created when the user has done their first review on
//! the related subject.

use serde::{Deserialize, Serialize};

use crate::{cross_feature::SubjectType, Id, Timestamp};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
/// Review statistics summarize the activity recorded in reviews. They contain
/// sum the number of correct and incorrect answers for both meaning and
/// reading. They track current and maximum streaks of correct answers. They
/// store the overall percentage of correct answers versus total answers.
///
/// A review statistic is created when the user has done their first review on
/// the related subject.
pub struct ReviewStatistic {
    /// Timestamp when the review statistic was created.
    pub created_at: Timestamp,
    /// Indicates if the associated subject has been hidden, preventing it from
    /// appearing in lessons or reviews.
    pub hidden: bool,
    /// Total number of correct answers submitted for the meaning of the
    /// associated subject.
    pub meaning_correct: u32,
    /// The current, uninterrupted series of correct answers given for the
    /// meaning of the associated subject.
    pub meaning_current_streak: u32,
    /// Total number of incorrect answers submitted for the meaning of the
    /// associated subject.
    pub meaning_incorrect: u32,
    /// The longest, uninterrupted series of correct answers ever given for the
    /// meaning of the associated subject.
    pub meaning_max_streak: u32,
    /// The overall correct answer rate by the user for the subject, including
    /// both meaning and reading.
    pub percentage_correct: u32,
    /// Total number of correct answers submitted for the reading of the
    /// associated subject.
    pub reading_correct: u32,
    /// The current, uninterrupted series of correct answers given for the
    /// reading of the associated subject.
    pub reading_current_streak: u32,
    /// Total number of incorrect answers submitted for the reading of the
    /// associated subject.
    pub reading_incorrect: u32,
    /// The longest, uninterrupted series of correct answers ever given for the
    /// reading of the associated subject.
    pub reading_max_streak: u32,
    /// Unique identifier of the associated subject.
    pub subject_id: Id,
    /// The type of the associated subject.
    pub subject_type: SubjectType,
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use crate::{cross_feature::SubjectType, Resource, ResourceCommon, ResourceType};

    use super::ReviewStatistic;

    #[test]
    fn test_deserialize_review_statistic() {
        let json = include_str!("../test_files/review_statistic.json");

        let stat: Resource<ReviewStatistic> = serde_json::from_str(json).expect("Deserialize");

        let expected_stat = Resource {
            id: 80461982,
            common: crate::ResourceCommon {
                object: ResourceType::ReviewStatistic,
                url: "https://api.wanikani.com/v2/review_statistics/80461982"
                    .parse()
                    .expect("URL"),
                data_updated_at: Some(
                    DateTime::parse_from_rfc3339("2018-04-03T11:50:31.558505Z")
                        .expect("Timestamp")
                        .into(),
                ),
            },
            data: ReviewStatistic {
                created_at: DateTime::parse_from_rfc3339("2017-09-05T23:38:10.964821Z")
                    .expect("Timestamp")
                    .into(),
                subject_id: 8761,
                subject_type: SubjectType::Radical,
                meaning_correct: 8,
                meaning_incorrect: 0,
                meaning_max_streak: 8,
                meaning_current_streak: 8,
                reading_correct: 0,
                reading_incorrect: 0,
                reading_current_streak: 0,
                reading_max_streak: 0,
                percentage_correct: 100,
                hidden: false,
            },
        };

        assert_eq!(stat, expected_stat);
    }

    #[test]
    fn test_serialize_review_statistic() {
        let data = ReviewStatistic {
            created_at: Utc::now(),
            hidden: true,
            meaning_correct: 5,
            meaning_current_streak: 3,
            meaning_incorrect: 2,
            meaning_max_streak: 3,
            percentage_correct: 65,
            reading_correct: 8,
            reading_current_streak: 8,
            reading_incorrect: 0,
            reading_max_streak: 8,
            subject_id: 69420,
            subject_type: SubjectType::KanaVocabulary,
        };
        let common = ResourceCommon {
            data_updated_at: Some(Utc::now()),
            object: ResourceType::ReviewStatistic,
            url: "https://some.url/5555".parse().expect("URL"),
        };
        let stat = Resource {
            common,
            data,
            id: 69420,
        };

        let json = serde_json::to_string(&stat).expect("Serialize");

        let new_stat = serde_json::from_str(&json).expect("Deserialize");

        assert_eq!(stat, new_stat);
    }
}
