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

pub use crate::cross_feature::*;
use crate::{voice_actor::Gender, Timestamp};

/// The `FetchSubject` trait exists to help avoid footguns when requesting
/// specific subjects with the API client.
pub trait FetchSubject: private::Sealed + for<'de> Deserialize<'de> {}

impl FetchSubject for Subject {}
impl FetchSubject for Radical {}
impl FetchSubject for Kanji {}
impl FetchSubject for Vocabulary {}
impl FetchSubject for KanaVocabulary {}

mod private {
    use super::{KanaVocabulary, Kanji, Radical, Subject, Vocabulary};

    pub trait Sealed {}

    impl Sealed for Subject {}
    impl Sealed for Radical {}
    impl Sealed for Kanji {}
    impl Sealed for Vocabulary {}
    impl Sealed for KanaVocabulary {}
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
/// Aggregation of assorted subject types to allow containing multiple subject
/// types in one collection.
pub enum Subject {
    /// A Radical
    Radical(Radical),
    /// A Kanji
    Kanji(Kanji),
    /// A Vocabulary word
    Vocabulary(Vocabulary),
    /// A kana-only vocabulary word
    KanaVocabulary(KanaVocabulary),
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// A kanji subject.
pub struct Kanji {
    /// Attributes common to all subjects.
    #[serde(flatten)]
    pub common: SubjectCommon,
    /// An array of numeric identifiers for the vocabulary that have the kanji
    /// as a component.
    pub amalgamation_subject_ids: Vec<u64>,
    /// The UTF-8 characters for the subject, including kanji and hiragana.
    pub characters: String,
    /// An array of numeric identifiers for the radicals that make up this
    /// kanji. Note that these are the subjects that must have passed
    /// assignments in order to unlock this subject's assignment.
    pub component_subject_ids: Vec<u64>,
    /// Meaning hint for the kanji.
    pub meaning_hint: Option<String>,
    /// Reading hint for the kanji.
    pub reading_hint: Option<String>,
    /// The kanji's reading mnemonic.
    pub reading_mnemonic: String,
    /// Selected readings for the kanji.
    pub readings: Vec<KanjiReading>,
    /// An array of numeric identifiers for kanji which are visually similar to the kanji in question.
    pub visually_similar_subject_ids: Vec<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// A Kanji reading.
pub struct KanjiReading {
    /// A singular subject reading.
    pub reading: String,
    /// Indicates priority in the WaniKani system.
    pub primary: bool,
    /// Indicates if the reading is used to evaluate user input for correctness.
    pub accepted_answer: bool,
    #[serde(rename = "type")]
    /// The kanji reading's classfication.
    pub reading_type: KanjiReadingType,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// The classfication of a kanji reading
pub enum KanjiReadingType {
    /// Kun'yomi readings are Japanese readings of a kanji.
    Kunyomi,
    /// Nanori readings are nonstandard readings almost exclusively used in
    /// names.
    Nanori,
    /// On'yomi readings are derived from the Chinese readings of a kanji.
    Onyomi,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// A kanji-based vocabulary word.
pub struct Vocabulary {
    #[serde(flatten)]
    /// Attributes common to all subjects.
    pub common: SubjectCommon,
    /// The UTF-8 characters for the subject, including kanji and hiragana.
    pub characters: String,
    /// An array of numeric identifiers for the kanji that make up this
    /// vocabulary. Note that these are the subjects that must be have passed
    /// assignments in order to unlock this subject's assignment.
    pub component_subject_ids: Vec<u64>,
    /// A collection of context sentences.
    pub context_sentences: Vec<ContextSentence>,
    /// Parts of speech.
    pub parts_of_speech: Vec<String>,
    /// A collection of pronunciation audio.
    pub pronunciation_audios: Vec<PronunciationAudio>,
    /// Selected readings for the vocabulary.
    pub readings: Vec<VocabularyReading>,
    /// The subject's reading mnemonic.
    pub reading_mnemonic: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// A context sentence that shows how the vocabulary is used.
pub struct ContextSentence {
    /// English translation of the sentence.
    pub en: String,
    /// Japanese context sentence.
    pub ja: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// Audio files that demonstrate how the vocabulary is pronounced.
pub struct PronunciationAudio {
    /// The location of the audio.
    pub url: Url,
    #[serde(with = "mime_serde_shim")]
    /// The content type of the audio. Currently the API delivers `audio/mpeg`
    /// and `audio/ogg`.
    pub content_type: Mime,
    /// Details about the pronunciation audio.
    pub metadata: AudioMetadata,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// A selected reading for the vocabulary.
pub struct VocabularyReading {
    /// Indicates if the reading is used to evaluate user input for correctness.
    pub accepted_answer: bool,
    /// Indicates priority in the WaniKani system.
    pub primary: bool,
    /// A singular subject reading.
    pub reading: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// Details on a pronunciation audio clip.
pub struct AudioMetadata {
    /// The gender of the voice actor.
    pub gender: Gender,
    /// A unique ID shared between same source pronunciation audio.
    pub source_id: u64,
    /// Vocabulary being pronounced in kana.
    pub pronunciation: String,
    /// A unique ID belonging to the voice actor.
    pub voice_actor_id: u64,
    /// Humanized name of the voice actor.
    pub voice_actor_name: String,
    /// Description of the voice.
    pub voice_description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// A kana-only vocabulary word.
pub struct KanaVocabulary {
    #[serde(flatten)]
    /// Attributes common to all subjects.
    pub common: SubjectCommon,
    /// The UTF-8 characters for the subject, including kanji and hiragana.
    pub characters: String,
    /// A collection of context sentences.
    pub context_sentences: Vec<ContextSentence>,
    /// Parts of speech.
    pub parts_of_speech: Vec<String>,
    /// A collection of pronunciation audio.
    pub pronunciation_audios: Vec<PronunciationAudio>,
}

#[cfg(test)]
mod tests {
    use std::vec;

