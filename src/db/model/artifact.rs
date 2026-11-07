use crate::db::schema::artifact::{self};
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use sanitize_filename::sanitize;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::Error;
use chrono::Utc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Serialize, Deserialize, ToSchema)]
#[db_enum(
    existing_type_path = "crate::db::schema::sql_types::ArtifactServiceType",
    value_style = "SCREAMING_SNAKE_CASE"
)]
#[derive(strum::Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ArtifactServiceType {
    User,
    Estate,
    Plot,
    Ownership,
    Payment,
}

#[derive(
    Queryable, Selectable, Insertable, Debug, PartialEq, Eq, Clone, Serialize, Deserialize,
)]
#[diesel(table_name = artifact)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Artifact {
    pub artifact_id: i32,
    pub file_display_name: String,
    pub file_path: String,
    pub upload_date: chrono::NaiveDate,
    pub uploaded_by: i32,
    pub service_type: ArtifactServiceType,
    pub service_reference_id: i32,
}

impl Artifact {
    pub fn create_artifact(
        conn: &mut PgConnection,
        file_display_name: &str,
        file_path: &str,
        uploaded_by: i32,
        service_type: ArtifactServiceType,
        service_reference_id: i32,
    ) -> Result<Artifact, Error> {
        let file_display_name = sanitize(file_display_name);
        diesel::insert_into(artifact::table)
            .values((
                artifact::file_display_name.eq(file_display_name),
                artifact::file_path.eq(file_path),
                artifact::upload_date.eq(Utc::now().date_naive()),
                artifact::uploaded_by.eq(uploaded_by),
                artifact::service_type.eq(service_type),
                artifact::service_reference_id.eq(service_reference_id),
            ))
            .get_result(conn)
            .map_err(Into::into)
    }

    pub fn get_artifact(conn: &mut PgConnection, artifact_id: i32) -> Result<Artifact, Error> {
        artifact::table
            .find(artifact_id)
            .select(Artifact::as_select())
            .first(conn)
            .map_err(Into::into)
    }

    pub fn get_all_artifact_ids_for_artifact_service_reference(
        conn: &mut PgConnection,
        artifact_service: ArtifactServiceType,
        artifact_service_reference_id: i32,
    ) -> Result<Vec<i32>, Error> {
        artifact::table
            .filter(artifact::service_type.eq(artifact_service))
            .filter(artifact::service_reference_id.eq(artifact_service_reference_id))
            .select(artifact::artifact_id)
            .get_results(conn)
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::{
        PgConnection,
        r2d2::{ConnectionManager, Pool},
    };
    use rstest::{fixture, rstest};

    #[fixture]
    #[once]
    pub fn pool() -> Pool<ConnectionManager<PgConnection>> {
        crate::tests::get_test_db_connection_pool("test_artifact")
    }

    #[rstest]
    pub fn test_create_artifact(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let file_display_name = "test_document.pdf";
        let file_path = "/tmp/test_document.pdf";

        let user = crate::db::model::tests::create_test_user(&mut conn);
        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let ownership = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);

        let artifact = Artifact::create_artifact(
            &mut conn,
            file_display_name,
            file_path,
            user.user_id,
            ArtifactServiceType::Ownership,
            ownership.ownership_id,
        )
        .unwrap();

        assert_eq!(artifact.file_display_name, file_display_name);
        assert_eq!(artifact.file_path, file_path);
        assert_eq!(artifact.upload_date, Utc::now().date_naive());
        assert_eq!(artifact.uploaded_by, user.user_id);
        assert_eq!(artifact.service_type, ArtifactServiceType::Ownership);
        assert_eq!(artifact.service_reference_id, ownership.ownership_id);

        assert_eq!(
            artifact,
            Artifact::get_artifact(&mut conn, artifact.artifact_id).unwrap()
        );
    }

    #[rstest]
    fn test_get_artifact(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let user = crate::db::model::tests::create_test_user(&mut conn);
        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let ownership = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);
        let artifact = crate::db::model::tests::create_test_artifact(
            &mut conn,
            &user,
            "file_path",
            ArtifactServiceType::Ownership,
            ownership.ownership_id,
        );

        assert_eq!(
            artifact,
            Artifact::get_artifact(&mut conn, artifact.artifact_id).unwrap()
        );

        assert!(Artifact::get_artifact(&mut conn, -1).is_err())
    }

    #[rstest]
    pub fn test_get_all_artifacts_for_ownership(pool: &Pool<ConnectionManager<PgConnection>>) {
        let mut conn = pool.clone().get().unwrap();

        let estate = crate::db::model::tests::create_test_estate(&mut conn);
        let plot = crate::db::model::tests::create_test_plot(&mut conn, &estate);
        let user = crate::db::model::tests::create_test_user(&mut conn);
        let ownership_1 = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);
        let ownership_2 = crate::db::model::tests::create_test_ownership(&mut conn, &plot, &user);
        let artifact_1 = crate::db::model::tests::create_test_artifact(
            &mut conn,
            &user,
            "file_path",
            ArtifactServiceType::Ownership,
            ownership_1.ownership_id,
        );
        let artifact_2 = crate::db::model::tests::create_test_artifact(
            &mut conn,
            &user,
            "file_path",
            ArtifactServiceType::Ownership,
            ownership_1.ownership_id,
        );
        let artifact_3 = crate::db::model::tests::create_test_artifact(
            &mut conn,
            &user,
            "file_path",
            ArtifactServiceType::Ownership,
            ownership_2.ownership_id,
        );

        assert_eq!(
            Artifact::get_all_artifact_ids_for_artifact_service_reference(
                &mut conn,
                ArtifactServiceType::Ownership,
                ownership_1.ownership_id
            )
            .unwrap(),
            vec![artifact_1.artifact_id, artifact_2.artifact_id]
        );
        assert_eq!(
            Artifact::get_all_artifact_ids_for_artifact_service_reference(
                &mut conn,
                ArtifactServiceType::Ownership,
                ownership_2.ownership_id
            )
            .unwrap(),
            vec![artifact_3.artifact_id]
        );
        assert_eq!(
            Artifact::get_all_artifact_ids_for_artifact_service_reference(
                &mut conn,
                ArtifactServiceType::Ownership,
                -1
            )
            .unwrap(),
            Vec::<i32>::new()
        );
    }
}
