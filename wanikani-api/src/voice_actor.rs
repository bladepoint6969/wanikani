//! Available voice actors used for vocabulary reading pronunciation audio.

use serde::{Deserialize, Serialize};

use crate::Timestamp;

pub use crate::cross_feature::Gender;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// Specific voice actor data
pub struct VoiceActor {
    /// When the voice actor was first added.
    pub created_at: Timestamp,
    /// The voice actor's name.
    pub name: String,
    /// The voice actor's gender.
    pub gender: Gender,
    /// Details about the voice actor.
    pub description: String,
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use url::Url;

    use crate::{voice_actor::Gender, Collection, Pages, Resource, ResourceCommon, ResourceType};

    use super::VoiceActor;

    #[test]
    fn test_voice_actor_deserialize() {
        let json = include_str!("../test_files/voice_actors.json");

        let collection: Collection<VoiceActor> = serde_json::from_str(json).expect("Deserialize");

        assert_eq!(collection.common.object, ResourceType::Collection);
        assert_eq!(
            collection.common.url.to_string(),
            "https://api.wanikani.com/v2/voice_actors"
        );
        assert_eq!(
            collection.common.data_updated_at.expect("Timestamp"),
            DateTime::parse_from_rfc3339("2023-06-15T19:15:46.020703Z").expect("Timestamp")
        );

        assert_eq!(
            collection.pages,
            Pages {
                per_page: 500,
                next_url: None,
                previous_url: None,
            }
        );
        assert_eq!(collection.total_count, 2);

        let data = collection.data;

        assert_eq!(data.len(), 2);
        let kyoko = data.get(0).expect("Exists");
        let kenichi = data.get(1).expect("Exists");

        let kyoko_expected = Resource::<VoiceActor> {
            id: 1,
            common: ResourceCommon {
                object: ResourceType::VoiceActor,
                url: Url::parse("https://api.wanikani.com/v2/voice_actors/1").expect("URL"),
                data_updated_at: Some(
                    DateTime::parse_from_rfc3339("2023-06-15T19:15:46.020703Z")
                        .expect("Timestamp")
                        .into(),
                ),
            },
            data: VoiceActor {
                created_at: DateTime::parse_from_rfc3339("2018-09-11T18:30:27.096474Z")
                    .expect("Timestamp")
                    .into(),
                name: "Kyoko".into(),
                gender: Gender::Female,
                description: "Tokyo accent".into(),
            },
        };
        let kenichi_expected = Resource::<VoiceActor> {
            id: 2,
            common: ResourceCommon {
                object: ResourceType::VoiceActor,
                url: Url::parse("https://api.wanikani.com/v2/voice_actors/2").expect("URL"),
                data_updated_at: Some(
                    DateTime::parse_from_rfc3339("2023-06-15T19:15:45.983401Z")
                        .expect("Timestamp")
                        .into(),
                ),
            },
            data: VoiceActor {
                created_at: DateTime::parse_from_rfc3339("2018-09-11T18:30:28.089969Z")
                    .expect("Timestamp")
                    .into(),
                name: "Kenichi".into(),
                gender: Gender::Male,
                description: "Tokyo accent".into(),
            },
        };

        assert_eq!(kyoko, &kyoko_expected);
        assert_eq!(kenichi, &kenichi_expected)
    }

    #[test]
    fn test_voice_actor_serialize() {
        let data = VoiceActor {
            created_at: Utc::now(),
            description: "Some test actor".into(),
            gender: Gender::Male,
            name: "Test Actor".into(),
        };
        let common = ResourceCommon {
            data_updated_at: Some(Utc::now()),
            url: Url::parse("https://api.wanikani.com/v2/voice_actors/69").expect("URL"),
            object: ResourceType::VoiceActor,
        };
        let vo = Resource::<VoiceActor> {
            id: 69,
            common,
            data,
        };

        let json = serde_json::to_string(&vo).expect("Serialize");

        let new_vo: Resource<VoiceActor> = serde_json::from_str(&json).expect("Deserialize");
        assert_eq!(new_vo, vo);
    }
}
