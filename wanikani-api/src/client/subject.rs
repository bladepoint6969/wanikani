use url::Url;

use crate::{
    subject::{Subject, WaniKaniSubject},
    Collection, Error, Resource, Timestamp,
};

use super::{Filter, WKClient};

const SUBJECT_PATH: &str = "subjects";

impl WKClient {
    /// Returns a collection of all subjects, ordered by ascending
    /// `created_at`, 1000 at a time.
    pub async fn get_subjects(
        &self,
        filters: &SubjectFilter,
    ) -> Result<Collection<Subject>, Error> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("Valid URL")
            .push(SUBJECT_PATH);

        filters.apply_filters(&mut url);

        let req = self.client.get(url);

        self.do_request("get_subjects", req).await
    }

    /// Retrieves a specific subject by its `id`. The structure of the
    /// response depends on the subject type.
    pub async fn get_specific_subject<T: WaniKaniSubject>(
        &self,
        id: u64,
    ) -> Result<Resource<T>, Error> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("Valid URL")
            .push(SUBJECT_PATH)
            .push(&id.to_string());

        let req = self.client.get(url);

        self.do_request("get_specific_subject", req).await
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Filter parameters for subjects
pub struct SubjectFilter {
    /// Only subjects where `data.id` matches one of the array values are
    /// returned.
    pub ids: Option<Vec<u64>>,
    /// Return subjects of the specified types.
    pub types: Option<Vec<crate::subject::SubjectType>>,
    /// Return subjects of the specified slug.
    pub slugs: Option<Vec<String>>,
    /// Return subjects at the specified levels.
    pub levels: Option<Vec<u32>>,
    /// Return subjects which are or are not hidden from the user-facing
    /// application.
    pub hidden: Option<bool>,
    /// Only subjects updated after this time are returned.
    pub updated_after: Option<Timestamp>,
}

#[cfg(feature = "subject")]
impl Filter for SubjectFilter {
    fn apply_filters(&self, url: &mut Url) {
        let mut query = url.query_pairs_mut();
        if let Some(ref ids) = self.ids {
            query.append_pair(
                "ids",
                ids.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
                    .as_str(),
            );
        }
        if let Some(ref types) = self.types {
            query.append_pair(
                "types",
                types
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
                    .as_str(),
            );
        }
        if let Some(ref slugs) = self.slugs {
            query.append_pair("slugs", slugs.join(",").as_str());
        }
        if let Some(ref levels) = self.levels {
            query.append_pair(
                "levels",
                levels
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
                    .as_str(),
            );
        }
        if let Some(hidden) = self.hidden {
            query.append_pair("hidden", hidden.to_string().as_str());
        }
        if let Some(updated_after) = self.updated_after {
            query.append_pair("updated_after", updated_after.to_rfc3339().as_str());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::client::{create_client, init_tests};

    #[tokio::test]
    async fn test_get_subjects() {
        use super::SubjectFilter;

        init_tests();

        let client = create_client();
        let filters = SubjectFilter {
            levels: Some(vec![2]),
            ..SubjectFilter::default()
        };
        assert!(client.get_subjects(&filters).await.is_ok());
    }

    #[cfg(feature = "subject")]
    #[tokio::test]
    async fn test_get_specific_subject() {
        use crate::{
            subject::{KanaVocabulary, Kanji, Radical, Subject, Vocabulary},
            Resource,
        };

        init_tests();

        let client = create_client();
        let mut subject: Resource<Subject> =
            client.get_specific_subject(1).await.expect("Get subject");
        let radical: Resource<Radical> = client.get_specific_subject(1).await.expect("Get radical");

        let Subject::Radical(subject_inner) = subject.data else {
            panic!("Incorrect type (Should be radical)");
        };

        assert_eq!(subject.id, radical.id);
        assert_eq!(subject.common, radical.common);
        assert_eq!(subject_inner, radical.data);

        subject = client.get_specific_subject(440).await.expect("Get subject");
        let kanji: Resource<Kanji> = client.get_specific_subject(440).await.expect("Get kanji");

        let Subject::Kanji(subject_inner) = subject.data else {
            panic!("Incorrect type (Should be kanji)");
        };

        assert_eq!(subject.id, kanji.id);
        assert_eq!(subject.common, kanji.common);
        assert_eq!(subject_inner, kanji.data);

        subject = client
            .get_specific_subject(2467)
            .await
            .expect("Get subject");
        let vocab: Resource<Vocabulary> =
            client.get_specific_subject(2467).await.expect("Get vocab");

        let Subject::Vocabulary(subject_inner) = subject.data else {
            panic!("Incorrect type (Should be kanji)");
        };

        assert_eq!(subject.id, vocab.id);
        assert_eq!(subject.common, vocab.common);
        assert_eq!(subject_inner, vocab.data);

        subject = client
            .get_specific_subject(9177)
            .await
            .expect("Get subject");
        let vocab: Resource<KanaVocabulary> = client
            .get_specific_subject(9177)
            .await
            .expect("Get kana vocab");

        let Subject::KanaVocabulary(subject_inner) = subject.data else {
            panic!("Incorrect type (Should be kanji)");
        };

        assert_eq!(subject.id, vocab.id);
        assert_eq!(subject.common, vocab.common);
        assert_eq!(subject_inner, vocab.data);
    }
}
