use diesel::{prelude::*, result::Error};
use serde::{Deserialize, Serialize};

use crate::schema::estate;

#[derive(
    Queryable, Selectable, Insertable, Debug, PartialEq, Eq, Clone, Serialize, Deserialize,
)]
#[diesel(table_name = estate)]
#[diesel(check_for_backend(diesel::pg::Pg))]
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

    pub fn create_estate(
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

#[cfg(test)]
mod tests {
    use diesel::{
        PgConnection,
        r2d2::{ConnectionManager, Pool},
    };

    use rstest::{fixture, rstest};

    use crate::model::estate::Estate;

    #[fixture]
    #[once]
    pub fn pool() -> Pool<ConnectionManager<PgConnection>> {
        crate::tests::get_test_db_connection_pool("test_estate")
    }

    #[rstest]
    pub fn test_create_estate(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let estate =
            Estate::create_estate(&mut conn, "estate_1", "location", "district", "landmark")
                .unwrap();

        assert_eq!(estate.name, "estate_1");
        assert_eq!(estate.location, "location");
        assert_eq!(estate.district, "district");
        assert_eq!(estate.landmark, "landmark");

        assert_eq!(
            estate,
            Estate::get_estate(&mut conn, estate.estate_id).unwrap()
        );
    }

    #[rstest]
    pub fn test_get_all_estates(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let estate_1 =
            Estate::create_estate(&mut conn, "estate_a1", "location", "district", "landmark")
                .unwrap();
        let estate_2 =
            Estate::create_estate(&mut conn, "estate_a2", "location", "district", "landmark")
                .unwrap();
        let estate_3 =
            Estate::create_estate(&mut conn, "estate_a3", "location", "district", "landmark")
                .unwrap();

        let all_estates = Estate::get_all_estates(&mut conn).unwrap();
        assert!(all_estates.contains(&estate_1));
        assert!(all_estates.contains(&estate_2));
        assert!(all_estates.contains(&estate_3));
    }
}
