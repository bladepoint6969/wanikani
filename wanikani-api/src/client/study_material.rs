use url::Url;

use crate::{
    study_material::{CreateStudyMaterial, StudyMaterial, UpdateStudyMaterial},
    Collection, Error, Id, Resource, Timestamp,
};

use super::{Filter, WKClient};

const STUDY_MATERIAL_PATH: &str = "study_materials";

impl WKClient {
    /// Returns a collection of all study material, ordered by ascending
    /// `created_at`, 500 at a time.
    pub async fn get_study_materials(
        &self,
        filters: &StudyMaterialFilter,
    ) -> Result<Collection<StudyMaterial>, Error> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("Valid URL")
            .push(STUDY_MATERIAL_PATH);

        filters.apply_filters(&mut url);

        let req = self.client.get(url);

        self.do_request("get_subjects", req).await
    }

    /// Retrieves a specific study material by its `id`.
    pub async fn get_specific_study_material(
        &self,
        id: Id,
    ) -> Result<Resource<StudyMaterial>, Error> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("Valid URL")
            .push(STUDY_MATERIAL_PATH)
            .push(&id.to_string());

        let req = self.client.get(url);

        self.do_request("get_specific_subject", req).await
    }

    /// Creates a study material for a specific `subject_id`.
    ///
    /// The owner of the api key can only create one study_material per
    /// `subject_id`.
    pub async fn create_study_material(
        &self,
        material: &CreateStudyMaterial,
    ) -> Result<Resource<StudyMaterial>, Error> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("Valid URL")
            .push(STUDY_MATERIAL_PATH);

        let req = self.client.post(url).json(material);

        self.do_request("create_study_material", req).await
    }

    /// Updates a study material for a specific `id`.
    pub async fn update_study_material(
        &self,
        id: Id,
        material: &UpdateStudyMaterial,
    ) -> Result<Resource<StudyMaterial>, Error> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("Valid URL")
            .push(STUDY_MATERIAL_PATH)
            .push(id.to_string().as_str());

        let req = self.client.put(url).json(material);

        self.do_request("update_study_material", req).await
    }
}

#[cfg(feature = "study_material")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// The collection of study material records will be filtered on the parameters
/// provided.
pub struct StudyMaterialFilter {
    /// Return study materials with a matching value in the `hidden` attribute.
    pub hidden: Option<bool>,
    /// Only study material records where `data.id` matches one of the array
    /// values are returned.
    pub ids: Option<Vec<Id>>,
    /// Only study material records where `data.subject_id` matches one of the
    /// array values are returned.
    pub subject_ids: Option<Vec<Id>>,
    /// Only study material records where `data.subject_type` matches one of the
    /// array values are returned.
    pub subject_types: Option<Vec<crate::subject::SubjectType>>,
    /// Only study material records updated after this time are returned.
    pub updated_after: Option<Timestamp>,
}

#[cfg(feature = "study_material")]
impl Filter for StudyMaterialFilter {
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
        if let Some(ref ids) = self.subject_ids {
            query.append_pair(
                "subject_ids",
                ids.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
                    .as_str(),
            );
        }
        if let Some(ref types) = self.subject_types {
            query.append_pair(
                "subject_types",
                types
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
    async fn test_get_study_materials() {
        init_tests();

        let client = create_client();

        assert!(client
            .get_study_materials(&Default::default())
            .await
            .is_ok());
    }

    #[cfg(feature = "study_material")]
    #[tokio::test]
    async fn test_get_specific_study_material() {
        init_tests();

        let client = create_client();

        let study_materials = client
            .get_study_materials(&Default::default())
            .await
            .expect("Get all study_materials");

        if let Some(prog) = study_materials.data.get(0) {
            assert!(client.get_specific_study_material(prog.id).await.is_ok());
        } else {
            log::warn!("No study materials detected, this test should not be considered reliable");
        }
    }

    #[tokio::test]
    async fn test_update_study_material() {
        use crate::study_material::UpdateStudyMaterial;

        init_tests();

        let client = create_client();

        let study_materials = client
            .get_study_materials(&Default::default())
            .await
            .expect("Get all study_materials");

        if let Some(prog) = study_materials.data.get(0) {
            let update = UpdateStudyMaterial {
                meaning_note: prog.data.meaning_note.clone(),
                ..Default::default()
            };

            let new_prog = client
                .update_study_material(prog.id, &update)
                .await
                .expect("Successful Update");
            assert_eq!(prog, &new_prog);
        } else {
            log::warn!("No study materials detected, this test should not be considered reliable");
        }
    }
}
