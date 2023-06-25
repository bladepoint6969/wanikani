//! The user summary returns basic information for the user making the API
//! request, identified by their API key.

use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::{ResourceCommon, Timestamp};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// The user summary returns basic information for the user making the API
/// request, identified by their API key.
pub struct User {
    #[serde(flatten)]
    /// Common resource data.
    pub common: ResourceCommon,
    /// The user's data
    pub data: UserData,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// Basic information for the user making the API request.
pub struct UserData {
    /// The user's UUID.
    pub id: Uuid,
    /// If the user is on vacation, this will be the timestamp of when that
    /// vacation started. If the user is not on vacation, this is None.
    pub current_vacation_started_at: Option<Timestamp>,
    /// The current level of the user. This ignores subscription status.
    pub level: u32,
    /// User settings specific to the WaniKani application.
    pub preferences: Preferences,
    /// The URL to the user's public facing profile page.
    pub profile_url: Url,
    /// The signup date for the user.
    pub started_at: Timestamp,
    /// Details about the user's subscription state.
    pub subscription: Subscription,
    /// The user's username
    pub username: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
/// User settings specific to the WaniKani application.
pub struct Preferences {
    /// The voice actor to be used for lessons and reviews. The value is
    /// associated to `subject.pronunciation_audios.metadata.voice_actor_id`.
    pub default_voice_actor_id: u32,
    /// Automatically play pronunciation audio for vocabulary during extra
    /// study.
    pub extra_study_autoplay_audio: bool,
    /// Automatically play pronunciation audio for vocabulary during lessons.
    pub lessons_autoplay_audio: bool,
    /// Number of subjects introduced to the user during lessons before
    /// quizzing.
    pub lessons_batch_size: u32,
    /// The order in which lessons are presented.
    pub lessons_presentation_order: LessonPresentationOrder,
    /// Automatically play pronunciation audio for vocabulary during reviews.
    pub reviews_autoplay_audio: bool,
    /// Toggle for display SRS change indicator after a subject has been
    /// completely answered during review.
    pub reviews_display_srs_indicator: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
/// Details about the user's subscription state.
pub struct Subscription {
    /// Whether or not the user currently has a paid subscription.
    pub active: bool,
    /// The maximum level of content accessible to the user for lessons,
    /// reviews, and content review. For unsubscribed/free users, the maximum
    /// level is `3`. For subscribed users, this is `60`. **Any application that
    /// uses data from the WaniKani API must respect these access limits.**
    pub max_level_granted: u32,
    /// The date when the user's subscription period ends. If the user has
    /// subscription type `lifetime` or `free` then the value is `None`.
    pub period_ends_at: Option<Timestamp>,
    #[serde(rename = "type")]
    /// The type of subscription the user has.
    pub sub_type: SubscriptionType,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// The order in which lessons are presented.
pub enum LessonPresentationOrder {
    #[default]
    /// Lessons are presented in order of level, then by subject `id`.
    AscendingLevelThenSubject,
    /// Lessons are presented in a random order.
    Shuffled,
    /// Lessons are presented in order of level, then randomly.
    AscendingLevelThenShuffled,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// Defines the kind of subscription a user has.
pub enum SubscriptionType {
    /// Free subscriptions aren't really subscriptions, but can represent
    /// people who've never subscribed or have an inactive subscription.
    Free,
    /// Recurring subscriptions renew on a periodic basis.
    Recurring,
    /// :ifetime means the user can access WaniKani forever. `period_ends_at` is
    /// `null`, mainly because ∞ is hard for computers to get. It's possible
    /// that a lifetime user will ask for a refund or have payment difficulties,
    /// so scheduled checks on the subscription status are still needed.
    Lifetime,
    /// Unknown means the user subscription state isn't exactly known. This is a
    /// weird state on WaniKani, should be treated as `free`, and reported to
    /// the WaniKani developers.
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
/// User information update request.
pub struct UpdateUser {
    /// Preference updates.
    pub preferences: UpdatePreferences,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
/// User preference updates.
pub struct UpdatePreferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The voice actor to be used for lessons and reviews.
    pub default_voice_actor_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Automatically play pronunciation audio for vocabulary during extra
    /// study.
    pub extra_study_autoplay_audio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Automatically play pronunciation audio for vocabulary during lessons.
    pub lessons_autoplay_audio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Number of subjects introduced to the user during lessons before
    /// quizzing.
    pub lessons_batch_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The order in which lessons are presented.
    pub lessons_presentation_order: Option<LessonPresentationOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Automatically play pronunciation audio for vocabulary during reviews.
    pub reviews_autoplay_audio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Toggle for display SRS change indicator after a subject has been
    /// completely answered during review.
    pub reviews_display_srs_indicator: Option<bool>,
}

impl From<&Preferences> for UpdatePreferences {
    fn from(value: &Preferences) -> Self {
        Self {
            default_voice_actor_id: value.default_voice_actor_id.into(),
            extra_study_autoplay_audio: value.extra_study_autoplay_audio.into(),
            lessons_autoplay_audio: value.lessons_autoplay_audio.into(),
            lessons_batch_size: value.lessons_batch_size.into(),
            lessons_presentation_order: value.lessons_presentation_order.into(),
            reviews_autoplay_audio: value.reviews_autoplay_audio.into(),
            reviews_display_srs_indicator: value.reviews_display_srs_indicator.into(),
        }
    }
}

impl From<Preferences> for UpdatePreferences {
    fn from(value: Preferences) -> Self {
        UpdatePreferences::from(&value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        user::{LessonPresentationOrder, SubscriptionType, User},
        ResourceCommon,
    };
    use chrono::{DateTime, Utc};
    use url::Url;

    use super::{Preferences, Subscription, UserData};

    #[test]
    fn test_user_deserialize() {
        let json = include_str!("../test_files/user.json");

        let user: User = serde_json::from_str(json).expect("Deserialize");

        assert_eq!(user.common.object, "user");
        assert_eq!(
            user.common.url.to_string(),
            "https://api.wanikani.com/v2/user"
        );

        let expected_timestamp =
            DateTime::parse_from_rfc3339("2018-04-06T14:26:53.022245Z").expect("Timestamp");
        assert_eq!(
            user.common.data_updated_at.expect("Timestamp"),
            expected_timestamp
        );

        let data = user.data;
        assert_eq!(data.id, uuid::uuid!("5a6a5234-a392-4a87-8f3f-33342afe8a42"));
        assert_eq!(data.username, "example_user");
        assert_eq!(data.level, 5);
        assert_eq!(
            data.profile_url.to_string(),
            "https://www.wanikani.com/users/example_user"
        );
        let expected_timestamp =
            DateTime::parse_from_rfc3339("2012-05-11T00:52:18.958466Z").expect("Timestamp");
        assert_eq!(data.started_at, expected_timestamp);
        assert!(data.current_vacation_started_at.is_none());

        let subscription = data.subscription;
        assert!(subscription.active);
        assert_eq!(subscription.sub_type, SubscriptionType::Recurring);
        assert_eq!(subscription.max_level_granted, 60);
        let expected_timestamp =
            DateTime::parse_from_rfc3339("2018-12-11T13:32:19.485748Z").expect("Timestamp");
        assert_eq!(
            subscription.period_ends_at.expect("Contained Timestamp"),
            expected_timestamp
        );

        let prefs = data.preferences;

        assert_eq!(prefs.default_voice_actor_id, 1);
        assert!(!prefs.extra_study_autoplay_audio);
        assert!(!prefs.lessons_autoplay_audio);
        assert_eq!(prefs.lessons_batch_size, 10);
        assert_eq!(
            prefs.lessons_presentation_order,
            LessonPresentationOrder::AscendingLevelThenSubject
        );
        assert!(!prefs.reviews_autoplay_audio);
        assert!(prefs.reviews_display_srs_indicator);
    }

    #[test]
    fn test_user_serialize() {
        let preferences = Preferences {
            default_voice_actor_id: 1,
            extra_study_autoplay_audio: true,
            lessons_autoplay_audio: true,
            lessons_batch_size: 5,
            lessons_presentation_order: LessonPresentationOrder::Shuffled,
            reviews_autoplay_audio: true,
            reviews_display_srs_indicator: true,
        };
        let subscription = Subscription {
            active: true,
            sub_type: SubscriptionType::Lifetime,
            max_level_granted: 60,
            period_ends_at: None,
        };
        let data = UserData {
            id: uuid::uuid!("00000000-0000-0000-0000-000000000000"),
            username: "my_test_user".into(),
            level: 8,
            profile_url: Url::parse("https://www.wanikani.com/users/my_test_user").expect("URL"),
            started_at: Utc::now(),
            current_vacation_started_at: None,
            subscription,
            preferences,
        };
        let common = ResourceCommon {
            object: "user".into(),
            url: Url::parse("https://api.wanikani.com/v2/user").expect("URL"),
            data_updated_at: Some(Utc::now()),
        };

        let user = User { common, data };

        let json = serde_json::to_string(&user).expect("Serialization passes");

        let new_user: User = serde_json::from_str(&json).expect("Deserialize correctly");

        assert_eq!(new_user, user);
    }
}
