tonic::include_proto!("property_management_system.estate_management");

impl From<crate::model::estate::Estate> for Estate {
    fn from(estate: crate::model::estate::Estate) -> Self {
        Estate {
            estate_id: estate.estate_id,
            name: estate.name,
            location: estate.location,
            district: estate.district,
            landmark: estate.landmark,
        }
    }
}
