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
        plot_number: String,
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
}
