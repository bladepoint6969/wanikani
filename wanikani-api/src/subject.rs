//! Subjects are the radicals, kanji, vocabulary, and kana_vocabulary that are
//! learned through lessons and reviews. They contain basic dictionary
//! information, such as meanings and/or readings, and information about their
//! relationship to other items with WaniKani, like their level.
//!
//! ## Markup highlighting
//!
//! One or many of these attributes can be present in `radical`, `kanji`, and
//! `vocabulary`:
//!
//! - `meaning_mnemonic`
//! - `reading_mnemonic`
//! - `meaning_hint`
//! - `reading_hint`
//!
//! The strings can include a WaniKani specific markup syntax. The following is
//! a list of markup used:
//!
//! - `<radical></radical>`
//! - `<kanji></kanji>`
//! - `<vocabulary></vocabulary>`
//! - `<meaning></meaning>`
//! - `<reading></reading>`

use mime::Mime;
use serde::{Deserialize, Serialize};
use url::Url;

pub use crate::subject_type::*;
use crate::Timestamp;

/// The `SubjectType` trait exists to help avoid footguns when requesting
/// specific subjects with the API client.
pub trait FetchSubject: private::Sealed + for<'de> Deserialize<'de> {}

impl FetchSubject for Subject {}
impl FetchSubject for Radical {}

mod private {
    use super::{Radical, Subject};

    pub trait Sealed {}

    impl Sealed for Subject {}
    impl Sealed for Radical {}
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
/// Aggregation of assorted subject types to allow containing multiple subject
/// types in one collection.
pub enum Subject {
    /// A Radical
    Radical(Radical),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// Attributes that are common to all subject types
pub struct SubjectCommon {
    /// Collection of auxiliary meanings.
    pub auxiliary_meanings: Vec<AuxilliaryMeaning>,
    /// Timestamp when the subject was created.
    pub created_at: Timestamp,
    /// A URL pointing to the page on wanikani.com that provides detailed
    /// information about this subject.
    pub document_url: Url,
    /// Timestamp when the subject was hidden, indicating associated assignments
    /// will no longer appear in lessons or reviews and that the subject page is
    /// no longer visible on wanikani.com.
    pub hidden_at: Option<Timestamp>,
    /// The position that the subject appears in lessons. Note that the value is
    /// scoped to the level of the subject, so there are duplicate values across
    /// levels.
    pub lesson_position: u32,
    /// The level of the subject.
    pub level: u32,
    /// The subject's meaning mnemonic.
    pub meaning_mnemonic: String,
    /// The subject meanings.
    pub meanings: Vec<Meaning>,
    /// The string that is used when generating the document URL for the
    /// subject. Radicals use their meaning, downcased. Kanji and vocabulary use
    /// their characters.
    pub slug: String,
    /// Unique identifier of the associated spaced repetition system
    pub spaced_repetition_system_id: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// A meaning for a subject.
pub struct Meaning {
    /// A singular subject meaning.
    pub meaning: String,
    /// Indicates priority in the WaniKani system.
    pub primary: bool,
    /// Indicates if the meaning is used to evaluate user input for correctness.
    pub accepted_answer: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// A secondary meaning for a subject. This may include incorrect meanings that
/// are used to check for incorrectness.
pub struct AuxilliaryMeaning {
    /// A singular subject meaning.
    pub meaning: String,
    #[serde(rename = "type")]
    /// The type of the meaning.
    pub meaning_type: MeaningType,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// An auxiliary meaning's type.
pub enum MeaningType {
    /// Meaning is used to match for correctness.
    Whitelist,
    /// Meaning is used to match for incorrectness.
    Blacklist,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// A radical is a kanji component, around which mnemonics for remembering kanji
/// can be created.
pub struct Radical {
    #[serde(flatten)]
    /// Attributes common to all subjects.
    pub common: SubjectCommon,
    /// An array of numeric identifiers for the kanji that have the radical as a
    /// component.
    pub amalgamation_subject_ids: Vec<u64>,
    /// Unlike kanji and vocabulary, radicals can have a `nul` value for
    /// `characters`. Not all radicals have a UTF entry, so the radical must be
    /// visually represented with an image instead.
    pub characters: Option<String>,
    /// A collection of images of the radical.
    pub character_images: Vec<CharacterImage>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// An image that can be used to represent a radical when no character is
/// available.
pub struct CharacterImage {
    /// The location of the image.
    pub url: Url,
    #[serde(with = "mime_serde_shim")]
    /// The content type of the image. Currently the API delivers `image/png`
    /// and `image/svg+xml`.
    pub content_type: Mime,
    /// Details about the image. Each content_type returns a uniquely structured
    /// object.
    pub metadata: ImageMetadata,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
/// Details about an image.
pub enum ImageMetadata {
    /// Details of an SVG image.
    SVG {
        /// The SVG asset contains built-in CSS styling.
        inline_styles: bool,
    },
    /// Details of a PNG image.
    PNG {
        /// Color of the asset in hexadecimal
        color: String,
        /// Dimension of the asset in pixels.
        dimensions: String,
        /// A name descriptor
        style_name: String,
    },
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use crate::{
        subject::{AuxilliaryMeaning, CharacterImage, ImageMetadata, Meaning, MeaningType},
        Resource, ResourceCommon, ResourceType,
    };

