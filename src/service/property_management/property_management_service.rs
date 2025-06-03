use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use diesel_migrations::MigrationHarness;
use tonic::{Request, Response, Status};

use super::property_management_proto::{
    Empty, Estate, EstateCreationRequest, EstateList, GetPlotsRequest, Ownership,
    OwnershipCreationRequest, OwnershipUpdateRequest, Plot, PlotCreationRequest, PlotList,
    property_management_server::PropertyManagement,
};
use crate::db::model::ownership::Ownership as OwnershipModel;
use crate::db::model::plot::Plot as PlotModel;
use crate::db::model::user::User as UserModel;
use crate::db::model::{estate::Estate as EstateModel, ownership::OwnershipState};
use crate::{MIGRATIONS, db::model::user_permission::PermissionType, jwt::Jwt};

pub struct PropertyManagementService {
    database_connection_pool: Pool<ConnectionManager<PgConnection>>,
    jwt: Jwt,
}

impl PropertyManagementService {
    pub fn new(database_connection_url: &str, jwt_secret_key: &str) -> Self {
        let database_connection_pool = crate::get_connection_pool(database_connection_url);
        database_connection_pool
            .clone()
            .get()
            .expect("Failed getting db pool")
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed running migrations");

        PropertyManagementService {
            database_connection_pool: database_connection_pool,
            jwt: Jwt::new(jwt_secret_key),
        }
    }
}

#[tonic::async_trait]
impl PropertyManagement for PropertyManagementService {
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
            .contains(&PermissionType::ModifyProperties.to_string())
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

