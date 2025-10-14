use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::schema::{ownership, plot};

use super::ownership::OwnershipState;

use crate::error::Error;

#[derive(
    Queryable, Selectable, Insertable, Debug, PartialEq, Eq, Clone, Serialize, Deserialize,
)]
#[diesel(table_name = plot)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Plot {
    pub plot_id: i32,
    pub estate_id: i32,
    pub plot_number: String,
    pub phase: String,
    pub length: i32,
    pub width: i32,
}

impl Plot {
    pub fn create_plot(
        conn: &mut PgConnection,
        estate_id: i32,
        plot_number: &str,
        phase: &str,
        length: i32,
        width: i32,
    ) -> Result<Plot, Error> {
        diesel::insert_into(plot::table)
            .values((
                plot::estate_id.eq(estate_id),
                plot::plot_number.eq(plot_number),
                plot::phase.eq(phase),
                plot::length.eq(length),
                plot::width.eq(width),
            ))
            .get_result(conn)
            .map_err(Into::into)
    }

    pub fn get_plot(conn: &mut PgConnection, plot_id: i32) -> Result<Plot, Error> {
        plot::table
            .find(plot_id)
            .select(Plot::as_select())
            .first(conn)
            .map_err(Into::into)
    }

    pub fn get_plot_with_plot_number_and_estate_id(
        conn: &mut PgConnection,
        plot_number: &str,
        estate_id: i32,
    ) -> Result<Plot, Error> {
        plot::table
            .filter(plot::plot_number.eq(plot_number))
            .filter(plot::estate_id.eq(estate_id))
            .select(Plot::as_select())
            .first(conn)
            .map_err(Into::into)
    }

