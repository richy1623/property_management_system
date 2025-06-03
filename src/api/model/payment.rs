use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::db::model::payment::PaymentState;

/// Represents a payment linked to an ownership.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
}

/// List wrapper for payments
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaymentList {
    pub payments: Vec<Payment>,
}

/// Request to update payment state with optional comment.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePaymentStateRequest {
    #[schema(example = 1001)]
    pub payment_id: i32,

    pub state: PaymentState,

    #[schema(example = "Marked as completed after verification")]
    pub comment: Option<String>,
}
