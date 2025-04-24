use diesel::{prelude::*, result::Error};
use serde::{Deserialize, Serialize};

use crate::schema::estate;

#[derive(
    Queryable, Selectable, Insertable, Debug, PartialEq, Eq, Clone, Serialize, Deserialize,
)]
#[diesel(table_name = estate)]
pub struct Estate {
    estate_id: i32,
    name: String,
    location: String,
    district: String,
    landmark: String,
}

impl Estate {
    pub fn get_all_estates(conn: &mut PgConnection) -> Result<Vec<Estate>, Error> {
        estate::table.load::<Estate>(conn)
    }

    pub fn add_estate(
        conn: &mut PgConnection,
        name: &str,
        location: &str,
        district: &str,
        landmark: &str,
    ) -> Result<Estate, Error> {
        diesel::insert_into(estate::table)
            .values((
                estate::name.eq(name),
                estate::location.eq(location),
                estate::district.eq(district),
                estate::landmark.eq(landmark),
            ))
            .get_result(conn)
    }

    pub fn get_estate(conn: &mut PgConnection, estate_id: i32) -> Result<Estate, Error> {
        estate::table
            .find(estate_id)
            .select(Estate::as_select())
            .first(conn)
    }
}
