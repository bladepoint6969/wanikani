//! Study materials store user-specific notes and synonyms for a given subject.
//! The records are created as soon as the user enters any study information.

use serde::{Deserialize, Serialize};

use crate::{subject::SubjectType, Timestamp};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// Study materials store user-specific notes and synonyms for a given subject.
/// The records are created as soon as the user enters any study information.
pub struct StudyMaterial {
    /// Timestamp when the study material was created.
    pub created_at: Timestamp,
    /// Indicates if the associated subject has been hidden, preventing it from
    /// appearing in lessons or reviews.
    pub hidden: bool,
    /// Free form note related to the meaning(s) of the associated subject.
    pub meaning_note: Option<String>,
    /// Synonyms for the meaning of the subject. These are used as additional
    /// correct answers during reviews.
    pub meaning_synonyms: Vec<String>,
    /// Free form note related to the reading(s) of the associated subject.
    pub reading_note: Option<String>,
    /// Unique identifier of the associated subject.
    pub subject_id: u64,
    /// The type of the associated subject.
    pub subject_type: SubjectType,
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use crate::{cross_feature::SubjectType, Resource, ResourceCommon, ResourceType};

    use super::StudyMaterial;

    #[test]
    fn test_deserialize_study_material() {
        let json = include_str!("../test_files/study_material.json");

        let study_mat: Resource<StudyMaterial> = serde_json::from_str(json).expect("Deserialize");

        assert_eq!(study_mat.id, 65231);
        assert_eq!(study_mat.common.object, ResourceType::StudyMaterial);
        assert_eq!(
            study_mat.common.url,
            "https://api.wanikani.com/v2/study_materials/65231"
                .parse()
                .expect("URL")
        );
        assert_eq!(
            study_mat.common.data_updated_at.expect("Timestamp"),
            DateTime::parse_from_rfc3339("2017-09-30T01:42:13.453291Z").expect("Timestamp")
        );

        let data = study_mat.data;

        assert_eq!(
            data.created_at,
            DateTime::parse_from_rfc3339("2017-09-30T01:42:13.453291Z").expect("Timestamp")
        );
        assert_eq!(data.subject_id, 241);
        assert_eq!(data.subject_type, SubjectType::Radical);
        assert_eq!(data.meaning_note.expect("Meaning"), "I like turtles");
        assert_eq!(data.reading_note.expect("Reading"), "I like たrtles");
        assert_eq!(data.meaning_synonyms, ["burn", "sizzle"]);
        assert!(!data.hidden)
    }

    #[test]
    fn test_serialize_study_material() {
        let data = StudyMaterial {
            created_at: Utc::now(),
            hidden: false,
            meaning_note: Some("Meaning".into()),
            meaning_synonyms: vec![],
            reading_note: Some("reading".into()),
            subject_id: 69,
            subject_type: SubjectType::KanaVocabulary,
        };
        let common = ResourceCommon {
            data_updated_at: Some(Utc::now()),
            object: ResourceType::StudyMaterial,
            url: "https://some.url/study_material".parse().expect("URL"),
        };

        let study_mat = Resource {
            common,
            data,
            id: 696969,
        };

        let json = serde_json::to_string(&study_mat).expect("Serialize");

        let new_study: Resource<StudyMaterial> = serde_json::from_str(&json).expect("Deserialize");

        assert_eq!(study_mat, new_study);
    }
}
