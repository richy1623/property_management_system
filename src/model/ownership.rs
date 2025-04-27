use diesel::{insert_into, prelude::*};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

use crate::schema::ownership;

#[derive(Debug, Clone, PartialEq, Eq, DbEnum, Serialize, Deserialize)]
#[db_enum(existing_type_path = "crate::schema::sql_types::OwnershipState")]
pub enum OwnershipState {
    PendingContract,
    ActivePayment,
    Paid,
    Defaulted,
}

#[derive(
    Queryable, Selectable, Insertable, Debug, PartialEq, Eq, Clone, Serialize, Deserialize,
)]
#[diesel(table_name = ownership)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Ownership {
    pub ownership_id: i32,
    pub person_id: i32,
    pub plot_id: i32,
    pub state: OwnershipState,
    pub total_sale_value: i32,
    pub monthly_repayment_amount: i32,
}

impl Ownership {
    pub fn create_ownership(
        conn: &mut PgConnection,
        person_id: i32,
        plot_id: i32,
        state: OwnershipState,
        total_sale_value: i32,
        monthly_repayment_amount: i32,
    ) -> QueryResult<Ownership> {
        insert_into(ownership::table)
            .values((
                ownership::person_id.eq(person_id),
                ownership::plot_id.eq(plot_id),
                ownership::state.eq(state),
                ownership::total_sale_value.eq(total_sale_value),
                ownership::monthly_repayment_amount.eq(monthly_repayment_amount),
            ))
            .get_result(conn)
    }
}
