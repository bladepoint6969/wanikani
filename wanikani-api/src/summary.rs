//! The summary report contains currently available lessons and reviews and the
//! reviews that will become available in the next 24 hours, grouped by the
//! hour.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ResourceCommon;

#[derive(Debug, Clone, Deserialize, Serialize)]
/// The summary report contains currently available lessons and reviews and the
/// reviews that will become available in the next 24 hours, grouped by the
/// hour.
pub struct Summary {
    #[serde(flatten)]
    /// Common resource data
    pub common: ResourceCommon,
    /// The summary report data
    pub data: SummaryData,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Summary report data
pub struct SummaryData {
    /// Details about subjects available for lessons.
    pub lessons: Vec<ReviewLessonSummary>,
    /// Earliest date when the reviews are available. Is `None` when the user
    /// has no reviews scheduled.
    pub next_reviews_at: Option<DateTime<Utc>>,
    /// Details about subjects available for reviews now and in the next 24
    /// hours by the hour (total of 25 objects)
    pub reviews: Vec<ReviewLessonSummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Summary of available lessons, or of an upcoming review period
pub struct ReviewLessonSummary {
    /// When the paired `subject_ids` are available for lessons or review. All
    /// timestamps are the top of an hour.
    pub available_at: DateTime<Utc>,
    /// Collection of unique identifiers for `subjects`.
    pub subject_ids: Vec<u64>,
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};

    use crate::{Resource, ResourceCommon};

    use super::{ReviewLessonSummary, Summary, SummaryData};

    #[test]
    fn test_summary_deserialize() {
        let json = include_str!("../test_files/summary.json");

        let summary: Resource = serde_json::from_str(json).expect("Deserialize");

        let Resource::Report(summary) = summary else {
            panic!("Incorrect resource")
        };

        let expected_timestamp = Utc
            .with_ymd_and_hms(2018, 4, 11, 21, 0, 0)
            .single()
            .expect("Expected Timestamp");

        assert_eq!(
            summary.common.url.to_string(),
            "https://api.wanikani.com/v2/summary"
        );
        assert_eq!(
            summary.common.data_updated_at.expect("Timestamp"),
            expected_timestamp
        );

        assert_eq!(
            summary.data.next_reviews_at.expect("Timestamp"),
            expected_timestamp
        );

        let lessons = summary.data.lessons;
        assert_eq!(lessons.len(), 1);
        assert_eq!(lessons[0].available_at, expected_timestamp);
        assert_eq!(lessons[0].subject_ids, [25, 26]);

        let reviews = summary.data.reviews;
        assert_eq!(reviews.len(), 2);
        assert_eq!(reviews[0].available_at, expected_timestamp);
        assert_eq!(reviews[0].subject_ids, [21, 23, 24]);

        assert_eq!(
            reviews[1].available_at,
            expected_timestamp + Duration::hours(1)
        );
        assert!(reviews[1].subject_ids.is_empty());
    }

    #[test]
    fn test_summary_serialize() {
        let timestamp = Utc
            .with_ymd_and_hms(2023, 1, 1, 0, 0, 0)
            .single()
            .expect("Expected Timestamp");
        let resource = Resource::Report(Summary {
            common: ResourceCommon {
                data_updated_at: Some(timestamp),
                url: "http://some.url/".parse().expect("URL"),
            },
            data: SummaryData {
                next_reviews_at: None,
                lessons: vec![ReviewLessonSummary {
                    available_at: timestamp,
                    subject_ids: vec![1, 2, 3],
                }],
                reviews: vec![],
            },
        });

        let json = serde_json::to_string(&resource).expect("Serialization passes");

        assert!(json.contains("\"object\":\"report\""));
        assert!(json.contains("\"url\":\"http://some.url/\""));
        assert!(json.contains("\"data_updated_at\":\"2023-01-01T00:00:00Z\""));
        assert!(json.contains("\"next_reviews_at\":null"));
        assert!(json.contains("\"available_at\":\"2023-01-01T00:00:00Z\""));
        assert!(json.contains("\"subject_ids\":[1,2,3]"));
        assert!(json.contains("\"reviews\":[]"));
    }
}
