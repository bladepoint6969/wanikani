use std::{env, io};

use wanikani_api::{
    client::{SubjectFilter, WKClient},
    subject::SubjectType,
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let api_key = env::var("API_KEY").expect("API key is set");

    let client = WKClient::new(api_key, reqwest::Client::default());

    let filters = SubjectFilter {
        types: Some(vec![SubjectType::Radical, SubjectType::Kanji]),
        ..SubjectFilter::default()
    };

    let mut collection = client.get_subjects(&filters).await.expect("Get Subjects");

    let mut subjects = collection.data;
    log::info!(
        "Total of {} subjects to download, have {}",
        collection.total_count,
        subjects.len()
    );

    while let Some(ref next_url) = collection.pages.next_url {
        collection = client
            .get_resource_by_url(next_url)
            .await
            .expect("Next page");
        subjects.append(&mut collection.data);

        log::info!(
            "Total of {} subjects to download, have {}",
            collection.total_count,
            subjects.len()
        );
    }

    serde_json::to_writer_pretty(io::stdout(), &subjects).expect("Serialize to stdout");
}
