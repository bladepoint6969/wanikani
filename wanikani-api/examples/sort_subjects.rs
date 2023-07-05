use serde::{Deserialize, Serialize};
use std::{env, io::stdout};
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

        user.data
            .preferences
            .lessons_presentation_order
            .order_subjects(&mut rng, sub_common, other_common)
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
