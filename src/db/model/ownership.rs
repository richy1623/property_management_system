use diesel::{insert_into, prelude::*};
use diesel_derive_enum::DbEnum;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::db::schema::ownership;

use crate::error::Error;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    DbEnum,
    Serialize,
    Deserialize,
    TryFromPrimitive,
    IntoPrimitive,
    ToSchema,
)]
#[db_enum(
    existing_type_path = "crate::db::schema::sql_types::OwnershipState",
    value_style = "SCREAMING_SNAKE_CASE"
)]
#[repr(i32)]
#[derive(strum::Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum OwnershipState {
    PendingContract = 0,
    ActivePayment = 1,
    Paid = 2,
    Defaulted = 3,
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
            .map_err(Into::into)
    }

    pub fn get_ownership(conn: &mut PgConnection, ownership_id: i32) -> Result<Ownership, Error> {
        ownership::table
            .find(ownership_id)
            .select(Ownership::as_select())
            .first(conn)
            .map_err(Into::into)
    }

    pub fn get_ownership_for_user_and_plot(
        conn: &mut PgConnection,
        user_id: i32,
        plot_id: i32,
    ) -> Result<Ownership, Error> {
        ownership::table
            .filter(ownership::user_id.eq(user_id))
            .filter(ownership::plot_id.eq(plot_id))
            .select(Ownership::as_select())
            .first(conn)
            .map_err(Into::into)
    }

    pub fn get_all_ownerships_for_user_id(
        conn: &mut PgConnection,
        user_id: i32,
    ) -> Result<Vec<Ownership>, Error> {
        ownership::table
            .filter(ownership::user_id.eq(user_id))
            .select(Ownership::as_select())
            .get_results(conn)
            .map_err(Into::into)
    }

    pub fn update_ownership(
        conn: &mut PgConnection,
        target_ownership_id: i32,
        new_state: Option<OwnershipState>,
        new_total_sale_value: Option<i32>,
        new_monthly_repayment_amount: Option<i32>,
        _updated_by_user_id: i32,
    ) -> Result<Ownership, Error> {
        // TODO audit log
        let existing_ownership = Ownership::get_ownership(conn, target_ownership_id)?;
        diesel::update(ownership::table.filter(ownership::ownership_id.eq(target_ownership_id)))
            .set((
                ownership::state.eq(new_state.unwrap_or(existing_ownership.state)),
                ownership::total_sale_value
                    .eq(new_total_sale_value.unwrap_or(existing_ownership.total_sale_value)),
                ownership::monthly_repayment_amount.eq(new_monthly_repayment_amount
                    .unwrap_or(existing_ownership.monthly_repayment_amount)),
            ))
            .get_result(conn)
            .map_err(Into::into)
    }

    pub fn get_ownerships_filtered(
        conn: &mut PgConnection,
        user_id: Option<i32>,
        plot_id: Option<i32>,
        state: Option<OwnershipState>,
    ) -> Result<Vec<Ownership>, Error> {
        let mut query = ownership::table.into_boxed();

        if let Some(user_id) = user_id {
            query = query.filter(ownership::user_id.eq(user_id));
        }

        if let Some(plot_id) = plot_id {
            query = query.filter(ownership::plot_id.eq(plot_id));
        }

        if let Some(ownership_state) = state {
            query = query.filter(ownership::state.eq(ownership_state));
        }

        query
            .select(Ownership::as_select())
            .get_results(conn)
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use diesel::{
        PgConnection,
        r2d2::{ConnectionManager, Pool},
    };

    use rstest::{fixture, rstest};

    use super::*;

    #[fixture]
    #[once]
    pub fn pool() -> Pool<ConnectionManager<PgConnection>> {
        crate::tests::get_test_db_connection_pool("test_ownership")
    }

    #[rstest]
    pub fn test_create_ownership(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let user = crate::db::model::tests::create_test_user(&mut conn);

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
    pub fn test_get_ownership_for_user_and_plot(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let estate = crate::db::model::tests::create_test_estate(&mut conn);

        let plot_1 = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let plot_2 = crate::db::model::tests::create_test_plot(&mut conn, &estate);

        let user = crate::db::model::tests::create_test_user(&mut conn);

        let ownership_1 = crate::db::model::tests::create_test_ownership(&mut conn, &plot_1, &user);
        let ownership_2 = crate::db::model::tests::create_test_ownership(&mut conn, &plot_2, &user);

        assert_eq!(
            Ownership::get_ownership_for_user_and_plot(&mut conn, user.user_id, plot_1.plot_id)
                .unwrap(),
            ownership_1
        );

        assert_eq!(
            Ownership::get_ownership_for_user_and_plot(&mut conn, user.user_id, plot_2.plot_id)
                .unwrap(),
            ownership_2
        );

        assert_eq!(
            Ownership::get_ownership_for_user_and_plot(&mut conn, -1, -1).unwrap_err(),
            super::Error::DatabaseError(diesel::result::Error::NotFound)
        );
    }

    #[rstest]
    pub fn test_get_all_ownerships_for_user_id(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let estate = crate::db::model::tests::create_test_estate(&mut conn);

        let plot_1 = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let plot_2 = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let plot_other = crate::db::model::tests::create_test_plot(&mut conn, &estate);

        let user = crate::db::model::tests::create_test_user(&mut conn);
        let user_other = crate::db::model::tests::create_test_user(&mut conn);

        let ownership_1 = crate::db::model::tests::create_test_ownership(&mut conn, &plot_1, &user);
        let ownership_2 = crate::db::model::tests::create_test_ownership(&mut conn, &plot_2, &user);
        let ownership_other =
            crate::db::model::tests::create_test_ownership(&mut conn, &plot_other, &user_other);

        assert_eq!(
            Ownership::get_all_ownerships_for_user_id(&mut conn, user.user_id).unwrap(),
            vec![ownership_1, ownership_2]
        );
        assert_eq!(
            Ownership::get_all_ownerships_for_user_id(&mut conn, user_other.user_id).unwrap(),
            vec![ownership_other]
        );
    }

    #[rstest]
    pub fn test_update_ownership_state(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let estate = crate::db::model::tests::create_test_estate(&mut conn);

        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);

        let user = crate::db::model::tests::create_test_user(&mut conn);

        let ownership = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);

        let update_ownership = Ownership::update_ownership(
            &mut conn,
            ownership.ownership_id,
            Some(OwnershipState::Paid),
            Some(10),
            Some(1),
            user.user_id,
        )
        .unwrap();

        assert_eq!(update_ownership.user_id, user.user_id);
        assert_eq!(update_ownership.plot_id, plot.plot_id);
        assert_eq!(update_ownership.state, OwnershipState::Paid);
        assert_eq!(update_ownership.total_sale_value, 10);
        assert_eq!(update_ownership.monthly_repayment_amount, 1);

        let ownership_in_db = Ownership::get_ownership(&mut conn, ownership.ownership_id).unwrap();

        assert_eq!(update_ownership, ownership_in_db);
    }

    #[rstest]
    pub fn test_get_ownerships_filtered(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let estate = crate::db::model::tests::create_test_estate(&mut conn);

        let plot_1 = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let plot_2 = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let plot_3 = crate::db::model::tests::create_test_plot(&mut conn, &estate);

        let user_1 = crate::db::model::tests::create_test_user(&mut conn);
        let user_2 = crate::db::model::tests::create_test_user(&mut conn);

        let ownership_1 =
            crate::db::model::tests::create_test_ownership(&mut conn, &plot_1, &user_1);
        let ownership_2 =
            crate::db::model::tests::create_test_ownership(&mut conn, &plot_2, &user_1);
        let ownership_3 =
            crate::db::model::tests::create_test_ownership(&mut conn, &plot_3, &user_2);
        let ownership_4 =
            crate::db::model::tests::create_test_ownership(&mut conn, &plot_1, &user_2);

        let ownership_3 = Ownership::update_ownership(
            &mut conn,
            ownership_3.ownership_id,
            Some(OwnershipState::Paid),
            None,
            None,
            user_2.user_id,
        )
        .unwrap();

        assert_eq!(
            Ownership::get_ownerships_filtered(&mut conn, Some(user_1.user_id), None, None)
                .unwrap(),
            vec![ownership_1.clone(), ownership_2.clone()]
        );
        assert_eq!(
            Ownership::get_ownerships_filtered(
                &mut conn,
                Some(user_1.user_id),
                Some(plot_1.plot_id),
                None
            )
            .unwrap(),
            vec![ownership_1.clone()]
        );
        assert_eq!(
            Ownership::get_ownerships_filtered(&mut conn, None, Some(plot_1.plot_id), None)
                .unwrap(),
            vec![ownership_1.clone(), ownership_4.clone()]
        );
        assert!(
            Ownership::get_ownerships_filtered(&mut conn, None, None, Some(OwnershipState::Paid))
                .unwrap()
                .contains(&ownership_3)
        );
        let all_ownerships =
            Ownership::get_ownerships_filtered(&mut conn, None, None, None).unwrap();
        assert!(all_ownerships.contains(&ownership_1));
        assert!(all_ownerships.contains(&ownership_2));
        assert!(all_ownerships.contains(&ownership_3));
        assert!(all_ownerships.contains(&ownership_4));
    }
}
