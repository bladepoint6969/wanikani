//! Users can reset their progress back to any level at or below their current
//! level. When they reset to a particular level, all of the `assignments` and
//! `review_statistics` at that level or higher are set back to their default
//! state.
//!
//! Resets contain information about when those resets happen, the starting
//! level, and the target level.

use serde::{Deserialize, Serialize};

use crate::Timestamp;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
/// Users can reset their progress back to any level at or below their current
/// level. When they reset to a particular level, all of the `assignments` and
/// `review_statistics` at that level or higher are set back to their default
/// state.
///
/// Resets contain information about when those resets happen, the starting level, and the target level.
pub struct Reset {
    /// Timestamp when the user confirmed the reset.
    pub confirmed_at: Option<Timestamp>,
    /// Timestamp when the reset was created.
    pub created_at: Timestamp,
    /// The user's level before the reset
    pub original_level: u32,
    /// The user's level after the reset. It must be less than or equal to
    /// `original_level`.
    pub target_level: u32,
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use crate::{Resource, ResourceCommon, ResourceType};

    use super::Reset;

    #[test]
    fn test_deserialize_reset() {
        let json = include_str!("../test_files/reset.json");

        let reset: Resource<Reset> = serde_json::from_str(json).expect("Deserialize");

        assert_eq!(reset.id, 234);

        let common = reset.common;

        assert_eq!(common.object, ResourceType::Reset);
        assert_eq!(
            common.url,
            "https://api.wanikani.com/v2/resets/80463006"
                .parse()
                .expect("URL")
        );
        assert_eq!(
            common.data_updated_at.expect("Timestamp"),
            DateTime::parse_from_rfc3339("2017-12-20T00:24:47.048380Z").expect("Timestamp")
        );

        let data = reset.data;
        assert_eq!(
            data.created_at,
            DateTime::parse_from_rfc3339("2017-12-20T00:03:56.642838Z").expect("Timestamp")
        );
        assert_eq!(data.original_level, 42);
        assert_eq!(data.target_level, 8);
        assert_eq!(
            data.confirmed_at.expect("Timestamp"),
            DateTime::parse_from_rfc3339("2017-12-19T23:31:18.077268Z").expect("Timestamp")
        );
    }

    #[test]
    fn test_serialize_reset() {
        let data = Reset {
            created_at: Utc::now(),
            confirmed_at: Some(Utc::now()),
            original_level: 60,
            target_level: 4,
        };

        let common = ResourceCommon {
            data_updated_at: Some(Utc::now()),
            object: ResourceType::Reset,
            url: "http://some.url/reset".parse().expect("URL"),
        };

        let reset = Resource {
            common,
            data,
            id: 69420,
        };

        let json = serde_json::to_string(&reset).expect("Serialize");

        let new_reset = serde_json::from_str(&json).expect("Deserialize");

        assert_eq!(reset, new_reset);
    }
}
