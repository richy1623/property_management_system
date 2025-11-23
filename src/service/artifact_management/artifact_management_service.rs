use crate::jwt::Jwt;
use crate::secure_storage::SecureStorage;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};

pub struct ArtifactManagementService {
    pub database_connection_pool: Pool<ConnectionManager<PgConnection>>,
    pub jwt: Jwt,
    pub secure_storage: SecureStorage,
}

impl ArtifactManagementService {
    pub fn new(
        database_connection_url: &str,
        jwt_secret_key: &str,
        secure_storage: SecureStorage,
    ) -> Self {
        ArtifactManagementService {
            database_connection_pool: crate::get_connection_pool(database_connection_url),
            jwt: Jwt::new(jwt_secret_key),
            secure_storage,
        }
    }
}
