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
        person_id -> Int4,
        plot_id -> Int4,
        state -> OwnershipState,
        total_sale_value -> Int4,
        monthly_repayment_amount -> Int4,
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
        proof_of_payment_path -> Text,
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
        change_timestamp -> Timestamp,
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
        nrc -> Nullable<Text>,
        password_hash -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PermissionType;

    user_permissions (user_id, permission) {
        user_id -> Int4,
        permission -> PermissionType,
    }
}

diesel::joinable!(ownership -> plot (plot_id));
diesel::joinable!(ownership -> user (person_id));
diesel::joinable!(payment -> ownership (ownership_id));
diesel::joinable!(payment -> user (last_changed_by_user));
diesel::joinable!(payment_state_audit -> payment (payment_id));
diesel::joinable!(payment_state_audit -> user (changed_by_user));
diesel::joinable!(plot -> estate (estate_id));
diesel::joinable!(user_permissions -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    estate,
    ownership,
    payment,
    payment_state_audit,
    plot,
    user,
    user_permissions,
);
