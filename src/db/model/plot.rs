use std::collections::HashMap;

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

    pub fn get_ownership_states_for_plots(
        conn: &mut PgConnection,
        plots: Vec<Plot>,
    ) -> Result<Vec<(Plot, Option<OwnershipState>)>, Error> {
        let mut plot_map: HashMap<i32, Plot> = plots.into_iter().map(|p| (p.plot_id, p)).collect();
        let plot_ids = plot_map.keys();
        plot::table
            .left_join(ownership::table)
            .filter(plot::plot_id.eq_any(plot_ids))
            .select((plot::plot_id, ownership::state.nullable()))
            .distinct_on(plot::plot_id)
            .order_by(plot::plot_id.asc())
            .then_order_by(ownership::contract_start_date.desc())
            .load::<(i32, Option<OwnershipState>)>(conn)
            .map_err(Into::into)
            .map(|plots_and_ownerships| {
                plots_and_ownerships
                    .into_iter()
                    .map(|(plot_id, ownership_state)| {
                        (plot_map.remove(&plot_id).unwrap(), ownership_state)
                    })
                    .collect::<Vec<(Plot, Option<OwnershipState>)>>()
            })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::NaiveDate;
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
            Plot::create_plot(&mut conn, estate_1.estate_id, "PF100", "AF", 30, 40).unwrap();
        let plot_2 =
            Plot::create_plot(&mut conn, estate_1.estate_id, "PF200", "BF", 30, 40).unwrap();
        let plot_3 =
            Plot::create_plot(&mut conn, estate_2.estate_id, "PF300", "GF", 30, 40).unwrap();

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
            OwnershipState::Paid,
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
        let result =
            Plot::get_plots_filtered(&mut conn, Some("PF100"), None, None, vec![]).unwrap();

        assert_eq!(result, vec![plot_1.clone()]);

        // --- Filter by estate_id ---
        let result =
            Plot::get_plots_filtered(&mut conn, None, Some(estate_2.estate_id), None, vec![])
                .unwrap();
        assert_eq!(result, vec![plot_3.clone()]);

        // --- Filter by phase ---
        let result = Plot::get_plots_filtered(&mut conn, None, None, Some("BF"), vec![]).unwrap();
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
        assert!(result.contains(&plot_1));
        assert!(!result.contains(&plot_2));
        assert!(!result.contains(&plot_3));
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
        assert!(result.contains(&plot_1));
        assert!(!result.contains(&plot_2));
        assert!(result.contains(&plot_3));
    }

    #[rstest]
    pub fn test_get_ownership_states_for_plots(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        // --- Setup: Estates ---
        let estate_1 = crate::db::model::tests::create_test_estate(&mut conn);

        // --- Setup: Plots ---
        let plot_1 = Plot::create_plot(&mut conn, estate_1.estate_id, "P100", "x", 30, 40).unwrap();
        let plot_2 = Plot::create_plot(&mut conn, estate_1.estate_id, "P200", "x", 30, 40).unwrap();
        let plot_3 = Plot::create_plot(&mut conn, estate_1.estate_id, "P300", "x", 30, 40).unwrap();

        // --- Setup: User ---
        let user = crate::db::model::tests::create_test_user(&mut conn);

        // --- Ownership: plot_1 has two records, latest should be picked ---
        // --- Ownership for plot 1 (two records, pick the latest) ---
        Ownership::create_ownership(
            &mut conn,
            user.user_id,
            plot_1.plot_id,
            OwnershipState::PendingContract,
            0,
            0,
            Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        )
        .unwrap();

        Ownership::create_ownership(
            &mut conn,
            user.user_id,
            plot_1.plot_id,
            OwnershipState::ActivePayment,
            0,
            0,
            Some(NaiveDate::from_ymd_opt(2024, 10, 1).unwrap()),
        )
        .unwrap();

        // --- Ownership for plot 2 (single ownership) ---
        Ownership::create_ownership(
            &mut conn,
            user.user_id,
            plot_2.plot_id,
            OwnershipState::ActivePayment,
            0,
            0,
            Some(NaiveDate::from_ymd_opt(2024, 5, 10).unwrap()),
        )
        .unwrap();
        // --- Input vector (order should not matter) ---
        let input_plots = vec![plot_1.clone(), plot_2.clone(), plot_3.clone()];

        let result = Plot::get_ownership_states_for_plots(&mut conn, input_plots).unwrap();

        // Convert result to a map for easier checking
        let mut map: HashMap<i32, Option<OwnershipState>> =
            result.into_iter().map(|(p, s)| (p.plot_id, s)).collect();

        // --- plot_1: Should pick the latest ownership by date DESC ---
        assert_eq!(
            map.remove(&plot_1.plot_id),
            Some(Some(OwnershipState::ActivePayment))
        );

        // --- plot_2: Only one ownership, so that one must appear ---
        assert_eq!(
            map.remove(&plot_2.plot_id),
            Some(Some(OwnershipState::ActivePayment))
        );

        // --- plot_3: No Ownership ---
        assert_eq!(map.remove(&plot_3.plot_id), Some(None));

        // No extra entries
        assert!(map.is_empty());
    }
}
