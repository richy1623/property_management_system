use crate::model::user::User;

tonic::include_proto!("property_management_system.property_management");

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

impl From<&crate::model::plot::Plot> for Plot {
    fn from(plot: &crate::model::plot::Plot) -> Self {
        Plot {
            plot_id: plot.plot_id,
            estate_id: plot.estate_id,
            plot_number: plot.plot_number.clone(),
            phase: plot.phase.clone(),
            length: plot.length,
            width: plot.width,
        }
    }
}

impl Ownership {
    pub fn from_model_and_user(
        ownership: &crate::model::ownership::Ownership,
        user: &User,
    ) -> Self {
        Ownership {
            ownership_id: ownership.ownership_id,
            user_id: ownership.user_id,
            user_name: user.user_name.clone(),
            plot_id: ownership.plot_id.clone(),
            state: ownership.state.clone().into(),
            total_sale_value: ownership.total_sale_value,
            monthly_repayment_amount: ownership.monthly_repayment_amount,
        }
    }
}
