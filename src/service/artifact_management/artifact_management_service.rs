use super::artifact_management_proto::{
    Artifact, ArtifactRef, GetArtifactRequest, StoreArtifactRequest,
    artifact_management_server::ArtifactManagement,
};
use crate::db::model::user_permission::PermissionType;
use crate::db::model::{artifact::Artifact as ArtifactModel, user::User as UserModel};
use crate::jwt::Jwt;
use crate::secure_storage::SecureStorage;
use chrono::Utc;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

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

#[tonic::async_trait]
impl ArtifactManagement for ArtifactManagementService {
    async fn store_artifact(
        &self,
        request: Request<StoreArtifactRequest>,
    ) -> Result<Response<ArtifactRef>, Status> {
        let store_artifact_request = request.into_inner();

        let mut conn = self
            .database_connection_pool
            .get()
            .map_err(|e| Status::internal(e.to_string()))?;

        let jwt_token = self
            .jwt
            .decode_jwt_token(&store_artifact_request.jwt_token)?;

        if !jwt_token
            .claims
            .permissions
            .contains(&PermissionType::ModifyPayments.to_string())
        {
            return Err(Status::permission_denied(
                "Insufficient permissions to store artifact",
            ));
        }

        let user = UserModel::get_user_by_user_name(&mut conn, &jwt_token.claims.sub)?;

        // Store the artifact
        let file_name = Uuid::new_v4().to_string();
        let stored_file_ref = self
            .secure_storage
            .store_file(
                &file_name,
                &store_artifact_request.file_content,
                Utc::now().date_naive(),
            )
            .await?;

        let artifact_model = ArtifactModel::create_artifact(
            &mut conn,
            &store_artifact_request.file_display_name,
            stored_file_ref.to_str().unwrap(),
            user.user_id,
            true,
        )?;

        Ok(Response::new((&artifact_model).into()))
    }

