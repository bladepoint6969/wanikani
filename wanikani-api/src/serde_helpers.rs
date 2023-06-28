use serde::{Deserialize, Serialize};

use crate::study_material;

#[derive(Debug, Deserialize, Serialize)]
struct CreateStudyMaterial {
    subject_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    meaning_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reading_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    meaning_synonyms: Option<Vec<String>>,
}

impl From<CreateStudyMaterial> for study_material::CreateStudyMaterial {
    fn from(value: CreateStudyMaterial) -> Self {
        Self {
            subject_id: value.subject_id,
            meaning_note: value.meaning_note,
            meaning_synonyms: value.meaning_synonyms,
            reading_note: value.reading_note,
        }
    }
}

impl From<study_material::CreateStudyMaterial> for CreateStudyMaterial {
    fn from(value: study_material::CreateStudyMaterial) -> Self {
        Self {
            subject_id: value.subject_id,
            meaning_note: value.meaning_note,
            reading_note: value.reading_note,
            meaning_synonyms: value.meaning_synonyms,
        }
    }
}

impl From<CreateStudyMaterialWrapper> for study_material::CreateStudyMaterial {
    fn from(value: CreateStudyMaterialWrapper) -> Self {
        value.study_material.into()
    }
}

impl From<study_material::CreateStudyMaterial> for CreateStudyMaterialWrapper {
    fn from(value: study_material::CreateStudyMaterial) -> Self {
        Self {
            study_material: value.into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateStudyMaterialWrapper {
    study_material: CreateStudyMaterial,
}

impl From<UpdateStudyMaterial> for study_material::UpdateStudyMaterial {
    fn from(value: UpdateStudyMaterial) -> Self {
        Self {
            meaning_note: value.meaning_note,
            reading_note: value.reading_note,
            meaning_synonyms: value.meaning_synonyms,
        }
    }
}

impl From<study_material::UpdateStudyMaterial> for UpdateStudyMaterial {
    fn from(value: study_material::UpdateStudyMaterial) -> Self {
        Self {
            meaning_note: value.meaning_note,
            reading_note: value.reading_note,
            meaning_synonyms: value.meaning_synonyms,
        }
    }
}

impl From<UpdateStudyMaterialWrapper> for study_material::UpdateStudyMaterial {
    fn from(value: UpdateStudyMaterialWrapper) -> Self {
        value.study_material.into()
    }
}

impl From<study_material::UpdateStudyMaterial> for UpdateStudyMaterialWrapper {
    fn from(value: study_material::UpdateStudyMaterial) -> Self {
        Self {
            study_material: value.into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct UpdateStudyMaterial {
    #[serde(skip_serializing_if = "Option::is_none")]
    meaning_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reading_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    meaning_synonyms: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateStudyMaterialWrapper {
    study_material: UpdateStudyMaterial,
}

pub mod update_prefs {
    use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};

    use crate::user::UpdatePreferences;

    #[derive(Deserialize, Serialize)]
    struct Wrapper {
        preferences: UpdatePreferences,
    }

    pub fn serialize<S>(value: &UpdatePreferences, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("user", 1)?;
        state.serialize_field("preferences", value)?;
        state.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<UpdatePreferences, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wrapper = Wrapper::deserialize(deserializer)?;
        Ok(wrapper.preferences)
    }
}