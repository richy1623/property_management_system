use diesel::Queryable;
use serde::{Deserialize, Serialize};

use crate::model::payment::PaymentState;

#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct PaymentAudit {
    pub audit_id: i32,
    pub payment_id: i32,
    pub previous_state: PaymentState,
    pub new_state: PaymentState,
    pub change_timestamp: chrono::NaiveDateTime,
    pub changed_by_user: i32,
    pub comment: String,
}
