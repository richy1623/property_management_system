pub mod model;
mod schema;

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel_migrations::EmbeddedMigrations;
use dotenvy::dotenv;

pub const MIGRATIONS: EmbeddedMigrations = diesel_migrations::embed_migrations!();

pub fn get_connection_pool(database_url: &str) -> Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();
    // let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    // Refer to the `r2d2` documentation for more methods to use
    // when building a connection pool
    Pool::builder()
        .test_on_check_out(true)
        .min_idle(Some(1))
        .build(manager)
        .expect("Could not build connection pool")
}

#[cfg(test)]
mod tests {
    use diesel::{
        Connection, PgConnection, RunQueryDsl,
        r2d2::{ConnectionManager, Pool},
    };
    use diesel_migrations::MigrationHarness;

    use crate::MIGRATIONS;

    pub fn get_test_db_connection_pool(db_name: &str) -> Pool<ConnectionManager<PgConnection>> {
        let admin_url = "postgres://username:password@localhost:5430/postgres"; // No specific DB yet
        let mut conn =
            PgConnection::establish(admin_url).expect("Error connecting to Postgres server");

        // First, drop the database if it exists
        let drop_db_sql = format!("DROP DATABASE IF EXISTS {}", db_name);
        diesel::sql_query(drop_db_sql)
            .execute(&mut conn)
            .expect("Failed to drop existing database");

        // Run the SQL to create it
        let create_db_sql = format!("CREATE DATABASE {}", db_name);
        diesel::sql_query(create_db_sql)
            .execute(&mut conn)
            .expect("Failed to create database");

        let pool = crate::get_connection_pool(&format!(
            "{}{}",
            "postgres://username:password@localhost:5430/", db_name
        ));

        pool.clone()
            .get()
            .unwrap()
            .run_pending_migrations(MIGRATIONS)
            .unwrap();

        pool
    }
}
