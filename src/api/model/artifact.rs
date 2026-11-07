use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::db::model::artifact::ArtifactServiceType;

/// Reference object for an artifact (lightweight)
#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
pub struct ArtifactReference {
    #[schema(example = 5001)]
    pub artifact_id: i32,

    #[schema(example = "receipt.pdf")]
    pub file_display_name: String,
}

/// Represents an uploaded artifact with metadata and file content
#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
pub struct Artifact {
    #[schema(example = 5001)]
    pub artifact_id: i32,

    #[schema(example = "receipt.pdf")]
    pub file_display_name: String,

    #[schema(example = "2025-06-01", format = "date")]
    pub upload_date: NaiveDate,

    #[schema(example = 101)]
    pub uploaded_by: i32,

    /// Binary content of the artifact
    #[schema(value_type = String, format = Binary)]
    pub file_content: Vec<u8>,
}

/// File to be saved to as an artifact
#[derive(Debug, ToSchema, Serialize, Deserialize, PartialEq)]
pub struct ArtifactFile {
    #[schema(example = "receipt.pdf")]
    /// Display name of the uploaded file
    pub file_name: String,

    #[schema(value_type = String, format = Binary)]
    /// Raw file contents as base64
    pub file_content: Vec<u8>,
}

/// Request payload for uploading an artifact to the system.
#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
pub struct ArtifactUploadRequest {
    /// File contents and metadata for the artifact being uploaded.
    pub artifact_file: ArtifactFile,

    /// Service domain the artifact belongs to.
    #[schema(example = "Ownership")]
    pub artifact_service: ArtifactServiceType,

    /// ID of the record in the service domain this artifact is associated with.
    #[schema(example = 101)]
    pub service_reference_id: i32,
}
