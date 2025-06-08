use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::db::model::ownership::OwnershipState;

/// Represents a plot of land within an estate.
#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
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
#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
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
#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
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
    #[schema(example = "[PendingContract]")]
    pub allowed_ownership_states: Option<Vec<OwnershipState>>,
}

/// A list of plots.
#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
pub struct PlotList {
    /// Collection of plot objects.
    pub plots: Vec<Plot>,
}

mod converter {
    use super::*;
    use crate::service::property_management::property_management_proto::{
        GetPlotsRequest as ProtoGetPlotsRequest, Plot as ProtoPlot,
        PlotCreationRequest as ProtoPlotCreationRequest, PlotList as ProtoPlotList,
    };
    impl From<Plot> for ProtoPlot {
        fn from(plot: Plot) -> Self {
            ProtoPlot {
                plot_id: plot.plot_id,
                estate_id: plot.estate_id,
                plot_number: plot.plot_number,
                phase: plot.phase,
                length: plot.length,
                width: plot.width,
            }
        }
    }

    impl From<(PlotCreationRequest, String)> for ProtoPlotCreationRequest {
        fn from((req, jwt_token): (PlotCreationRequest, String)) -> Self {
            ProtoPlotCreationRequest {
                jwt_token: jwt_token,
                estate_id: req.estate_id,
                plot_number: req.plot_number,
                phase: req.phase,
                length: req.length,
                width: req.width,
            }
        }
    }

    impl From<(GetPlotsRequest, String)> for ProtoGetPlotsRequest {
        fn from((req, jwt_token): (GetPlotsRequest, String)) -> Self {
            ProtoGetPlotsRequest {
                jwt_token: jwt_token,
                plot_number: req.plot_number,
                estate_id: req.estate_id,
                phase: req.phase,
                allowed_ownership_states: req
                    .allowed_ownership_states
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| s as i32)
                    .collect(),
            }
        }
    }

    impl From<ProtoPlot> for Plot {
        fn from(p: ProtoPlot) -> Self {
            Self {
                plot_id: p.plot_id,
                estate_id: p.estate_id,
                plot_number: p.plot_number,
                phase: p.phase,
                length: p.length,
                width: p.width,
            }
        }
    }

    impl From<ProtoPlotList> for PlotList {
        fn from(p: ProtoPlotList) -> Self {
            Self {
                plots: p.plots.into_iter().map(Plot::from).collect(),
            }
        }
    }

    impl From<ProtoPlotCreationRequest> for PlotCreationRequest {
        fn from(plot_creation_request: ProtoPlotCreationRequest) -> Self {
            PlotCreationRequest {
                estate_id: plot_creation_request.estate_id,
                plot_number: plot_creation_request.plot_number,
                phase: plot_creation_request.phase,
                length: plot_creation_request.length,
                width: plot_creation_request.width,
            }
        }
    }

    impl From<ProtoGetPlotsRequest> for GetPlotsRequest {
        fn from(get_plots_request: ProtoGetPlotsRequest) -> Self {
            GetPlotsRequest {
                plot_number: get_plots_request.plot_number,
                estate_id: get_plots_request.estate_id,
                phase: get_plots_request.phase,
                allowed_ownership_states: Some(
                    get_plots_request
                        .allowed_ownership_states
                        .into_iter()
                        .map(|state| {
                            OwnershipState::try_from(state).expect("Invalid Ownership State")
                        })
                        .collect(),
                ),
            }
        }
    }
}
