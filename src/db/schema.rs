// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "ownership_state"))]
    pub struct OwnershipState;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "payment_state"))]
    pub struct PaymentState;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "permission_type"))]
    pub struct PermissionType;
}

diesel::table! {
    artifact (artifact_id) {
        artifact_id -> Int4,
        file_display_name -> Text,
        file_path -> Text,
        upload_date -> Date,
        uploaded_by -> Int4,
        is_temporary -> Bool,
    }
}

diesel::table! {
    estate (estate_id) {
        estate_id -> Int4,
        name -> Text,
        location -> Text,
        district -> Text,
        landmark -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::OwnershipState;

    ownership (ownership_id) {
        ownership_id -> Int4,
        user_id -> Int4,
        plot_id -> Int4,
        state -> OwnershipState,
        total_sale_value -> Int4,
        monthly_repayment_amount -> Int4,
        contract_start_date -> Nullable<Date>,
    }
}

diesel::table! {
    ownership_artifact (ownership_id, artifact_id) {
        ownership_id -> Int4,
        artifact_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PaymentState;

    payment (payment_id) {
        payment_id -> Int4,
        ownership_id -> Int4,
        date -> Date,
        amount -> Int4,
        state -> PaymentState,
        comment -> Text,
        reference -> Text,
        last_changed_by_user -> Int4,
        proof_of_payment_artifact -> Nullable<Int4>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PaymentState;

    payment_state_audit (audit_id) {
        audit_id -> Int4,
        payment_id -> Int4,
        previous_state -> PaymentState,
        new_state -> PaymentState,
        change_timestamp -> Timestamptz,
        changed_by_user -> Int4,
        comment -> Text,
    }
}

diesel::table! {
    plot (plot_id) {
        plot_id -> Int4,
        estate_id -> Int4,
        plot_number -> Text,
        phase -> Text,
        length -> Int4,
        width -> Int4,
    }
}

diesel::table! {
    user (user_id) {
        user_id -> Int4,
        user_name -> Text,
        name -> Text,
        cellphone_number -> Nullable<Text>,
        email_address -> Nullable<Text>,
        identification_number -> Nullable<Text>,
        password_hash -> Text,
        is_employee -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PermissionType;

    user_permission (user_id, permission) {
        user_id -> Int4,
        permission -> PermissionType,
    }
}

diesel::joinable!(artifact -> user (uploaded_by));
diesel::joinable!(ownership -> plot (plot_id));
diesel::joinable!(ownership -> user (user_id));
diesel::joinable!(ownership_artifact -> artifact (artifact_id));
diesel::joinable!(ownership_artifact -> ownership (ownership_id));
diesel::joinable!(payment -> artifact (proof_of_payment_artifact));
diesel::joinable!(payment -> ownership (ownership_id));
diesel::joinable!(payment -> user (last_changed_by_user));
diesel::joinable!(payment_state_audit -> payment (payment_id));
diesel::joinable!(payment_state_audit -> user (changed_by_user));
diesel::joinable!(plot -> estate (estate_id));
diesel::joinable!(user_permission -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    artifact,
    estate,
    ownership,
    ownership_artifact,
    payment,
    payment_state_audit,
    plot,
    user,
    user_permission,
);
