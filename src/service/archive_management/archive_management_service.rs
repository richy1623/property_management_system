use super::archive_management_proto::{
    Artifact, ArtifactRef, GetArtifactRequest, StoreArtifactRequest,
    artifact_management_server::ArtifactManagement,
};
use crate::jwt::Jwt;
use crate::model::user_permission::PermissionType;
use crate::model::{artifact::Artifact as ArtifactModel, user::User as UserModel};
use crate::secure_storage::SecureStorage;
use chrono::Utc;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct ArtifactManagementService {
    database_connection_pool: Pool<ConnectionManager<PgConnection>>,
    jwt: Jwt,
    secure_storage: SecureStorage,
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
