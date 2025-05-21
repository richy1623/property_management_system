use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use diesel_migrations::MigrationHarness;
use tonic::{Request, Response, Status};

use super::estate_management_proto::{
    Empty, Estate, EstateCreationRequest, EstateList, GetPlotsRequest, Plot, PlotCreationRequest,
    PlotList, estate_management_server::EstateManagement,
};
use crate::model::plot::Plot as PlotModel;
use crate::model::{estate::Estate as EstateModel, ownership::OwnershipState};
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

    async fn create_plot(
        &self,
        request: Request<PlotCreationRequest>,
    ) -> Result<Response<Plot>, Status> {
        let mut conn = self
            .database_connection_pool
            .clone()
            .get()
            .map_err(|e| Status::internal(e.to_string()))?;

        let req = request.into_inner();

        // Validate JWT
        let jwt_token = self.jwt.decode_jwt_token(&req.jwt_token)?;
        if !jwt_token
            .claims
            .permissions
            .contains(&PermissionType::UpdateEstates.to_string())
        {
            return Err(Status::permission_denied(
                "Insufficient permissions to create plot",
            ));
        }

        let plot = PlotModel::create_plot(
            &mut conn,
            req.estate_id,
            &req.plot_number,
            &req.phase,
            req.length,
            req.width,
        )?;

        Ok(Response::new((&plot).into()))
    }

    async fn get_plots(
        &self,
        request: Request<GetPlotsRequest>,
    ) -> Result<Response<PlotList>, Status> {
        let mut conn = self
            .database_connection_pool
            .clone()
            .get()
            .map_err(|e| Status::internal(e.to_string()))?;

        let get_plots_request = request.into_inner();

        // Validate JWT
        let jwt_token = self.jwt.decode_jwt_token(&get_plots_request.jwt_token)?;
        if jwt_token.claims.permissions.is_empty() {
            return Err(Status::permission_denied("Invalid permissions"));
        }

        let allowed_ownership_states: Vec<OwnershipState> = get_plots_request
            .allowed_ownership_states
            .iter()
            .map(|state| {
                (*state).try_into().map_err(|_e| {
                    Status::invalid_argument(format!(
                        "Invalid allowed_ownership_states: {:?}",
                        &get_plots_request.allowed_ownership_states
                    ))
                })
            })
            .collect::<Result<_, _>>()?;

        let plots: Vec<PlotModel> = PlotModel::get_plots_filtered(
            &mut conn,
            get_plots_request.plot_number.as_deref(),
            get_plots_request.estate_id,
            get_plots_request.phase.as_deref(),
            allowed_ownership_states,
        )?;

        Ok(Response::new(PlotList {
            plots: plots.iter().map(|plot| plot.into()).collect(),
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

    #[tokio::test]
    async fn test_create_plot_success() {
        let pool = crate::tests::get_test_db_connection_pool(
            "estate_management_service_test_create_plot_success",
        );
        let mut conn = pool.clone().get().unwrap();
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");

        let service = EstateManagementService {
            database_connection_pool: pool.clone(),
            jwt,
        };

        let estate = crate::model::tests::create_test_estate(&mut conn);

        let request = PlotCreationRequest {
            jwt_token: service.jwt.encode_jwt_token(
                ClaimsBuilder::new("user")
                    .permission(PermissionType::UpdateEstates.to_string())
                    .build(),
            ),
            estate_id: estate.estate_id,
            plot_number: "P-100".to_string(),
            phase: "Phase A".to_string(),
            length: 20,
            width: 30,
        };

        let plot = service
            .create_plot(Request::new(request))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(plot.plot_number, "P-100");
        assert_eq!(plot.phase, "Phase A");
        assert_eq!(plot.length, 20);
        assert_eq!(plot.width, 30);

        let plots: Vec<Plot> = PlotModel::get_plots_filtered(&mut conn, None, None, None, vec![])
            .unwrap()
            .iter()
            .map(|p| p.into())
            .collect();
        assert_eq!(plots, vec![plot]);
    }

    #[tokio::test]
    async fn test_create_plot_permission_denied() {
        let pool = crate::tests::get_test_db_connection_pool(
            "estate_management_service_test_create_plot_permission_denied",
        );
        let mut conn = pool.clone().get().unwrap();
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");

        let service = EstateManagementService {
            database_connection_pool: pool.clone(),
            jwt,
        };

        let estate = crate::model::tests::create_test_estate(&mut conn);

        let request = PlotCreationRequest {
            jwt_token: service
                .jwt
                .encode_jwt_token(ClaimsBuilder::new("user").build()),
            estate_id: estate.estate_id,
            plot_number: "P-100".to_string(),
            phase: "Phase A".to_string(),
            length: 20,
            width: 30,
        };

        let result = service.create_plot(Request::new(request)).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::PermissionDenied);
    }

    #[tokio::test]
    async fn test_get_plots_filtered() {
        let pool = crate::tests::get_test_db_connection_pool(
            "estate_management_service_test_get_plots_filtered",
        );
        let mut conn = pool.clone().get().unwrap();
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");

        let service = EstateManagementService {
            database_connection_pool: pool.clone(),
            jwt,
        };

        let estate = crate::model::tests::create_test_estate(&mut conn);

        let plot_1 =
            (&PlotModel::create_plot(&mut conn, estate.estate_id, "PN-1", "Phase X", 20, 30)
                .unwrap())
                .into();
        let plot_2 =
            (&PlotModel::create_plot(&mut conn, estate.estate_id, "PN-2", "Phase X", 25, 35)
                .unwrap())
                .into();

        let request = GetPlotsRequest {
            jwt_token: service.jwt.encode_jwt_token(
                ClaimsBuilder::new("user")
                    .permission(PermissionType::UpdateEstates.to_string())
                    .build(),
            ),
            plot_number: None,
            estate_id: None,
            phase: Some("Phase X".to_string()),
            allowed_ownership_states: vec![],
        };

        let plot_list = service
            .get_plots(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(plot_list.plots, vec![plot_1, plot_2]);
    }

    #[tokio::test]
    async fn test_get_plots_filtered_permission_denied() {
        let pool = crate::tests::get_test_db_connection_pool(
            "estate_management_service_test_get_plots_filtered_permission_denied",
        );
        let mut conn = pool.clone().get().unwrap();
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");

        let service = EstateManagementService {
            database_connection_pool: pool.clone(),
            jwt,
        };

        let estate = crate::model::tests::create_test_estate(&mut conn);

        let _plot_1 =
            PlotModel::create_plot(&mut conn, estate.estate_id, "PN-1", "Phase X", 20, 30).unwrap();
        let _plot_2 =
            PlotModel::create_plot(&mut conn, estate.estate_id, "PN-2", "Phase X", 25, 35).unwrap();

        let request = GetPlotsRequest {
            jwt_token: service
                .jwt
                .encode_jwt_token(ClaimsBuilder::new("user").build()),
            plot_number: None,
            estate_id: None,
            phase: Some("Phase X".to_string()),
            allowed_ownership_states: vec![],
        };

        let result = service.get_plots(Request::new(request)).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::PermissionDenied);
    }
}
