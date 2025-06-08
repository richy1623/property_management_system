use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::db::model::ownership::OwnershipState;

/// Ownership information linking a user to a plot.
#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
pub struct Ownership {
    #[schema(example = 1001)]
    pub ownership_id: i32,

    #[schema(example = 101)]
    pub user_id: i32,

    #[schema(example = "john_doe")]
    pub user_name: String,

    #[schema(example = 101)]
    pub plot_id: i32,

    pub state: OwnershipState,

    #[schema(example = 5_000_000)]
    pub total_sale_value: i32,

    #[schema(example = 50_000)]
    pub monthly_repayment_amount: i32,
}

/// Request for creating a new ownership.
#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
pub struct OwnershipCreationRequest {
    #[schema(example = 101)]
    pub user_id: i32,

    #[schema(example = 101)]
    pub plot_id: i32,

    pub state: OwnershipState,

    #[schema(example = 5_000_000)]
    pub total_sale_value: i32,

    #[schema(example = 50_000)]
    pub monthly_repayment_amount: i32,
}

/// Request for updating an ownership.
#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
pub struct OwnershipUpdateRequest {
    #[schema(example = 1001)]
    pub ownership_id: i32,

    pub state: Option<OwnershipState>,

    #[schema(example = 5_000_000)]
    pub total_sale_value: Option<i32>,

    #[schema(example = 50_000)]
    pub monthly_repayment_amount: Option<i32>,
}
