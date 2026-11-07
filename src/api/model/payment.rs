use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    api::model::artifact::{ArtifactFile, ArtifactReference},
    db::model::payment::PaymentState,
};

/// Represents a payment linked to an ownership.
#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
pub struct Payment {
    #[schema(example = 1001)]
    pub payment_id: i32,

    #[schema(example = 2001)]
    pub ownership_id: i32,

    #[schema(example = "2025-06-01", format = "date")]
    pub date: NaiveDate,

    #[schema(example = 50_000)]
    pub amount: i32,

    pub state: PaymentState,

    #[schema(example = "Payment for May 2025")]
    pub comment: String,

    #[schema(example = "REF123456")]
    pub reference: String,

    #[schema(example = 101)]
    pub last_changed_by_user: i32,

    pub proof_of_payment: Option<ArtifactReference>,
}

///  Request for creating a new payment.
#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
pub struct PaymentCreationRequest {
    #[schema(example = 2001)]
    pub ownership_id: i32,

    #[schema(example = "2025-06-01", format = "date")]
    pub date: NaiveDate,

    #[schema(example = 50_000)]
    pub amount: i32,

    #[schema(example = "REF123456")]
    pub reference: String,

    #[schema(example = 101)]
    pub user_creating_payment: i32,

    pub proof_of_payment_file: Option<ArtifactFile>,
}

/// List wrapper for payments
#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
pub struct PaymentList {
    pub payments: Vec<Payment>,
}

/// Request to update payment state with optional comment.
#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
pub struct UpdatePaymentStateRequest {
    #[schema(example = 1001)]
    pub payment_id: i32,

    #[schema(example = PaymentState::PendingApproval)]
    pub state: PaymentState,

    #[schema(example = "Marked as completed after verification")]
    pub comment: String,
}

#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
pub struct GetOwnershipPaymentsRequest {
    #[schema(example = "[1001, 1002]")]
    pub ownership_ids: Vec<i32>,
}

#[derive(Debug, ToSchema, Deserialize, Serialize, PartialEq)]
pub struct PaymentsInStateQuery {
    #[schema(example = PendingApproval)]
    pub state: PaymentState,
    #[schema(example = 0)]
    pub offset: Option<i32>,
    #[schema(example = 10)]
    pub limit: Option<i32>,
}

mod converter {
    use super::*;
    use crate::{
        db::model::{artifact::Artifact as ArtifactModel, payment::Payment as PaymentModel},
        service::payment_management::payment_management_proto::{
            GetPaymentsInStateRequest as ProtoGetPaymentsInStateRequest, Payment as ProtoPayment,
            PaymentList as ProtoPaymentList,
            PaymentStateUpdateRequest as ProtoPaymentStateUpdateRequest,
        },
    };

    impl From<Payment> for ProtoPayment {
        fn from(payment: Payment) -> Self {
            Self {
                payment_id: payment.payment_id,
                ownership_id: payment.ownership_id,
                date: Some(payment.date.into()),
                amount: payment.amount,
                state: payment.state.into(),
                comment: payment.comment,
                reference: payment.reference,
                last_changed_by_user: payment.last_changed_by_user,
            }
        }
    }

    impl From<PaymentList> for ProtoPaymentList {
        fn from(payment_list: PaymentList) -> Self {
            Self {
                payments: payment_list.payments.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl From<(UpdatePaymentStateRequest, String)> for ProtoPaymentStateUpdateRequest {
        fn from((payment_state_update_request, jwt): (UpdatePaymentStateRequest, String)) -> Self {
            Self {
                jwt_token: jwt,
                payment_id: payment_state_update_request.payment_id,
                state: payment_state_update_request.state.into(),
                comment: payment_state_update_request.comment,
            }
        }
    }

    impl From<(PaymentsInStateQuery, String)> for ProtoGetPaymentsInStateRequest {
        fn from((query, jwt_token): (PaymentsInStateQuery, String)) -> Self {
            Self {
                jwt_token,
                state: query.state.into(),
                offset: query.offset.unwrap_or(0),
                limit: query.limit.unwrap_or(100),
            }
        }
    }

    impl From<ProtoPayment> for Payment {
        fn from(payment: ProtoPayment) -> Self {
            Self {
                payment_id: payment.payment_id,
                ownership_id: payment.ownership_id,
                date: payment
                    .date
                    .map(Into::into)
                    .expect("Missing date in proto::Payment"),
                amount: payment.amount,
                state: payment.state.try_into().unwrap(),
                comment: payment.comment,
                reference: payment.reference,
                last_changed_by_user: payment.last_changed_by_user,
                // TODO URGENT fix this to return the path
                proof_of_payment: None,
            }
        }
    }

    impl From<ProtoPaymentList> for PaymentList {
        fn from(payment_list: ProtoPaymentList) -> Self {
            Self {
                payments: payment_list.payments.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl From<ProtoPaymentStateUpdateRequest> for UpdatePaymentStateRequest {
        fn from(payment_state_update_request: ProtoPaymentStateUpdateRequest) -> Self {
            Self {
                payment_id: payment_state_update_request.payment_id,
                state: payment_state_update_request.state.try_into().unwrap(),
                comment: payment_state_update_request.comment,
            }
        }
    }

    impl From<ProtoGetPaymentsInStateRequest> for PaymentsInStateQuery {
        fn from(get_payments_in_state_request: ProtoGetPaymentsInStateRequest) -> Self {
            Self {
                state: get_payments_in_state_request.state.try_into().unwrap(),
                offset: Some(get_payments_in_state_request.offset),
                limit: Some(get_payments_in_state_request.limit),
            }
        }
    }

    impl From<(&PaymentModel, Option<&ArtifactModel>)> for Payment {
        fn from((payment, artifact): (&PaymentModel, Option<&ArtifactModel>)) -> Self {
            let artifact_reference = if let Some(artifact) = artifact {
                Some(ArtifactReference {
                    artifact_id: artifact.artifact_id,
                    file_display_name: artifact.file_display_name.clone(),
                })
            } else {
                None
            };
            Self {
                payment_id: payment.payment_id,
                ownership_id: payment.ownership_id,
                date: payment.date,
                amount: payment.amount,
                state: payment.state.clone(),
                comment: payment.comment.clone(),
                reference: payment.reference.clone(),
                last_changed_by_user: payment.last_changed_by_user,
                proof_of_payment: artifact_reference,
            }
        }
    }
}
