tonic::include_proto!("property_management_system.artifact_management");

use chrono::{Datelike, NaiveDate};

use crate::db::model::artifact::Artifact as ArtifactModel;

impl From<NaiveDate> for Date {
    fn from(d: NaiveDate) -> Self {
        Date {
            day: d.day().try_into().unwrap(),
            year: d.year_ce().1.try_into().unwrap(),
            month: <u32 as TryInto<i32>>::try_into(d.month0()).unwrap() + 1,
        }
    }
}
impl From<Date> for NaiveDate {
    fn from(d: Date) -> Self {
        NaiveDate::from_ymd_opt(d.year, d.month as u32, d.day as u32)
            .expect("Invalid date from protobuf")
    }
}

impl Artifact {
    pub fn from_model(artifact: &ArtifactModel, file_content: Vec<u8>) -> Self {
        Artifact {
            artifact_id: artifact.artifact_id,
            file_display_name: artifact.file_display_name.clone(),
            file_content,
            upload_date: Some(artifact.upload_date.into()),
            uploaded_by: artifact.uploaded_by,
        }
    }
}

impl From<&ArtifactModel> for ArtifactRef {
    fn from(artifact: &ArtifactModel) -> Self {
        ArtifactRef {
            artifact_id: artifact.artifact_id,
            file_display_name: artifact.file_display_name.clone(),
            upload_date: Some(artifact.upload_date.into()),
            uploaded_by: artifact.uploaded_by,
        }
    }
}