    pub fn get_plots_filtered(
        conn: &mut PgConnection,
        plot_number: Option<&str>,
        estate_id: Option<i32>,
        phase: Option<&str>,
        allowed_ownership_states: Vec<OwnershipState>,
    ) -> Result<Vec<Plot>, Error> {
        let mut plot_query = plot::table.into_boxed();

        if let Some(plot_number) = plot_number {
            plot_query = plot_query.filter(plot::plot_number.eq(plot_number));
        }
        if let Some(estate_id) = estate_id {
            plot_query = plot_query.filter(plot::estate_id.eq(estate_id));
        }
        if let Some(phase) = phase {
            plot_query = plot_query.filter(plot::phase.eq(phase));
        }

        if !allowed_ownership_states.is_empty() {
            return plot_query
                .inner_join(ownership::table)
                .filter(ownership::state.eq_any(allowed_ownership_states))
                .select(Plot::as_select())
                .get_results(conn)
                .map_err(Into::into);
        }

        plot_query
            .select(Plot::as_select())
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

    use crate::db::model::{
        estate::Estate,
        ownership::{Ownership, OwnershipState},
        plot::Plot,
    };

    #[fixture]
    #[once]
    pub fn pool() -> Pool<ConnectionManager<PgConnection>> {
        crate::tests::get_test_db_connection_pool("test_plots")
    }

    #[rstest]
    pub fn test_plot(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let estate =
            Estate::create_estate(&mut conn, "estate_1", "location", "district", "landmark")
                .unwrap();

        let plot = Plot::create_plot(&mut conn, estate.estate_id, "P1", "phase 1", 30, 40).unwrap();

        assert_eq!(plot.estate_id, estate.estate_id);
        assert_eq!(plot.plot_number, "P1");
        assert_eq!(plot.phase, "phase 1");
        assert_eq!(plot.length, 30);
        assert_eq!(plot.width, 40);

        assert_eq!(plot, Plot::get_plot(&mut conn, plot.plot_id).unwrap());
    }

    #[rstest]
    pub fn test_get_plot_with_plot_number_and_estate_id(
        pool: &Pool<ConnectionManager<PgConnection>>,
    ) {
        let mut conn = pool.clone().get().unwrap();

        let estate_1 = crate::db::model::tests::create_test_estate(&mut conn);
        let estate_2 = crate::db::model::tests::create_test_estate(&mut conn);

        let plot_1 =
            Plot::create_plot(&mut conn, estate_1.estate_id, "P1", "phase 1", 30, 40).unwrap();
        let plot_2 =
            Plot::create_plot(&mut conn, estate_1.estate_id, "P2", "phase 1", 30, 40).unwrap();
        let plot_3 =
            Plot::create_plot(&mut conn, estate_2.estate_id, "P1", "phase 1", 30, 40).unwrap();

        assert_eq!(
            plot_1,
            Plot::get_plot_with_plot_number_and_estate_id(
                &mut conn,
                &plot_1.plot_number,
                estate_1.estate_id
            )
            .unwrap()
        );
        assert_eq!(
            plot_2,
            Plot::get_plot_with_plot_number_and_estate_id(
                &mut conn,
                &plot_2.plot_number,
                estate_1.estate_id
            )
            .unwrap()
        );
        assert_eq!(
            plot_3,
            Plot::get_plot_with_plot_number_and_estate_id(
                &mut conn,
                &plot_3.plot_number,
                estate_2.estate_id
            )
            .unwrap()
        );
    }

    #[rstest]
    pub fn test_get_plots_filtered(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        // Setup estates
        let estate_1 = crate::db::model::tests::create_test_estate(&mut conn);
        let estate_2 = crate::db::model::tests::create_test_estate(&mut conn);

        // Setup plots
        let plot_1 =
            Plot::create_plot(&mut conn, estate_1.estate_id, "P100", "Alpha", 30, 40).unwrap();
        let plot_2 =
            Plot::create_plot(&mut conn, estate_1.estate_id, "P200", "Beta", 30, 40).unwrap();
        let plot_3 =
            Plot::create_plot(&mut conn, estate_2.estate_id, "P300", "Gamma", 30, 40).unwrap();

        // Create test user
        let user = crate::db::model::tests::create_test_user(&mut conn);

        Ownership::create_ownership(
            &mut conn,
            user.user_id,
            plot_1.plot_id,
            OwnershipState::ActivePayment,
            0,
            0,
            None,
        )
        .unwrap();
        Ownership::create_ownership(
            &mut conn,
            user.user_id,
            plot_2.plot_id,
            OwnershipState::ActivePayment,
            0,
            0,
            None,
        )
        .unwrap();
        Ownership::create_ownership(
            &mut conn,
            user.user_id,
            plot_3.plot_id,
            OwnershipState::PendingContract,
            0,
            0,
            None,
        )
        .unwrap();

        // --- Filter by plot_number ---
        let result = Plot::get_plots_filtered(&mut conn, Some("P100"), None, None, vec![]).unwrap();

        assert_eq!(result, vec![plot_1.clone()]);

        // --- Filter by estate_id ---
        let result =
            Plot::get_plots_filtered(&mut conn, None, Some(estate_2.estate_id), None, vec![])
                .unwrap();
        assert_eq!(result, vec![plot_3.clone()]);

        // --- Filter by phase ---
        let result = Plot::get_plots_filtered(&mut conn, None, None, Some("Beta"), vec![]).unwrap();
        assert_eq!(result, vec![plot_2.clone()]);

        // --- Filter by ownership state ---
        let result = Plot::get_plots_filtered(
            &mut conn,
            None,
            None,
            None,
            vec![OwnershipState::ActivePayment],
        )
        .unwrap();
        assert_eq!(result, vec![plot_1.clone(), plot_2.clone()]);
        let result = Plot::get_plots_filtered(
            &mut conn,
            None,
            None,
            None,
            vec![
                OwnershipState::ActivePayment,
                OwnershipState::PendingContract,
            ],
        )
        .unwrap();
        assert_eq!(result, vec![plot_1.clone(), plot_2.clone(), plot_3.clone()]);
    }
}
