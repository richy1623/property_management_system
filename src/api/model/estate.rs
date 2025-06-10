use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A residential estate.
#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
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
#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
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
#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
pub struct EstateList {
    /// Collection of estate objects.
    pub estates: Vec<Estate>,
}

mod converter {
    use super::*;
    use crate::service::property_management::property_management_proto::{
        Estate as ProtoEstate, EstateCreationRequest as ProtoEstateCreationRequest,
        EstateList as ProtoEstateList,
    };
    impl From<Estate> for ProtoEstate {
        fn from(estate: Estate) -> Self {
            Self {
                estate_id: estate.estate_id,
                name: estate.name,
                location: estate.location,
                district: estate.district,
                landmark: estate.landmark,
            }
        }
    }

    impl From<EstateList> for ProtoEstateList {
        fn from(estate_list: EstateList) -> Self {
            Self {
                estates: estate_list.estates.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl From<(EstateCreationRequest, String)> for ProtoEstateCreationRequest {
        fn from((estate_creation_request, jwt): (EstateCreationRequest, String)) -> Self {
            Self {
                jwt_token: jwt,
                name: estate_creation_request.name,
                location: estate_creation_request.location,
                district: estate_creation_request.district,
                landmark: estate_creation_request.landmark,
            }
        }
    }

    impl From<ProtoEstate> for Estate {
        fn from(estate: ProtoEstate) -> Self {
            Self {
                estate_id: estate.estate_id,
                name: estate.name,
                location: estate.location,
                district: estate.district,
                landmark: estate.landmark,
            }
        }
    }

    impl From<ProtoEstateList> for EstateList {
        fn from(estate_list: ProtoEstateList) -> Self {
            Self {
                estates: estate_list.estates.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl From<ProtoEstateCreationRequest> for EstateCreationRequest {
        fn from(estate_creation_request: ProtoEstateCreationRequest) -> Self {
            Self {
                name: estate_creation_request.name,
                location: estate_creation_request.location,
                district: estate_creation_request.district,
                landmark: estate_creation_request.landmark,
            }
        }
    }
}
