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

    #[schema(example = OwnershipState::PendingContract)]
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

    #[schema(example = OwnershipState::PendingContract)]
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

    #[schema(example = OwnershipState::PendingContract)]
    pub state: Option<OwnershipState>,

    #[schema(example = 5_000_000)]
    pub total_sale_value: Option<i32>,

    #[schema(example = 50_000)]
    pub monthly_repayment_amount: Option<i32>,
}

/// Filter parameters for querying ownerships
#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
pub struct OwnershipFilter {
    /// Optional user ID to filter ownerships
    #[schema(example = 10)]
    pub user_id: Option<i32>,

    /// Optional plot ID to filter ownerships
    #[schema(example = 5)]
    pub plot_id: Option<i32>,

    /// Optional state to filter ownerships
    #[schema(example = "ACTIVE")]
    pub state: Option<OwnershipState>,
}

mod converter {
    use super::*;
    use crate::db::model::ownership::Ownership as OwnershipModel;
    use crate::service::property_management::property_management_proto::{
        Ownership as ProtoOwnership, OwnershipCreationRequest as ProtoOwnershipCreationRequest,
        OwnershipUpdateRequest as ProtoOwnershipUpdateRequest,
    };

    impl From<Ownership> for ProtoOwnership {
        fn from(ownership: Ownership) -> Self {
            Self {
                ownership_id: ownership.ownership_id,
                user_id: ownership.user_id,
                user_name: ownership.user_name,
                plot_id: ownership.plot_id,
                state: ownership.state.into(),
                total_sale_value: ownership.total_sale_value,
                monthly_repayment_amount: ownership.monthly_repayment_amount,
            }
        }
    }

    impl From<(OwnershipCreationRequest, String)> for ProtoOwnershipCreationRequest {
        fn from((ownership_creation_request, jwt): (OwnershipCreationRequest, String)) -> Self {
            Self {
                jwt_token: jwt,
                user_id: ownership_creation_request.user_id,
                plot_id: ownership_creation_request.plot_id,
                state: ownership_creation_request.state.into(),
                total_sale_value: ownership_creation_request.total_sale_value,
                monthly_repayment_amount: ownership_creation_request.monthly_repayment_amount,
            }
        }
    }

    impl From<(OwnershipUpdateRequest, String)> for ProtoOwnershipUpdateRequest {
        fn from((ownership_update_request, jwt): (OwnershipUpdateRequest, String)) -> Self {
            Self {
                jwt_token: jwt,
                ownership_id: ownership_update_request.ownership_id,
                state: ownership_update_request.state.map(Into::into),
                total_sale_value: ownership_update_request.total_sale_value.map(Into::into),
                monthly_repayment_amount: ownership_update_request
                    .monthly_repayment_amount
                    .map(Into::into),
            }
        }
    }

    impl From<ProtoOwnership> for Ownership {
        fn from(ownership: ProtoOwnership) -> Self {
            Self {
                ownership_id: ownership.ownership_id,
                user_id: ownership.user_id,
                user_name: ownership.user_name,
                plot_id: ownership.plot_id,
                state: ownership.state.try_into().unwrap(),
                total_sale_value: ownership.total_sale_value,
                monthly_repayment_amount: ownership.monthly_repayment_amount,
            }
        }
    }

    impl From<ProtoOwnershipCreationRequest> for OwnershipCreationRequest {
        fn from(ownership_creation_request: ProtoOwnershipCreationRequest) -> Self {
            Self {
                user_id: ownership_creation_request.user_id,
                plot_id: ownership_creation_request.plot_id,
                state: ownership_creation_request.state.try_into().unwrap(),
                total_sale_value: ownership_creation_request.total_sale_value,
                monthly_repayment_amount: ownership_creation_request.monthly_repayment_amount,
            }
        }
    }

    impl From<ProtoOwnershipUpdateRequest> for OwnershipUpdateRequest {
        fn from(ownership_update_request: ProtoOwnershipUpdateRequest) -> Self {
            Self {
                ownership_id: ownership_update_request.ownership_id,
                state: ownership_update_request
                    .state
                    .map(|state| state.try_into().unwrap()),
                total_sale_value: ownership_update_request.total_sale_value,
                monthly_repayment_amount: ownership_update_request.monthly_repayment_amount,
            }
        }
    }

    impl From<(OwnershipModel, String)> for Ownership {
        fn from((ownership, username): (OwnershipModel, String)) -> Self {
            Self {
                ownership_id: ownership.ownership_id,
                user_id: ownership.user_id,
                user_name: username,
                plot_id: ownership.plot_id,
                state: ownership.state,
                total_sale_value: ownership.total_sale_value,
                monthly_repayment_amount: ownership.monthly_repayment_amount,
            }
        }
    }
}
