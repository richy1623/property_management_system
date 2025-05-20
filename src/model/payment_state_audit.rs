use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{model::payment::PaymentState, schema::payment_state_audit};

use super::Error;

#[derive(
    Queryable, Selectable, Insertable, Debug, PartialEq, Eq, Clone, Serialize, Deserialize,
)]
#[diesel(table_name = payment_state_audit)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PaymentStateAudit {
    pub audit_id: i32,
    pub payment_id: i32,
    pub previous_state: PaymentState,
    pub new_state: PaymentState,
    pub change_timestamp: chrono::DateTime<Utc>,
    pub changed_by_user: i32,
    pub comment: String,
}

impl PaymentStateAudit {
    pub fn get_all_payment_state_audits_for_payment_id(
        conn: &mut PgConnection,
        payment_id: i32,
    ) -> Result<Vec<PaymentStateAudit>, Error> {
        payment_state_audit::table
            .filter(payment_state_audit::payment_id.eq(payment_id))
            .select(PaymentStateAudit::as_select())
            .get_results(conn)
            .map_err(Into::into)
    }
}
