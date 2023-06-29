use std::{env, io::stdout};

use rand::Rng;
use serde::{Deserialize, Serialize};
use wanikani_api::{
    client::SubjectFilter, prelude::WKClient, subject::Subject, user::LessonPresentationOrder,
    Resource,
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenvy::dotenv().ok();

    let api_key = env::var("API_KEY").expect("API key is set");

    let client = WKClient::new(api_key, reqwest::Client::default());

    let filters = &SubjectFilter {
        levels: Some(vec![1, 2]),
        ..Default::default()
    };

    let user = client.get_user_information().await.expect("User prefs");

    let mut subjects = client.get_subjects(filters).await.expect("subjects").data;
    let mut rng = rand::thread_rng();

    subjects.sort_by(|subject, other| {
        let sub_common = match subject.data {
            Subject::KanaVocabulary(ref subject) => &subject.common,
            Subject::Kanji(ref subject) => &subject.common,
            Subject::Radical(ref subject) => &subject.common,
            Subject::Vocabulary(ref subject) => &subject.common,
        };

        let other_common = match other.data {
            Subject::KanaVocabulary(ref subject) => &subject.common,
            Subject::Kanji(ref subject) => &subject.common,
            Subject::Radical(ref subject) => &subject.common,
            Subject::Vocabulary(ref subject) => &subject.common,
        };

        match user.data.preferences.lessons_presentation_order {
            LessonPresentationOrder::AscendingLevelThenSubject => {
                match sub_common.level.cmp(&other_common.level) {
                    std::cmp::Ordering::Equal => sub_common
                        .lesson_position
                        .cmp(&other_common.lesson_position),
                    ord => ord,
                }
            }
            LessonPresentationOrder::AscendingLevelThenShuffled => {
                match sub_common.level.cmp(&other_common.level) {
                    std::cmp::Ordering::Equal => rng.gen::<u32>().cmp(&rng.gen()),
                    ord => ord,
                }
            }
            LessonPresentationOrder::Shuffled => rng.gen::<u32>().cmp(&rng.gen()),
        }
    });

    let sorted = SortedSubjects {
        order: user.data.preferences.lessons_presentation_order,
        subjects,
    };

    serde_json::to_writer_pretty(stdout(), &sorted).expect("Write to stdout");
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct SortedSubjects {
    order: LessonPresentationOrder,
    subjects: Vec<Resource<Subject>>,
}