    use super::{Radical, Subject, SubjectCommon};

    #[test]
    fn test_radical_deserialize() {
        let json = include_str!("../test_files/radical.json");

        let subject: Resource<Subject> = serde_json::from_str(json).expect("Deserialize subject");
        let Subject::Radical(subject_inner) = subject.data else {
            panic!("Incorrect subject type");
        };

        let radical: Resource<Radical> = serde_json::from_str(json).expect("Deserialize radical");

        // Prove that Subject and Radical Deserializations are identical
        assert_eq!(radical.id, subject.id);
        assert_eq!(radical.common, subject.common);
        assert_eq!(radical.data, subject_inner);

        assert_eq!(radical.id, 1);

        let common = radical.common;
        assert_eq!(common.object, ResourceType::Radical);
        assert_eq!(
            common.url,
            "https://api.wanikani.com/v2/subjects/1"
                .parse()
                .expect("URL")
        );
        assert_eq!(
            common.data_updated_at.expect("Timestamp"),
            DateTime::parse_from_rfc3339("2018-03-29T23:13:14.064836Z").expect("Timestamp")
        );

        let data = radical.data;
        assert_eq!(data.amalgamation_subject_ids, [5, 4, 98]);
        assert_eq!(
            data.common.auxiliary_meanings,
            [AuxilliaryMeaning {
                meaning: "ground".into(),
                meaning_type: MeaningType::Blacklist,
            }]
        );
        assert_eq!(data.characters.expect("Chars"), "一");
        assert_eq!(data.character_images, [CharacterImage {
            url: "https://cdn.wanikani.com/images/legacy/576-subject-1-without-css-original.svg?1520987227".parse().expect("URL"),
            metadata: ImageMetadata::SVG { inline_styles: false },
            content_type: mime::IMAGE_SVG
        }]);
        assert_eq!(
            data.common.created_at,
            DateTime::parse_from_rfc3339("2012-02-27T18:08:16.000000Z").expect("Timestamp")
        );
        assert_eq!(
            data.common.document_url,
            "https://www.wanikani.com/radicals/ground"
                .parse()
                .expect("URL")
        );
        assert!(data.common.hidden_at.is_none());
        assert_eq!(data.common.lesson_position, 1);
        assert_eq!(data.common.level, 1);
        assert_eq!(
            data.common.meanings,
            [Meaning {
                meaning: "Ground".into(),
                primary: true,
                accepted_answer: true
            }]
        );
        assert_eq!(data.common.meaning_mnemonic, "This radical consists of a single, horizontal stroke. What's the biggest, single, horizontal stroke? That's the ground. Look at the <radical>ground</radical>, look at this radical, now look at the ground again. Kind of the same, right?");
        assert_eq!(data.common.slug, "ground");
        assert_eq!(data.common.spaced_repetition_system_id, 2);
    }

    #[test]
    fn test_radical_serialize() {
        let common = SubjectCommon {
            auxiliary_meanings: vec![],
            created_at: Utc::now(),
            document_url: "https://www.wanikani.com/radicals/test_rad"
                .parse()
                .expect("URL"),
            hidden_at: None,
            lesson_position: 69,
            level: 420,
            meaning_mnemonic: "This is a test radical".into(),
            meanings: vec![Meaning {
                accepted_answer: true,
                primary: false,
                meaning: "This is the meaning".into(),
            }],
            slug: "test".into(),
            spaced_repetition_system_id: 5,
        };
        let data = Radical {
            amalgamation_subject_ids: vec![5, 10, 15],
            character_images: vec![],
            characters: Some("💩".into()),
            common,
        };
        let common = ResourceCommon {
            data_updated_at: Some(Utc::now()),
            object: ResourceType::Radical,
            url: "https://api.wanikani.com/v2/subjects/69"
                .parse()
                .expect("URL"),
        };
        let radical = Resource {
            common,
            data,
            id: 69,
        };

        let json = serde_json::to_string(&radical).expect("Serialize");

        let subject: Resource<Subject> = serde_json::from_str(&json).expect("Deserialize");

        assert_eq!(subject.common, radical.common);
        assert_eq!(subject.id, radical.id);
        let Subject::Radical(subject_inner) = subject.data else {
            panic!("Incorrect subject type");
        };
        assert_eq!(subject_inner, radical.data);
    }
}
