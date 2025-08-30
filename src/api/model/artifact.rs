use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
