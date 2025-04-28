use diesel::{insert_into, prelude::*, result::Error};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

use crate::schema::ownership;

#[derive(Debug, Clone, PartialEq, Eq, DbEnum, Serialize, Deserialize)]
#[db_enum(
    existing_type_path = "crate::schema::sql_types::OwnershipState",
    value_style = "SCREAMING_SNAKE_CASE"
)]
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
    pub user_id: i32,
    pub plot_id: i32,
    pub state: OwnershipState,
    pub total_sale_value: i32,
    pub monthly_repayment_amount: i32,
}

impl Ownership {
    pub fn create_ownership(
        conn: &mut PgConnection,
        user_id: i32,
        plot_id: i32,
        state: OwnershipState,
        total_sale_value: i32,
        monthly_repayment_amount: i32,
    ) -> Result<Ownership, Error> {
        insert_into(ownership::table)
            .values((
                ownership::user_id.eq(user_id),
                ownership::plot_id.eq(plot_id),
                ownership::state.eq(state),
                ownership::total_sale_value.eq(total_sale_value),
                ownership::monthly_repayment_amount.eq(monthly_repayment_amount),
            ))
            .get_result(conn)
    }

    pub fn get_ownership(conn: &mut PgConnection, ownership_id: i32) -> Result<Ownership, Error> {
        ownership::table
            .find(ownership_id)
            .select(Ownership::as_select())
            .first(conn)
    }

    pub fn get_all_ownerships_for_user_id(
        conn: &mut PgConnection,
        user_id: i32,
    ) -> Result<Vec<Ownership>, Error> {
        ownership::table
            .filter(ownership::user_id.eq(user_id))
            .select(Ownership::as_select())
            .get_results(conn)
    }
}

#[cfg(test)]
mod tests {
    use diesel::{
        PgConnection,
        r2d2::{ConnectionManager, Pool},
    };

    use rstest::{fixture, rstest};

    use crate::model::{
        ownership::{Ownership, OwnershipState},
        tests::create_test_ownership,
    };

    #[fixture]
    #[once]
    pub fn pool() -> Pool<ConnectionManager<PgConnection>> {
        crate::tests::get_test_db_connection_pool("test_ownership")
    }

    #[rstest]
    pub fn test_create_ownership(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let estate = crate::model::tests::create_test_estate(&mut conn);
        let plot = crate::model::tests::create_test_plot(&mut conn, &estate);
        let user = crate::model::tests::create_test_user(&mut conn);

        let ownership = Ownership::create_ownership(
            &mut conn,
            user.user_id,
            plot.plot_id,
            OwnershipState::PendingContract,
            1000,
            10,
        )
        .unwrap();

        assert_eq!(ownership.user_id, user.user_id);
        assert_eq!(ownership.plot_id, plot.plot_id);
        assert_eq!(ownership.state, OwnershipState::PendingContract);
        assert_eq!(ownership.total_sale_value, 1000);
        assert_eq!(ownership.monthly_repayment_amount, 10);

        assert_eq!(
            ownership,
            Ownership::get_ownership(&mut conn, ownership.ownership_id).unwrap()
        );
    }

    #[rstest]
    pub fn test_get_all_ownerships_for_user_id(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let estate = crate::model::tests::create_test_estate(&mut conn);

        let plot_1 = crate::model::tests::create_test_plot(&mut conn, &estate);
        let plot_2 = crate::model::tests::create_test_plot(&mut conn, &estate);
        let plot_other = crate::model::tests::create_test_plot(&mut conn, &estate);

        let user = crate::model::tests::create_test_user(&mut conn);
        let user_other = crate::model::tests::create_test_user(&mut conn);

        let ownership_1 = create_test_ownership(&mut conn, &plot_1, &user);
        let ownership_2 = create_test_ownership(&mut conn, &plot_2, &user);
        let ownership_other = create_test_ownership(&mut conn, &plot_other, &user_other);

        assert_eq!(
            Ownership::get_all_ownerships_for_user_id(&mut conn, user.user_id).unwrap(),
            vec![ownership_1, ownership_2]
        );
        assert_eq!(
            Ownership::get_all_ownerships_for_user_id(&mut conn, user_other.user_id).unwrap(),
            vec![ownership_other]
        );
    }
}
