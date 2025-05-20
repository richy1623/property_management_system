use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use diesel_migrations::MigrationHarness;
use tonic::{Request, Response, Status};

use super::estate_management_proto::{
    Empty, Estate, EstateCreationRequest, EstateList, estate_management_server::EstateManagement,
};
use crate::model::estate::Estate as EstateModel;
use crate::{MIGRATIONS, jwt::Jwt, model::user_permission::PermissionType};

pub struct EstateManagementService {
    database_connection_pool: Pool<ConnectionManager<PgConnection>>,
    jwt: Jwt,
}

impl EstateManagementService {
    pub fn new(database_connection_url: &str) -> Self {
        let database_connection_pool = crate::get_connection_pool(database_connection_url);
        database_connection_pool
            .clone()
            .get()
            .expect("Failed getting db pool")
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed running migrations");

        EstateManagementService {
            database_connection_pool: database_connection_pool,
            jwt: Jwt::new("a-string-secret-at-least-256-bits-long"),
        }
    }
}

#[tonic::async_trait]
impl EstateManagement for EstateManagementService {
    async fn create_estate(
        &self,
        request: Request<EstateCreationRequest>,
    ) -> Result<Response<Estate>, Status> {
        let mut conn = match self.database_connection_pool.clone().get() {
            Ok(conn) => conn,
            Err(e) => return Err(Status::internal(e.to_string())),
        };

        let request = request.into_inner();
        // let request_estate = request.estate.unwrap();

        // Validate JWT token
        let jwt_token = self.jwt.decode_jwt_token(&request.jwt_token)?;

        // Check for required claim
        if !jwt_token
            .claims
            .permissions
            .contains(&PermissionType::UpdateEstates.to_string())
        {
            return Err(Status::permission_denied(
                "Insufficient permissions to create estate",
            ));
        }

        let estate = EstateModel::create_estate(
            &mut conn,
            &request.name,
            &request.location,
            &request.district,
            &request.landmark,
        )?;

        let reply = super::estate_management_proto::Estate {
            estate_id: estate.estate_id,
            name: estate.name.clone(),
            location: estate.location.clone(),
            district: estate.district.clone(),
            landmark: estate.landmark.clone(),
        };

        Ok(Response::new(reply))
    }

    async fn get_all_estates(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<EstateList>, Status> {
        let mut conn = match self.database_connection_pool.clone().get() {
            Ok(conn) => conn,
            Err(e) => return Err(Status::internal(e.to_string())),
        };

        let estates = EstateModel::get_all_estates(&mut conn)?;

        let estate_list = estates
            .iter()
            .map(|estate| super::estate_management_proto::Estate {
                estate_id: estate.estate_id,
                name: estate.name.clone(),
                location: estate.location.clone(),
                district: estate.district.clone(),
                landmark: estate.landmark.clone(),
            })
            .collect();

        Ok(Response::new(EstateList {
            estates: estate_list,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwt::ClaimsBuilder;

    #[tokio::test]
    async fn test_create_estate_success() {
        let pool = crate::tests::get_test_db_connection_pool(
            "estate_management_service_test_test_create_estate_success",
        );
        let mut conn = pool.clone().get().unwrap();
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");

        let service = EstateManagementService {
            database_connection_pool: pool.clone(),
            jwt,
        };

        // Create a valid request
        let request = EstateCreationRequest {
            jwt_token: service.jwt.encode_jwt_token(
                ClaimsBuilder::new("user")
                    .permission(PermissionType::UpdateEstates.to_string())
                    .build(),
            ),
            name: "Test Estate".to_string(),
            location: "Test Location".to_string(),
            district: "Test District".to_string(),
            landmark: "Test Landmark".to_string(),
        };

        let response = service.create_estate(Request::new(request)).await;

        assert!(response.is_ok());
        let estate = response.unwrap().into_inner();
        assert_eq!(estate.name, "Test Estate");
        assert_eq!(estate.location, "Test Location");
        assert_eq!(estate.district, "Test District");
        assert_eq!(estate.landmark, "Test Landmark");

        let estates: Vec<Estate> = EstateModel::get_all_estates(&mut conn)
            .unwrap()
            .iter()
            .map(|e| e.clone().into())
            .collect();
        assert_eq!(estates, vec![estate]);
    }

    #[tokio::test]
    async fn test_create_estate_permission_denied() {
        let pool = crate::tests::get_test_db_connection_pool(
            "estate_management_service_test_test_create_estate_permission_denied",
        );
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");

        let service = EstateManagementService {
            database_connection_pool: pool,
            jwt,
        };

        let token = service
            .jwt
            .encode_jwt_token(ClaimsBuilder::new("user").build());

        let request = EstateCreationRequest {
            jwt_token: token,
            name: "Test Estate".to_string(),
            location: "Test Location".to_string(),
            district: "Test District".to_string(),
            landmark: "Test Landmark".to_string(),
        };

        let result = service.create_estate(Request::new(request)).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::PermissionDenied);
    }

    #[tokio::test]
    async fn test_get_all_estates_success() {
        let pool = crate::tests::get_test_db_connection_pool(
            "estate_management_service_test_test_get_all_estates_success",
        );
        let mut conn = pool.clone().get().unwrap();
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");

        let service = EstateManagementService {
            database_connection_pool: pool,
            jwt,
        };

        let estate_1: Estate = crate::model::tests::create_test_estate(&mut conn).into();
        let estate_2: Estate = crate::model::tests::create_test_estate(&mut conn).into();

        let request = Request::new(Empty {});

        let result = service.get_all_estates(request).await.unwrap();

        assert_eq!(result.get_ref().estates, vec![estate_1, estate_2]);
    }
}
