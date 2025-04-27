use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

use crate::schema::payment;

#[derive(Debug, Clone, PartialEq, Eq, DbEnum, Serialize, Deserialize)]
#[db_enum(existing_type_path = "crate::schema::sql_types::PaymentState")]
pub enum PaymentState {
    Pending,
    Completed,
    Failed,
}

#[derive(
    Queryable, Selectable, Insertable, Debug, PartialEq, Eq, Clone, Serialize, Deserialize,
)]
#[diesel(table_name = payment)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Payment {
    pub payment_id: i32,
    pub ownership_id: i32,
    pub date: chrono::NaiveDate,
    pub amount: i32,
    pub state: PaymentState,
    pub comment: String,
    pub reference: String,
    pub last_changed_by_user: i32,
    pub proof_of_payment_path: String,
}
