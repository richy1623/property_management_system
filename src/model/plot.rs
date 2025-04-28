use diesel::{prelude::*, result::Error};
use serde::{Deserialize, Serialize};

use crate::schema::plot;

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
    }

    pub fn get_plot(conn: &mut PgConnection, plot_id: i32) -> Result<Plot, Error> {
        plot::table
            .find(plot_id)
            .select(Plot::as_select())
            .first(conn)
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
    }
}

#[cfg(test)]
mod tests {
    use diesel::{
        PgConnection,
        r2d2::{ConnectionManager, Pool},
    };

    use rstest::{fixture, rstest};

    use crate::model::{estate::Estate, plot::Plot};

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

        let estate_1 = crate::model::tests::create_test_estate(&mut conn);
        let estate_2 = crate::model::tests::create_test_estate(&mut conn);

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
}
