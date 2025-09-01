use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};

pub mod estate_handler;
pub mod ownership_handler;
pub mod payment_handler;
pub mod plot_handler;
pub mod user_handler;

pub fn get_database_connection(
    database_connection_pool: Pool<ConnectionManager<PgConnection>>,
) -> Result<PooledConnection<ConnectionManager<PgConnection>>, actix_web::Error> {
    database_connection_pool.get().map_err(|error| {
        tracing::error!(
            "Internal server error. Failed to establish connection to database: {:?}",
            error
        );
        actix_web::error::ErrorInternalServerError("Failed to establish connection to database")
    })
}