    use chrono::{DateTime, Utc};

    use crate::{
        cross_feature::Gender,
        subject::{
            AudioMetadata, AuxilliaryMeaning, CharacterImage, ContextSentence, ImageMetadata,
            KanaVocabulary, Kanji, KanjiReading, KanjiReadingType, Meaning, MeaningType,
            PronunciationAudio, Vocabulary, VocabularyReading,
        },
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

    #[test]
    fn test_kanji_deserialize() {
        let json = include_str!("../test_files/kanji.json");

        let subject: Resource<Subject> = serde_json::from_str(json).expect("Deserialize subject");
        let Subject::Kanji(subject_inner) = subject.data else {
            panic!("Incorrect subject type");
        };

        let kanji: Resource<Kanji> = serde_json::from_str(json).expect("Deserialize kanji");

        // Prove that Subject and Kanji Deserializations are identical
        assert_eq!(kanji.id, subject.id);
        assert_eq!(kanji.common, subject.common);
        assert_eq!(kanji.data, subject_inner);

        assert_eq!(kanji.id, 440);

        let common = kanji.common;
        assert_eq!(common.object, ResourceType::Kanji);
        assert_eq!(
            common.url,
            "https://api.wanikani.com/v2/subjects/440"
                .parse()
                .expect("URL")
        );
        assert_eq!(
            common.data_updated_at.expect("Timestamp"),
            DateTime::parse_from_rfc3339("2018-03-29T23:14:30.805034Z").expect("Timestamp")
        );

        let data = kanji.data;
        assert_eq!(data.amalgamation_subject_ids, [56, 88, 91]);
        assert_eq!(
            data.common.auxiliary_meanings,
            [
                AuxilliaryMeaning {
                    meaning: "one".into(),
                    meaning_type: MeaningType::Blacklist,
                },
                AuxilliaryMeaning {
                    meaning: "flat".into(),
                    meaning_type: MeaningType::Whitelist,
                }
            ]
        );
        assert_eq!(data.characters, "一");
        assert_eq!(data.component_subject_ids, [1]);
        assert_eq!(
            data.common.created_at,
            DateTime::parse_from_rfc3339("2012-02-27T19:55:19.000000Z").expect("Timestamp")
        );
        assert_eq!(
            data.common.document_url,
            "https://www.wanikani.com/kanji/%E4%B8%80"
                .parse()
                .expect("URL")
        );
        assert!(data.common.hidden_at.is_none());
        assert_eq!(data.common.lesson_position, 2);
        assert_eq!(data.common.level, 1);
        assert_eq!(
            data.common.meanings,
            [Meaning {
                meaning: "One".into(),
                primary: true,
                accepted_answer: true,
            }]
        );
        assert_eq!(data.meaning_hint.expect("Hint"), "To remember the meaning of <kanji>One</kanji>, imagine yourself there at the scene of the crime. You grab <kanji>One</kanji> in your arms, trying to prop it up, trying to hear its last words. Instead, it just splatters some blood on your face. \"Who did this to you?\" you ask. The number One points weakly, and you see number Two running off into an alleyway. He's always been jealous of number One and knows he can be number one now that he's taken the real number one out.");
        assert_eq!(data.common.meaning_mnemonic, "Lying on the <radical>ground</radical> is something that looks just like the ground, the number <kanji>One</kanji>. Why is this One lying down? It's been shot by the number two. It's lying there, bleeding out and dying. The number One doesn't have long to live.");
        assert_eq!(
            data.readings,
            [
                KanjiReading {
                    reading_type: KanjiReadingType::Onyomi,
                    primary: true,
                    accepted_answer: true,
                    reading: "いち".into(),
                },
                KanjiReading {
                    reading_type: KanjiReadingType::Kunyomi,
                    primary: false,
                    accepted_answer: false,
                    reading: "ひと".into(),
                },
                KanjiReading {
                    reading_type: KanjiReadingType::Nanori,
                    primary: false,
                    accepted_answer: false,
                    reading: "かず".into(),
                }
            ]
        );
        assert_eq!(data.reading_mnemonic, "As you're sitting there next to <kanji>One</kanji>, holding him up, you start feeling a weird sensation all over your skin. From the wound comes a fine powder (obviously coming from the special bullet used to kill One) that causes the person it touches to get extremely <reading>itchy</reading> (いち)");
        assert_eq!(data.reading_hint.expect("Hint"), "Make sure you feel the ridiculously <reading>itchy</reading> sensation covering your body. It climbs from your hands, where you're holding the number <kanji>One</kanji> up, and then goes through your arms, crawls up your neck, goes down your body, and then covers everything. It becomes uncontrollable, and you're scratching everywhere, writhing on the ground. It's so itchy that it's the most painful thing you've ever experienced (you should imagine this vividly, so you remember the reading of this kanji).");
        assert_eq!(data.common.slug, "一");
        assert!(data.visually_similar_subject_ids.is_empty());
        assert_eq!(data.common.spaced_repetition_system_id, 1);
    }

    #[test]
    fn test_kanji_serialize() {
        let common = SubjectCommon {
            auxiliary_meanings: vec![],
            created_at: Utc::now(),
            document_url: "https://www.wanikani.com/kanji/test_kan"
                .parse()
                .expect("URL"),
            hidden_at: None,
            lesson_position: 4,
            level: 2,
            meaning_mnemonic: "This is a test kanji".into(),
            meanings: vec![],
            slug: "kanji".into(),
            spaced_repetition_system_id: 5,
        };
        let data = Kanji {
            amalgamation_subject_ids: vec![],
            characters: "kanji".into(),
            common,
            component_subject_ids: vec![1, 2, 3],
            meaning_hint: None,
            reading_hint: None,
            reading_mnemonic: "this is the reading mnemonic".into(),
            readings: vec![],
            visually_similar_subject_ids: vec![1, 2, 3],
        };
        let common = ResourceCommon {
            data_updated_at: Some(Utc::now()),
            object: ResourceType::Kanji,
            url: "https://api.wanikani.com/v2/subjects/440"
                .parse()
                .expect("URL"),
        };
        let kanji = Resource {
            common,
            data,
            id: 5,
        };

        let json = serde_json::to_string(&kanji).expect("Serialize");
        let subject: Resource<Subject> = serde_json::from_str(&json).expect("Deserialize");

        assert_eq!(subject.common, kanji.common);
        assert_eq!(subject.id, kanji.id);
        let Subject::Kanji(subject_inner) = subject.data else {
            panic!("Incorrect subject type");
        };
        assert_eq!(subject_inner, kanji.data);
    }

    #[test]
    fn test_vocab_deserialize() {
        let json = include_str!("../test_files/vocabulary.json");

        let subject: Resource<Subject> = serde_json::from_str(json).expect("Deserialize subject");
        let Subject::Vocabulary(subject_inner) = subject.data else {
            panic!("Incorrect subject type");
        };

        let vocab: Resource<Vocabulary> = serde_json::from_str(json).expect("Deserialize vocab");

        // Prove that Subject and Vocab Deserializations are identical
        assert_eq!(vocab.id, subject.id);
        assert_eq!(vocab.common, subject.common);
        assert_eq!(vocab.data, subject_inner);

        assert_eq!(vocab.id, 2467);

        let common = vocab.common;
        assert_eq!(common.object, ResourceType::Vocabulary);
        assert_eq!(
            common.url,
            "https://api.wanikani.com/v2/subjects/2467"
                .parse()
                .expect("URL")
        );
        assert_eq!(
            common.data_updated_at.expect("Timestamp"),
            DateTime::parse_from_rfc3339("2018-12-12T23:09:52.234049Z").expect("Timestamp")
        );

        let data = vocab.data;
        assert_eq!(
            data.common.auxiliary_meanings,
            [AuxilliaryMeaning {
                meaning: "1".into(),
                meaning_type: MeaningType::Whitelist,
            }]
        );
        assert_eq!(data.characters, "一");
        assert_eq!(data.component_subject_ids, [440]);
        assert_eq!(
            data.context_sentences,
            [
                ContextSentence {
                    en: "Let’s meet up once.".into(),
                    ja: "一ど、あいましょう。".into()
                },
                ContextSentence {
                    en: "First place was an American.".into(),
                    ja: "一いはアメリカ人でした。".into()
                },
                ContextSentence {
                    en: "I’m the weakest man in the world.".into(),
                    ja: "ぼくはせかいで一ばんよわい。".into()
                }
            ]
        );
        assert_eq!(
            data.common.created_at,
            DateTime::parse_from_rfc3339("2012-02-28T08:04:47.000000Z").expect("Timestamp")
        );
        assert_eq!(
            data.common.document_url,
            "https://www.wanikani.com/vocabulary/%E4%B8%80"
                .parse()
                .expect("URL")
        );
        assert!(data.common.hidden_at.is_none());
        assert_eq!(data.common.lesson_position, 44);
        assert_eq!(data.common.level, 1);
        assert_eq!(
            data.common.meanings,
            [Meaning {
                meaning: "One".into(),
                primary: true,
                accepted_answer: true
            }]
        );
        assert_eq!(data.common.meaning_mnemonic, "As is the case with most vocab words that consist of a single kanji, this vocab word has the same meaning as the kanji it parallels, which is <vocabulary>one</vocabulary>.");
        assert_eq!(data.parts_of_speech, ["numeral"]);
        assert_eq!(
            data.pronunciation_audios,
            [
                PronunciationAudio {
                    content_type: "audio/mpeg".parse().expect("Mime"),
                    url: "https://cdn.wanikani.com/audios/3020-subject-2467.mp3?1547862356"
                        .parse()
                        .expect("URL"),
                    metadata: AudioMetadata {
                        gender: Gender::Male,
                        source_id: 2711,
                        voice_actor_id: 2,
                        pronunciation: "いち".into(),
                        voice_actor_name: "Kenichi".into(),
                        voice_description: "Tokyo accent".into(),
                    },
                },
                PronunciationAudio {
                    content_type: "audio/ogg".parse().expect("Mime"),
                    url: "https://cdn.wanikani.com/audios/3018-subject-2467.ogg?1547862356"
                        .parse()
                        .expect("URL"),
                    metadata: AudioMetadata {
                        gender: Gender::Male,
                        source_id: 2711,
                        voice_actor_id: 2,
                        pronunciation: "いち".into(),
                        voice_actor_name: "Kenichi".into(),
                        voice_description: "Tokyo accent".into(),
                    }
                }
            ]
        );
        assert_eq!(
            data.readings,
            [VocabularyReading {
                accepted_answer: true,
                primary: true,
                reading: "いち".into(),
            }]
        );
        assert_eq!(data.reading_mnemonic, "When a vocab word is all alone and has no okurigana (hiragana attached to kanji) connected to it, it usually uses the kun'yomi reading. Numbers are an exception, however. When a number is all alone, with no kanji or okurigana, it is going to be the on'yomi reading, which you learned with the kanji.  Just remember this exception for alone numbers and you'll be able to read future number-related vocab to come.");
        assert_eq!(data.common.slug, "一");
        assert_eq!(data.common.spaced_repetition_system_id, 1);
    }

    #[test]
    fn test_vocab_serialize() {
        let readings = vec![VocabularyReading {
            accepted_answer: true,
            primary: true,
            reading: "this is a test reading".into(),
        }];
        let pronunciation_audios = vec![PronunciationAudio {
            content_type: "audio/mpeg".parse().expect("Mime"),
            url: "https://cdn.wanikani.com/audios/3018-subject-2467.ogg?1547862356"
                .parse()
                .expect("URL"),
            metadata: AudioMetadata {
                gender: Gender::Male,
                source_id: 555,
                pronunciation: "Pro".into(),
                voice_actor_id: 5,
                voice_actor_name: "Steve".into(),
                voice_description: "Example of metadata".into(),
            },
        }];
        let parts_of_speech = vec!["test".to_string()];
        let meanings = vec![Meaning {
            accepted_answer: true,
            primary: true,
            meaning: "test meaning".into(),
        }];
        let context_sentences = vec![ContextSentence {
            en: "This is a pen".into(),
            ja: "これはペンです".into(),
        }];
        let component_subject_ids = vec![6, 7, 3];
        let auxiliary_meanings = vec![];
        let common = SubjectCommon {
            auxiliary_meanings,
            created_at: Utc::now(),
            document_url: "https://some.url/test".parse().expect("URL"),
            hidden_at: None,
            lesson_position: 8,
            level: 1,
            meaning_mnemonic: "This is a test mnemonic".into(),
            meanings,
            slug: "💩".into(),
            spaced_repetition_system_id: 69,
        };
        let data = Vocabulary {
            characters: "💩🏩".into(),
            common,
            component_subject_ids,
            context_sentences,
            parts_of_speech,
            pronunciation_audios,
            reading_mnemonic: "this is another mnemonic".into(),
            readings,
        };
        let common = ResourceCommon {
            data_updated_at: Some(Utc::now()),
            object: ResourceType::Vocabulary,
            url: "https://some.url/common".parse().expect("URL"),
        };

        let vocab = Resource {
            common,
            data,
            id: 55555,
        };

        let json = serde_json::to_string(&vocab).expect("Serialize");

        let subject: Resource<Subject> = serde_json::from_str(&json).expect("Deserialize");

        let Subject::Vocabulary(subject_inner) = subject.data else {
            panic!("Incorrect subject type");
        };

        // Prove that Subject and Vocab Deserializations are identical
        assert_eq!(vocab.id, subject.id);
        assert_eq!(vocab.common, subject.common);
        assert_eq!(vocab.data, subject_inner);
    }

    #[test]
    fn test_kana_deserialize() {
        let json = include_str!("../test_files/kana_vocabulary.json");

        let subject: Resource<Subject> = serde_json::from_str(json).expect("Deserialize subject");
        let Subject::KanaVocabulary(subject_inner) = subject.data else {
            panic!("Incorrect subject type");
        };

        let vocab: Resource<KanaVocabulary> =
            serde_json::from_str(json).expect("Deserialize vocab");

        // Prove that Subject and Vocab Deserializations are identical
        assert_eq!(vocab.id, subject.id);
        assert_eq!(vocab.common, subject.common);
        assert_eq!(vocab.data, subject_inner);

        assert_eq!(vocab.id, 9210);

        let common = vocab.common;
        assert_eq!(common.object, ResourceType::KanaVocabulary);
        assert_eq!(
            common.url,
            "https://api.wanikani.com/v2/subjects/9210"
                .parse()
                .expect("URL")
        );
        assert_eq!(
            common.data_updated_at.expect("Timestamp"),
            DateTime::parse_from_rfc3339("2023-05-03T13:01:51.333012Z").expect("Timestamp")
        );

        let data = vocab.data;
        assert_eq!(
            data.common.created_at,
            DateTime::parse_from_rfc3339("2023-04-24T23:52:43.457614Z").expect("Timestamp")
        );
        assert_eq!(data.common.level, 8);
        assert_eq!(data.common.slug, "おやつ");
        assert!(data.common.hidden_at.is_none());
        assert_eq!(
            data.common.document_url,
            "https://www.wanikani.com/vocabulary/おやつ"
                .parse()
                .expect("URL")
        );
        assert_eq!(data.characters, "おやつ");
        assert_eq!(
            data.common.meanings,
            [Meaning {
                accepted_answer: true,
                primary: true,
                meaning: "Snack".into(),
            }]
        );
        assert!(data.common.auxiliary_meanings.is_empty());
        assert_eq!(data.parts_of_speech, ["noun"]);
        assert_eq!(data.common.meaning_mnemonic, "<reading>Oh yah! Two</reading> (<ja>おやつ</ja>) <vocabulary>snack</vocabulary>s, just for you. Imagine your two snacks. What are they? I bet they're delicious. Oh yah!\r\n\r\nYou can use <ja>おやつ</ja> to refer to a small amount of food eaten between meals, including candies and light meals like onigiri.");
        assert_eq!(
            data.context_sentences,
            [
                ContextSentence {
                    en: "Today I had a muffin for a snack.".into(),
                    ja: "今日はおやつにマフィンを食べた。".into()
                },
                ContextSentence {
                    en: "Shall we take a snack break?".into(),
                    ja: "そろそろおやつにする？".into()
                },
                ContextSentence {
                    en: "Kaori's snacks are always homemade!".into(),
                    ja: "カオリちゃんのおやつは、いつも手作りだよ！".into()
                }
            ]
        );
        assert_eq!(
            data.pronunciation_audios,
            [PronunciationAudio {
                content_type: "audio/webm".parse().expect("Mime"),
                url: "https://files.wanikani.com/w4yp5o02betioucki05lp6x78quy"
                    .parse()
                    .expect("URL"),
                metadata: AudioMetadata {
                    gender: Gender::Male,
                    source_id: 44757,
                    pronunciation: "おやつ".into(),
                    voice_actor_id: 2,
                    voice_actor_name: "Kenichi".into(),
                    voice_description: "Tokyo accent".into(),
                }
            }]
        );
        assert_eq!(data.common.lesson_position, 0);
        assert_eq!(data.common.spaced_repetition_system_id, 1);
    }

    #[test]
    fn test_kana_serialize() {
        let pronunciation_audios = vec![PronunciationAudio {
            content_type: "audio/mpeg".parse().expect("Mime"),
            url: "https://cdn.wanikani.com/audios/3018-subject-2467.ogg?1547862356"
                .parse()
                .expect("URL"),
            metadata: AudioMetadata {
                gender: Gender::Male,
                source_id: 555,
                pronunciation: "Pro".into(),
                voice_actor_id: 5,
                voice_actor_name: "Steve".into(),
                voice_description: "Example of metadata".into(),
            },
        }];
        let parts_of_speech = vec!["test".to_string()];
        let meanings = vec![Meaning {
            accepted_answer: true,
            primary: true,
            meaning: "test meaning".into(),
        }];
        let context_sentences = vec![ContextSentence {
            en: "This is a pen".into(),
            ja: "これはペンです".into(),
        }];
        let auxiliary_meanings = vec![];
        let common = SubjectCommon {
            auxiliary_meanings,
            created_at: Utc::now(),
            document_url: "https://some.url/test".parse().expect("URL"),
            hidden_at: None,
            lesson_position: 8,
            level: 1,
            meaning_mnemonic: "This is a test mnemonic".into(),
            meanings,
            slug: "💩".into(),
            spaced_repetition_system_id: 69,
        };
        let data = KanaVocabulary {
            characters: "💩🏩".into(),
            common,
            context_sentences,
            parts_of_speech,
            pronunciation_audios,
        };
        let common = ResourceCommon {
            data_updated_at: Some(Utc::now()),
            object: ResourceType::Vocabulary,
            url: "https://some.url/common".parse().expect("URL"),
        };

        let vocab = Resource {
            common,
            data,
            id: 55555,
        };

        let json = serde_json::to_string(&vocab).expect("Serialize");

        let subject: Resource<Subject> = serde_json::from_str(&json).expect("Deserialize");

        let Subject::KanaVocabulary(subject_inner) = subject.data else {
            panic!("Incorrect subject type");
        };

        // Prove that Subject and Vocab Deserializations are identical
        assert_eq!(vocab.id, subject.id);
        assert_eq!(vocab.common, subject.common);
        assert_eq!(vocab.data, subject_inner);
    }
}
