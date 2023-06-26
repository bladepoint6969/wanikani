//! Level progressions contain information about a user's progress through the
//! WaniKani levels.
//!
//! A level progression is created when a user has met the prerequisites for
//! leveling up, which are:
//!
//! - Reach a 90% passing rate on assignments for a user's current level with a
//!   `subject_type` of `kanji`. Passed assignments have `data.passed` equal to
//!   true and a `data.passed_at` that's in the past.
//! - Have access to the level. Under `/user`, the `data.level` must be less
//!   than or equal to `data.subscription.max_level_granted`.

use serde::{Deserialize, Serialize};

use crate::Timestamp;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
/// Level progressions contain information about a user's progress through the
/// WaniKani levels.
///
/// A level progression is created when a user has met the prerequisites for
/// leveling up.
pub struct LevelProgression {
    /// Timestamp when the user abandons the level. This is primary used when
    /// the user initiates a [`reset`](crate::reset).
    pub abandoned_at: Option<Timestamp>,
    /// Timestamp when the user burns 100% of the assignments belonging to the
    /// associated subject's level.
    pub completed_at: Option<Timestamp>,
    /// Timestamp when the level progression is created.
    pub created_at: Timestamp,
    /// The level of the progression.
    pub level: u32,
    /// Timestamp when the user passes at least 90% of the assignments with a
    /// type of [`Kanji`](crate::subject::Kanji) belonging to the associated
    /// subject's level.
    pub passed_at: Option<Timestamp>,
    /// Timestamp when the user starts their first lesson of a subject belonging
    /// to the level.
    pub started_at: Option<Timestamp>,
    /// Timestamp when the user can access lessons and reviews for the `level`.
    pub unlocked_at: Option<Timestamp>,
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use crate::{Resource, ResourceCommon, ResourceType};

    use super::LevelProgression;

    #[test]
    fn test_level_progression_deserialize() {
        let json = include_str!("../test_files/level_progression.json");

        let level_progression: Resource<LevelProgression> =
            serde_json::from_str(json).expect("Deserialize");

        assert_eq!(level_progression.id, 49392);

        let common = level_progression.common;
        assert_eq!(common.object, ResourceType::LevelProgression);
        assert_eq!(
            common.url,
            "https://api.wanikani.com/v2/level_progressions/49392"
                .parse()
                .expect("URL")
        );
        assert_eq!(
            common.data_updated_at.expect("Timestamp"),
            DateTime::parse_from_rfc3339("2017-03-30T11:31:20.438432Z").expect("Timestamp")
        );

        let data = level_progression.data;
        assert_eq!(
            data.created_at,
            DateTime::parse_from_rfc3339("2017-03-30T08:21:51.439918Z").expect("Timestamp")
        );
        assert_eq!(data.level, 42);
        assert_eq!(
            data.unlocked_at.expect("Timestamp"),
            DateTime::parse_from_rfc3339("2017-03-30T08:21:51.439918Z").expect("Timestamp")
        );
        assert_eq!(
            data.started_at.expect("Timestamp"),
            DateTime::parse_from_rfc3339("2017-03-30T11:31:20.438432Z").expect("Timestamp")
        );
        assert!(data.passed_at.is_none());
        assert!(data.completed_at.is_none());
        assert!(data.abandoned_at.is_none());
    }

    #[test]
    fn test_level_progression_serialize() {
        let data = LevelProgression {
            abandoned_at: Some(Utc::now()),
            completed_at: None,
            created_at: Utc::now(),
            level: 69,
            passed_at: Some(Utc::now()),
            started_at: None,
            unlocked_at: Some(Utc::now()),
        };
        let common = ResourceCommon {
            data_updated_at: Some(Utc::now()),
            object: ResourceType::LevelProgression,
            url: "https://api.wanikani.com/v2/level_progressions/69420"
                .parse()
                .expect("URL"),
        };
        let level_progression = Resource {
            id: 69420,
            common,
            data,
        };

        let json = serde_json::to_string(&level_progression).expect("Serialize");

        let new_prog: Resource<_> = serde_json::from_str(&json).expect("Deserialize");

        assert_eq!(new_prog, level_progression);
    }
}