    async fn get_artifact(
        &self,
        request: Request<GetArtifactRequest>,
    ) -> Result<Response<Artifact>, Status> {
        let req = request.into_inner();

        let mut conn = self
            .database_connection_pool
            .get()
            .map_err(|e| Status::internal(e.to_string()))?;

        let jwt_token = self.jwt.decode_jwt_token(&req.jwt_token)?;

        if !jwt_token
            .claims
            .permissions
            .contains(&PermissionType::ViewPayments.to_string())
        {
            return Err(Status::permission_denied(
                "Insufficient permissions to view artifact",
            ));
        }

        let artifact = ArtifactModel::get_artifact(&mut conn, req.artifact_id)?;

        let file_contents = self.secure_storage.read_file(&artifact.file_path).await?;

        Ok(Response::new(Artifact::from_model(
            &artifact,
            file_contents,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::model::artifact::Artifact as ArtifactModel;
    use crate::db::model::user_permission::PermissionType;
    use crate::jwt::ClaimsBuilder;
    use rstest::{fixture, rstest};
    use tempfile::TempDir;

    #[fixture]
    #[once]
    pub fn pool() -> Pool<ConnectionManager<PgConnection>> {
        crate::tests::get_test_db_connection_pool("artifact_management_service")
    }

    #[rstest]
    #[tokio::test]
    async fn test_store_artifact_success(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.get().unwrap();

        let jwt = Jwt::new("a-secret-at-least-256-bits-long");
        let temp_dir = TempDir::new().unwrap();

        let secure_storage = SecureStorage::new(temp_dir.path(), &[0u8; 32]).unwrap();

        let service = ArtifactManagementService {
            database_connection_pool: pool.clone(),
            jwt: jwt.clone(),
            secure_storage,
        };

        let user = crate::db::model::tests::create_test_user(&mut conn);

        let token = jwt.encode_jwt_token(
            ClaimsBuilder::new(&user.user_name)
                .permission(PermissionType::ModifyPayments.to_string())
                .build(),
        );

        let req = StoreArtifactRequest {
            jwt_token: token,
            file_display_name: "Test File".to_string(),
            file_content: b"file data".to_vec(),
        };

        let response = service
            .store_artifact(Request::new(req))
            .await
            .unwrap()
            .into_inner();

        let artifact = ArtifactModel::get_artifact(&mut conn, response.artifact_id).unwrap();
        assert_eq!(artifact.file_display_name, "Test File");
    }

    #[rstest]
    #[tokio::test]
    async fn test_store_artifact_invalid_permissions(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.get().unwrap();

        let jwt = Jwt::new("a-secret-at-least-256-bits-long");
        let temp_dir = TempDir::new().unwrap();

        let secure_storage = SecureStorage::new(temp_dir.path(), &[0u8; 32]).unwrap();

        let service = ArtifactManagementService {
            database_connection_pool: pool.clone(),
            jwt: jwt.clone(),
            secure_storage,
        };

        let user = crate::db::model::tests::create_test_user(&mut conn);

        let token = jwt.encode_jwt_token(ClaimsBuilder::new(&user.user_name).build());

        let req = StoreArtifactRequest {
            jwt_token: token,
            file_display_name: "No Perms".to_string(),
            file_content: b"data".to_vec(),
        };

        let result = service.store_artifact(Request::new(req)).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::PermissionDenied);
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_artifact_success(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.get().unwrap();

        let jwt = Jwt::new("a-secret-at-least-256-bits-long");
        let temp_dir = TempDir::new().unwrap();

        let secure_storage = SecureStorage::new(temp_dir.path(), &[0u8; 32]).unwrap();

        let service = ArtifactManagementService {
            database_connection_pool: pool.clone(),
            jwt: jwt.clone(),
            secure_storage,
        };

        let user = crate::db::model::tests::create_test_user(&mut conn);

        let token = jwt.encode_jwt_token(
            ClaimsBuilder::new(&user.user_name)
                .permission(PermissionType::ViewPayments.to_string())
                .build(),
        );

        let path = service
            .secure_storage
            .store_file(
                "sample",
                b"artifact content",
                chrono::Utc::now().date_naive(),
            )
            .await
            .unwrap();

        let artifact = ArtifactModel::create_artifact(
            &mut conn,
            "GetFile",
            path.to_str().unwrap(),
            user.user_id,
            true,
        )
        .unwrap();

        let req = GetArtifactRequest {
            jwt_token: token,
            artifact_id: artifact.artifact_id,
        };

        let response = service
            .get_artifact(Request::new(req))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(response.file_display_name, "GetFile");
        assert_eq!(response.file_content, b"artifact content");
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_artifact_invalid_permissions(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.get().unwrap();

        let jwt = Jwt::new("a-secret-at-least-256-bits-long");
        let temp_dir = TempDir::new().unwrap();

        let secure_storage = SecureStorage::new(temp_dir.path(), &[0u8; 32]).unwrap();

        let service = ArtifactManagementService {
            database_connection_pool: pool.clone(),
            jwt: jwt.clone(),
            secure_storage,
        };

        let user = crate::db::model::tests::create_test_user(&mut conn);

        let token = jwt.encode_jwt_token(ClaimsBuilder::new(&user.user_name).build());

        let req = GetArtifactRequest {
            jwt_token: token,
            artifact_id: 123, // nonexistent
        };

        let result = service.get_artifact(Request::new(req)).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::PermissionDenied);
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_artifact_no_such_file(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.get().unwrap();

        let jwt = Jwt::new("a-secret-at-least-256-bits-long");
        let temp_dir = TempDir::new().unwrap();

        let secure_storage = SecureStorage::new(temp_dir.path(), &[0u8; 32]).unwrap();

        let service = ArtifactManagementService {
            database_connection_pool: pool.clone(),
            jwt: jwt.clone(),
            secure_storage,
        };

        let user = crate::db::model::tests::create_test_user(&mut conn);

        let token = jwt.encode_jwt_token(
            ClaimsBuilder::new(&user.user_name)
                .permission(PermissionType::ViewPayments.to_string())
                .build(),
        );

        let fake_path = temp_dir.path().join("nonexistent_file.enc");
        let artifact = ArtifactModel::create_artifact(
            &mut conn,
            "MissingFile",
            fake_path.to_str().unwrap(),
            user.user_id,
            true,
        )
        .unwrap();

        let req = GetArtifactRequest {
            jwt_token: token,
            artifact_id: artifact.artifact_id,
        };

        let result = service.get_artifact(Request::new(req)).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Internal);
    }
}
