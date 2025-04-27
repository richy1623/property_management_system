use diesel::{insert_into, prelude::*, result::Error};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

use crate::schema::payment;

static USER_SUBMITTED_PAYMENT_COMMENT: &str = "User submitted payment request";

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

impl Payment {
    pub fn create_payment(
        conn: &mut PgConnection,
        ownership_id: i32,
        date: chrono::NaiveDate,
        amount: i32,
        state: PaymentState,
        comment: &str,
        reference: &str,
        last_changed_by_user: i32,
        proof_of_payment_path: &str,
    ) -> Result<Payment, Error> {
        insert_into(payment::table)
            .values((
                payment::ownership_id.eq(ownership_id),
                payment::date.eq(date),
                payment::amount.eq(amount),
                payment::state.eq(state),
                payment::comment.eq(comment),
                payment::reference.eq(reference),
                payment::last_changed_by_user.eq(last_changed_by_user),
                payment::proof_of_payment_path.eq(proof_of_payment_path),
            ))
            .get_result(conn)
    }

    pub fn create_user_payment(
        conn: &mut PgConnection,
        ownership_id: i32,
        date: chrono::NaiveDate,
        amount: i32,
        reference: &str,
        last_changed_by_user: i32,
        proof_of_payment_path: &str,
    ) -> Result<Payment, Error> {
        Payment::create_payment(
            conn,
            ownership_id,
            date,
            amount,
            PaymentState::Pending,
            USER_SUBMITTED_PAYMENT_COMMENT,
            reference,
            last_changed_by_user,
            proof_of_payment_path,
        )
    }
}