        let reply = super::property_management_proto::Estate {
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
            .map(|estate| super::property_management_proto::Estate {
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
            .contains(&PermissionType::ModifyProperties.to_string())
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
        self.jwt.decode_jwt_token(&get_plots_request.jwt_token)?;

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

    async fn create_ownership(
        &self,
        request: Request<OwnershipCreationRequest>,
    ) -> Result<Response<Ownership>, Status> {
        let mut conn = self
            .database_connection_pool
            .get()
            .map_err(|e| Status::internal(e.to_string()))?;

        let request = request.into_inner();

        // Validate JWT token
        let jwt = self.jwt.decode_jwt_token(&request.jwt_token)?;

        // Check permission
        if !jwt
            .claims
            .permissions
            .contains(&PermissionType::ModifyOwnerships.to_string())
        {
            return Err(Status::permission_denied("Insufficient permissions"));
        }

        // Create ownership in DB
        let ownership = OwnershipModel::create_ownership(
            &mut conn,
            request.user_id,
            request.plot_id,
            request.state.try_into().map_err(|_e| {
                Status::invalid_argument(format!("Invalid state: {:?}", request.state))
            })?,
            request.total_sale_value,
            request.monthly_repayment_amount,
        )?;

        // Fetch the user to get username
        let user = UserModel::get_user_by_user_id(&mut conn, request.user_id)?;

        Ok(Response::new(Ownership {
            ownership_id: ownership.ownership_id,
            user_id: user.user_id,
            user_name: user.user_name,
            plot_id: ownership.plot_id,
            state: ownership.state.into(),
            total_sale_value: ownership.total_sale_value,
            monthly_repayment_amount: ownership.monthly_repayment_amount,
        }))
    }

    async fn update_ownership(
        &self,
        request: Request<OwnershipUpdateRequest>,
    ) -> Result<Response<Ownership>, Status> {
        let OwnershipUpdateRequest {
            jwt_token,
            ownership_id,
            state,
            total_sale_value,
            monthly_repayment_amount,
        } = request.into_inner();

        // Get DB connection
        let mut conn = self
            .database_connection_pool
            .clone()
            .get()
            .map_err(|e| Status::internal(e.to_string()))?;

        // Decode and validate JWT
        let jwt_token = self.jwt.decode_jwt_token(&jwt_token)?;
        if !jwt_token
            .claims
            .permissions
            .contains(&PermissionType::ModifyOwnerships.to_string())
        {
            return Err(Status::permission_denied(
                "Insufficient permissions to modify ownerships",
            ));
        }

        // Get user making update
        let user_making_update =
            UserModel::get_user_by_user_name(&mut conn, &jwt_token.claims.sub)?;

        // Parse ownership state from i32 to enum
        let new_ownership_state: Option<OwnershipState> = match state {
            Some(s) => Some(
                OwnershipState::try_from(s).map_err(|e| Status::invalid_argument(e.to_string()))?,
            ),
            None => None,
        };

        // Update ownership
        let updated_ownership = OwnershipModel::update_ownership(
            &mut conn,
            ownership_id,
            new_ownership_state,
            total_sale_value,
            monthly_repayment_amount,
            user_making_update.user_id,
        )?;

        Ok(Response::new(Ownership::from_model_and_user(
            &updated_ownership,
            &user_making_update,
        )))
    }
}

#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    use super::*;
    use crate::jwt::ClaimsBuilder;

    #[fixture]
    #[once]
    pub fn pool() -> Pool<ConnectionManager<PgConnection>> {
        crate::tests::get_test_db_connection_pool("property_management_service")
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_estate_success(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");

        let service = PropertyManagementService {
            database_connection_pool: pool.clone(),
            jwt,
        };

        // Create a valid request
        let request = EstateCreationRequest {
            jwt_token: service.jwt.encode_jwt_token(
                ClaimsBuilder::new("user")
                    .permission(PermissionType::ModifyProperties.to_string())
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
        assert!(estates.contains(&estate));
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_estate_permission_denied(pool: &Pool<ConnectionManager<PgConnection>>) {
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");

        let service = PropertyManagementService {
            database_connection_pool: pool.clone(),
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

    #[rstest]
    #[tokio::test]
    async fn test_get_all_estates_success(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");

        let service = PropertyManagementService {
            database_connection_pool: pool.clone(),
            jwt,
        };

        let estate_1: Estate = crate::db::model::tests::create_test_estate(&mut conn).into();
        let estate_2: Estate = crate::db::model::tests::create_test_estate(&mut conn).into();

        let request = Request::new(Empty {});

        let result = service.get_all_estates(request).await.unwrap().into_inner();

        assert!(result.estates.contains(&estate_1));
        assert!(result.estates.contains(&estate_2));
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_plot_success(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");

        let service = PropertyManagementService {
            database_connection_pool: pool.clone(),
            jwt,
        };

        let estate = crate::db::model::tests::create_test_estate(&mut conn);

        let request = PlotCreationRequest {
            jwt_token: service.jwt.encode_jwt_token(
                ClaimsBuilder::new("user")
                    .permission(PermissionType::ModifyProperties.to_string())
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

        let plot_in_db = PlotModel::get_plot_with_plot_number_and_estate_id(
            &mut conn,
            "P-100",
            estate.estate_id,
        )
        .unwrap();
        assert_eq!(Plot::from(&plot_in_db), plot);
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_plot_permission_denied(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");

        let service = PropertyManagementService {
            database_connection_pool: pool.clone(),
            jwt,
        };

        let estate = crate::db::model::tests::create_test_estate(&mut conn);

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

    #[rstest]
    #[tokio::test]
    async fn test_get_plots_filtered(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");

        let service = PropertyManagementService {
            database_connection_pool: pool.clone(),
            jwt,
        };

        let estate = crate::db::model::tests::create_test_estate(&mut conn);

        let plot_1 =
            PlotModel::create_plot(&mut conn, estate.estate_id, "PN-1", "Phase X", 20, 30).unwrap();
        let plot_2 =
            PlotModel::create_plot(&mut conn, estate.estate_id, "PN-2", "Phase X", 25, 35).unwrap();
        let _plot_3 =
            PlotModel::create_plot(&mut conn, estate.estate_id, "PN-3", "Phase X", 25, 35).unwrap();

        let user = crate::db::model::tests::create_test_user(&mut conn);
        crate::db::model::tests::create_test_ownership(&mut conn, &plot_1, &user);
        crate::db::model::tests::create_test_ownership(&mut conn, &plot_2, &user);

        let request = GetPlotsRequest {
            jwt_token: service.jwt.encode_jwt_token(
                ClaimsBuilder::new("user")
                    .permission(PermissionType::ModifyProperties.to_string())
                    .build(),
            ),
            plot_number: None,
            estate_id: None,
            phase: Some("Phase X".to_string()),
            allowed_ownership_states: vec![0],
        };

        let plot_list = service
            .get_plots(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(
            plot_list.plots,
            vec![Into::<Plot>::into(&plot_1), Into::<Plot>::into(&plot_2)]
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_plots_filtered_permission_denied(
        pool: &Pool<ConnectionManager<PgConnection>>,
    ) {
        let mut conn = pool.clone().get().unwrap();
        let jwt = Jwt::new("a-string-secret-at-least-256-bits-long");

        let service = PropertyManagementService {
            database_connection_pool: pool.clone(),
            jwt,
        };

        let estate = crate::db::model::tests::create_test_estate(&mut conn);

        let _plot_1 =
            PlotModel::create_plot(&mut conn, estate.estate_id, "PN-1", "Phase X", 20, 30).unwrap();
        let _plot_2 =
            PlotModel::create_plot(&mut conn, estate.estate_id, "PN-2", "Phase X", 25, 35).unwrap();

        let request = GetPlotsRequest {
            jwt_token: "InvalidJwtToken".to_string(),
            plot_number: None,
            estate_id: None,
            phase: Some("Phase X".to_string()),
            allowed_ownership_states: vec![],
        };

        let result = service.get_plots(Request::new(request)).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_ownership_with_permission(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.get().unwrap();

        let service = PropertyManagementService {
            database_connection_pool: pool.clone(),
            jwt: Jwt::new("a-string-secret-at-least-256-bits-long"),
        };

        // Setup test user and plot
        let user = crate::db::model::tests::create_test_user(&mut conn);
        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot =
            PlotModel::create_plot(&mut conn, estate.estate_id, "P123", "Alpha", 50, 40).unwrap();

        let request = OwnershipCreationRequest {
            jwt_token: service.jwt.encode_jwt_token(
                ClaimsBuilder::new("admin")
                    .permission(PermissionType::ModifyOwnerships.to_string())
                    .build(),
            ),
            user_id: user.user_id,
            plot_id: plot.plot_id,
            state: OwnershipState::ActivePayment.into(),
            total_sale_value: 100_000,
            monthly_repayment_amount: 2_000,
        };

        let response = service
            .create_ownership(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(response.user_id, user.user_id);
        assert_eq!(response.user_name, user.user_name);
        assert_eq!(response.plot_id, plot.plot_id);
        assert_eq!(
            response.state,
            Into::<i32>::into(OwnershipState::ActivePayment)
        );
        assert_eq!(response.total_sale_value, 100_000);
        assert_eq!(response.monthly_repayment_amount, 2_000);
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_ownership_without_permission(
        pool: &Pool<ConnectionManager<PgConnection>>,
    ) {
        let service = PropertyManagementService {
            database_connection_pool: pool.clone(),
            jwt: Jwt::new("a-string-secret-at-least-256-bits-long"),
        };

        let request = OwnershipCreationRequest {
            jwt_token: service
                .jwt
                .encode_jwt_token(ClaimsBuilder::new("basic_user").build()),
            user_id: 1,
            plot_id: 1,
            state: OwnershipState::ActivePayment.into(),
            total_sale_value: 50_000,
            monthly_repayment_amount: 1_000,
        };

        let response = service.create_ownership(Request::new(request)).await;

        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code(), tonic::Code::PermissionDenied);
    }

    #[rstest]
    #[tokio::test]
    async fn test_update_ownership(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.get().unwrap();

        // Create test ownership
        let user = crate::db::model::tests::create_test_user(&mut conn);
        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let ownership = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);

        let service = PropertyManagementService {
            database_connection_pool: pool.clone(),
            jwt: Jwt::new("a-secret-at-least-256-bits-long"),
        };

        // Perform update
        let ownership_update_request = OwnershipUpdateRequest {
            jwt_token: service.jwt.encode_jwt_token(
                ClaimsBuilder::new(&user.user_name)
                    .permission(PermissionType::ModifyOwnerships.to_string())
                    .build(),
            ),
            ownership_id: ownership.ownership_id,
            state: Some(OwnershipState::Paid.into()),
            total_sale_value: Some(10),
            monthly_repayment_amount: Some(1),
        };

        let response = service
            .update_ownership(Request::new(ownership_update_request))
            .await
            .unwrap();

        let updated = response.into_inner();

        // Check returned values
        assert_eq!(updated.ownership_id, ownership.ownership_id);
        assert_eq!(updated.state, OwnershipState::Paid as i32);
        assert_eq!(updated.total_sale_value, 10);
        assert_eq!(updated.monthly_repayment_amount, 1);
        assert_eq!(updated.plot_id, ownership.plot_id);
        assert_eq!(updated.user_id, ownership.user_id);

        // Confirm persisted changes
        let db_ownership = Ownership::from_model_and_user(
            &OwnershipModel::get_ownership(&mut conn, ownership.ownership_id).unwrap(),
            &user,
        );

        assert_eq!(db_ownership, updated);
    }
}
