// how to use GRPC: https://www.youtube.com/watch?v=JkSa-qA2jnY
tonic::include_proto!("property_management_system.user_management");

impl From<&crate::model::user::User> for User {
    fn from(user: &crate::model::user::User) -> Self {
        User {
            user_id: user.user_id,
            username: user.user_name.clone(),
            name: user.name.clone(),
            cellphone_number: user.cellphone_number.clone(),
            email_address: user.email_address.clone(),
            identification_number: user.identification_number.clone(),
        }
    }
}
