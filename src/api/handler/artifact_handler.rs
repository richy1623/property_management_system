use std::sync::Arc;

use crate::{
    api::{
        AppServices,
        model::artifact::{Artifact, ArtifactReference, ArtifactUploadRequest},
    },
    db::model::{
        artifact::Artifact as ArtifactModel, user::User as UserModel,
        user_permission::PermissionType,
    },
};
use actix_web::Result;
use actix_web::{HttpRequest, HttpResponse, Responder, error::ErrorUnauthorized, web};
use chrono::Utc;

/// Retrieve artifact with a given artifact ID
///
/// Accepts an artifact ID and returns the artifact associated with it.
/// Authorization: bearer token.
#[utoipa::path(
    get,
    path = "/api/artifact/{artifact_id}",
    tag = "Artifact Management",
    params(
        ("artifact_id" = i32, Path, description = "Artifact ID to return", example="1001")
    ),
    security(
        ("bearerAuth" = [])
    ),
    responses(
        (status = 200, description = "The requested artifact", body = Artifact),
        (status = 400, description = "Missing or invalid input"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_artifact(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    path: web::Path<i32>,
) -> Result<impl Responder> {
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;

    let artifact_id = path.into_inner();

    let mut conn = super::get_database_connection(
        services
            .archive_management_service
            .database_connection_pool
            .clone(),
    )?;

    // Validate JWT
    let jwt_token = services
        .payment_management_service
        .jwt
        .decode_jwt_token(&authorization_header)?;

    // let user = UserModel::get_user_by_user_name(&mut conn, jwt_token.claims.sub)?;

    if !jwt_token
        .claims
        .permissions
        .contains(&PermissionType::ViewPayments.to_string())
    {
        return Err(ErrorUnauthorized(
            "Insufficient permissions to view artifact",
        ));
    }

    let artifact = ArtifactModel::get_artifact(&mut conn, artifact_id)?;

    let file_contents = services
        .archive_management_service
        .secure_storage
        .read_file(artifact.file_path)
        .await?;

    Ok(HttpResponse::Ok().json(Artifact {
        artifact_id,
        file_display_name: artifact.file_display_name,
        upload_date: artifact.upload_date,
        uploaded_by: artifact.uploaded_by,
        file_content: file_contents,
    }))
}

/// Upload a new artifact to the system.
///
/// The uploaded file is persisted into secure storage, and a database
/// record is created referencing the stored file.
///
/// Authorization: bearer token with permissions to upload artifacts.
#[utoipa::path(
    post,
    path = "/api/artifact/",
    tag = "Artifact Management",
    request_body(
        content = ArtifactUploadRequest,
        description = "Artifact upload request containing file + metadata"
    ),
    security(
        ("bearerAuth" = [])
    ),
    responses(
        (status = 201, description = "Artifact successfully uploaded", body = ArtifactReference),
        (status = 400, description = "Invalid input or file data"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Unexpected server error")
    )
)]
pub async fn upload_artifact(
    services: web::Data<Arc<AppServices>>,
    http_request: HttpRequest,
    request: web::Json<ArtifactUploadRequest>,
) -> Result<impl Responder> {
    let authorization_header = crate::api::extract_authorization_header(&http_request)?;
    // Extract request body
    let ArtifactUploadRequest {
        artifact_file,
        artifact_service,
        service_reference_id,
    } = request.into_inner();

    // Validate JWT
    let jwt_token = services
        .payment_management_service
        .jwt
        .decode_jwt_token(&authorization_header)?;

    // Map the required Permissions
    let required_permission = match artifact_service {
        crate::db::model::artifact::ArtifactServiceType::User => PermissionType::ModifyUsers,
        crate::db::model::artifact::ArtifactServiceType::Estate
        | crate::db::model::artifact::ArtifactServiceType::Plot
        | crate::db::model::artifact::ArtifactServiceType::Ownership => {
            PermissionType::ModifyProperties
        }
        crate::db::model::artifact::ArtifactServiceType::Payment => PermissionType::ModifyPayments,
    };

    // Ensure user has permission to upload artifacts
    if !jwt_token
        .claims
        .permissions
        .contains(&required_permission.to_string())
    {
        return Err(ErrorUnauthorized(
            "Insufficient permissions to upload artifact",
        ));
    }

    let mut conn = super::get_database_connection(
        services
            .archive_management_service
            .database_connection_pool
            .clone(),
    )?;

    let user_making_request = UserModel::get_user_by_user_name(&mut conn, &jwt_token.claims.sub)?;
    // Persist file to secure storage
    let stored_path = services
        .archive_management_service
        .secure_storage
        .store_file(
            &artifact_file.file_name,
            &artifact_file.file_content,
            Utc::now().date_naive(),
        )
        .await?;

    // Create DB record
    let created = ArtifactModel::create_artifact(
        &mut conn,
        &artifact_file.file_name,
        &stored_path.to_str().unwrap(),
        user_making_request.user_id,
        artifact_service,
        service_reference_id,
    )?;

    // Return lightweight response object
    Ok(HttpResponse::Created().json(ArtifactReference {
        artifact_id: created.artifact_id,
        file_display_name: created.file_display_name,
    }))
}

#[cfg(test)]
mod tests {
    use crate::{
        db::model::{artifact::ArtifactServiceType, user_permission::PermissionType},
        jwt::{ClaimsBuilder, Jwt},
        secure_storage::SecureStorage,
    };

    use super::*;

    use actix_web::{App, http::StatusCode, test};
    use chrono::Utc;
    use diesel::{
        PgConnection,
        r2d2::{ConnectionManager, Pool},
    };
    use rstest::{fixture, rstest};
    use uuid::Uuid;

    #[fixture]
    #[once]
    pub fn test_services() -> (
        AppServices,
        Jwt,
        SecureStorage,
        Pool<ConnectionManager<PgConnection>>,
    ) {
        crate::api::tests::create_test_app_service("test_artifact_handler")
    }

    #[rstest]
    #[actix_web::test]
    async fn test_get_artifact(
        test_services: &(
            AppServices,
            Jwt,
            SecureStorage,
            Pool<ConnectionManager<PgConnection>>,
        ),
    ) {
        let (services, jwt, secure_storage, pool) = test_services.clone();
        let mut conn = pool.get().unwrap();

        let app_services = Arc::new(services);
        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(app_services.clone()))
                .route("/api/artifact/{artifact_id}", web::get().to(get_artifact)),
        )
        .await;

        // Create database entries
        let user = crate::db::model::tests::create_test_user(&mut conn);

        let test_file_content = "hello world!";
        let test_file_name = "test.txt";

        let file_name = Uuid::new_v4().to_string();
        let stored_file_ref = secure_storage
            .store_file(
                &file_name,
                test_file_content.as_bytes(),
                Utc::now().date_naive(),
            )
            .await
            .unwrap();

        let artifact_model = ArtifactModel::create_artifact(
            &mut conn,
            &test_file_name,
            stored_file_ref.to_str().unwrap(),
            user.user_id,
            ArtifactServiceType::User,
            user.user_id,
        )
        .unwrap();

        // Act: Send http request
        let jwt_token = jwt.encode_jwt_token(
            ClaimsBuilder::new("admin")
                .permission(PermissionType::ViewPayments.to_string())
                .build(),
        );

        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/artifact/{}",
                artifact_model.artifact_id.to_string()
            ))
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert: Check HTTP status code and returned payload
        assert_eq!(resp.status(), StatusCode::OK);

        let artifact: Artifact = test::read_body_json(resp).await;
        assert_eq!(artifact.artifact_id, artifact_model.artifact_id);
        assert_eq!(artifact.file_content, test_file_content.as_bytes());
        assert_eq!(artifact.file_display_name, "test.txt");
        assert_eq!(artifact.upload_date, Utc::now().date_naive());
        assert_eq!(artifact.uploaded_by, user.user_id);
    }

    #[rstest]
    #[actix_web::test]
    async fn test_upload_artifact_success(
        test_services: &(
            AppServices,
            Jwt,
            SecureStorage,
            Pool<ConnectionManager<PgConnection>>,
        ),
    ) {
        let (services, jwt, secure_storage, pool) = test_services.clone();
        let mut conn = pool.get().unwrap();

        let app_services = Arc::new(services);

        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(app_services.clone()))
                .route("/api/artifact/", web::post().to(upload_artifact)),
        )
        .await;

        // --- Arrange ------------------------------------------------------------------

        // Create test user
        let user = crate::db::model::tests::create_test_user(&mut conn);

        // Test file
        let test_file_name = "upload_test.pdf";
        let test_file_bytes = b"this is a test artifact upload";

        // Artifact belongs to Ownership and will be attached to reference id:
        let service_reference_id = 77;

        // Create JWT with required permissions
        let jwt_token = jwt.encode_jwt_token(
            ClaimsBuilder::new(&user.user_name)
                .permission(PermissionType::ModifyProperties.to_string())
                .build(),
        );

        // Construct request body
        let request_body = serde_json::json!({
            "artifact_file": {
                "file_name": test_file_name,
                "file_content": test_file_bytes
            },
            "artifact_service": "Ownership",
            "service_reference_id": service_reference_id
        });

        // --- Act ----------------------------------------------------------------------

        let req = test::TestRequest::post()
            .uri("/api/artifact/")
            .insert_header(("Authorization", format!("Bearer {jwt_token}")))
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // --- Assert: response --------------------------------------------------------

        assert_eq!(resp.status(), StatusCode::CREATED);

        let returned: ArtifactReference = test::read_body_json(resp).await;

        assert_eq!(returned.file_display_name, test_file_name);

        // --- Assert: database record -------------------------------------------------

        // Check DB contains new artifact entry
        let artifact_in_db = ArtifactModel::get_artifact(&mut conn, returned.artifact_id)
            .expect("Artifact should exist in DB after upload");

        assert_eq!(artifact_in_db.file_display_name, test_file_name);
        assert_eq!(artifact_in_db.service_reference_id, service_reference_id);
        assert_eq!(artifact_in_db.service_type, ArtifactServiceType::Ownership);
        assert_eq!(artifact_in_db.uploaded_by, user.user_id);

        // --- Assert: stored file contents -------------------------------------------

        let stored_file_contents = secure_storage
            .read_file(artifact_in_db.file_path.clone())
            .await
            .expect("Secure storage file must exist");

        assert_eq!(stored_file_contents, test_file_bytes);
    }
}
