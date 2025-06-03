use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A residential estate.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Estate {
    /// Unique identifier of the estate.
    #[schema(example = 1)]
    pub estate_id: i32,

    /// Name of the estate.
    #[schema(example = "Sunrise Estate")]
    pub name: String,

    /// Location of the estate.
    #[schema(example = "Durban")]
    pub location: String,

    /// District where the estate is situated.
    #[schema(example = "Westlake")]
    pub district: String,

    /// Notable landmark near the estate.
    #[schema(example = "Near the big mall")]
    pub landmark: String,
}

/// Request to create a new estate.
#[derive(Debug, Deserialize, ToSchema)]
pub struct EstateCreationRequest {
    /// Name of the estate.
    #[schema(example = "Sunrise Estate")]
    pub name: String,

    /// Location of the estate.
    #[schema(example = "Durban")]
    pub location: String,

    /// District where the estate is situated.
    #[schema(example = "Westlake")]
    pub district: String,

    /// Notable landmark near the estate.
    #[schema(example = "Near the big mall")]
    pub landmark: String,
}

/// A list of estates.
#[derive(Debug, Serialize, ToSchema)]
pub struct EstateList {
    /// Collection of estate objects.
    pub estates: Vec<Estate>,
}
