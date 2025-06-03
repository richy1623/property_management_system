use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::db::model::ownership::OwnershipState;

/// Represents a plot of land within an estate.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Plot {
    /// Unique identifier of the plot.
    #[schema(example = 101)]
    pub plot_id: i32,

    /// ID of the estate the plot belongs to.
    #[schema(example = 1)]
    pub estate_id: i32,

    /// Plot number identifier.
    #[schema(example = "P-101")]
    pub plot_number: String,

    /// Phase of the estate the plot is in.
    #[schema(example = "Phase 1")]
    pub phase: String,

    /// Length of the plot in meters.
    #[schema(example = 40)]
    pub length: i32,

    /// Width of the plot in meters.
    #[schema(example = 30)]
    pub width: i32,
}

/// Request to create a new plot in an estate.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PlotCreationRequest {
    /// ID of the estate the plot belongs to.
    #[schema(example = 1)]
    pub estate_id: i32,

    /// Plot number identifier.
    #[schema(example = "P-101")]
    pub plot_number: String,

    /// Phase of the estate.
    #[schema(example = "Phase 1")]
    pub phase: String,

    /// Length of the plot in meters.
    #[schema(example = 40)]
    pub length: i32,

    /// Width of the plot in meters.
    #[schema(example = 30)]
    pub width: i32,
}

/// Request to filter plots based on criteria.
#[derive(Debug, Deserialize, ToSchema)]
pub struct GetPlotsRequest {
    /// Filter by plot number.
    #[schema(example = "P-101")]
    pub plot_number: Option<String>,

    /// Filter by estate ID.
    #[schema(example = 1)]
    pub estate_id: Option<i32>,

    /// Filter by estate phase.
    #[schema(example = "Phase 1")]
    pub phase: Option<String>,

    /// List of allowed ownership states to filter by.
    pub allowed_ownership_states: Option<Vec<OwnershipState>>,
}

/// A list of plots.
#[derive(Debug, Serialize, ToSchema)]
pub struct PlotList {
    /// Collection of plot objects.
    pub plots: Vec<Plot>,
}
