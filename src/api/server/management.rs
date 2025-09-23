use std::sync::Arc;

use actix_web::{HttpResponse, Scope, web};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::AppServices;
use crate::api::handler::artifact_handler::get_artifact;
use crate::api::handler::estate_handler::{create_estate, list_estates};
use crate::api::handler::health_handler::health_check;
use crate::api::handler::ownership_handler::{
    create_ownership, get_ownership, get_ownerships, update_ownership,
};
use crate::api::handler::payment_handler::{
    create_payment, get_payments_for_ownership, get_payments_in_state, update_payment_state,
};
use crate::api::handler::plot_handler::{create_plot, list_plots};
use crate::api::handler::user_handler::{
    create_user, get_user_details, list_users, login_user, update_user_details,
};

pub fn api_routes(app_services: Arc<AppServices>) -> Scope {
    web::scope("/api")
        .app_data(actix_web::web::Data::new(app_services.clone()))
        .app_data(
            web::JsonConfig::default().limit(50 * 1024 * 1024), // 50 MB limit
        )
        // User routes
        .service(
            web::scope("/user")
                .route("/login", web::post().to(login_user))
                .route("/create", web::post().to(create_user))
                .route("/list", web::post().to(list_users))
                .route("/{user_id}", web::get().to(get_user_details))
                .route("/{user_id}", web::put().to(update_user_details)),
        )
        // Estate routes
        .service(
            web::scope("/estate")
                .route("/create", web::post().to(create_estate))
                .route("/list", web::get().to(list_estates)),
        )
        // Plot routes
        .service(
            web::scope("/plot")
                .route("/create", web::post().to(create_plot))
                .route("/list", web::post().to(list_plots)),
        )
        // Ownership routes
        .service(
            web::scope("/ownership")
                .route("/{ownership_id}", web::get().to(get_ownership))
                .route("/create", web::post().to(create_ownership))
                .route("/update", web::put().to(update_ownership)),
        )
        .service(web::scope("/ownerships").route("", web::get().to(get_ownerships)))
        // Payment routes
        .service(
            web::scope("/payment")
                .route("/", web::post().to(create_payment))
                .route("/ownership", web::get().to(get_payments_for_ownership))
                .route("/state", web::get().to(get_payments_in_state))
                .route("/update", web::post().to(update_payment_state)),
        )
        // Artifact routes
        .service(web::scope("/artifact").route("/{artifact_id}", web::get().to(get_artifact)))
        // Health check routes
        .service(web::scope("/health").route("", web::get().to(health_check)))
        // Serve Swagger UI
        .service(
            web::scope("/docs")
                .service(
                    SwaggerUi::new("ui/{_:.*}").url("/api/docs/openapi.json", Default::default()),
                )
                .route(
                    "/openapi.json",
                    web::get().to(|| async { HttpResponse::Ok().json(doc::ApiDoc::openapi()) }),
                ),
        )
    // Note: when adding paths here, add them below to include them in the swagger
}

pub mod doc {

    use crate::api::handler::{
        artifact_handler, estate_handler, health_handler, ownership_handler, payment_handler,
        plot_handler, user_handler,
    };

    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(
    info(title="Management API", description = "Management API"),
    paths(
        user_handler::login_user,
        user_handler::create_user,
        user_handler::list_users,
        user_handler::get_user_details,
        user_handler::update_user_details,
        estate_handler::create_estate,
        estate_handler::list_estates,
        plot_handler::create_plot,
        plot_handler::list_plots,
        ownership_handler::create_ownership,
        ownership_handler::update_ownership,
        ownership_handler::get_ownership,
        ownership_handler::get_ownerships,
        payment_handler::create_payment,
        payment_handler::get_payments_for_ownership,
        payment_handler::get_payments_in_state,
        payment_handler::update_payment_state,
        artifact_handler::get_artifact,
        health_handler::health_check
    ),
    tags(
        (name = "User Management", description = "Operations related to user authentication and management"),
        (name = "Property Management", description = "Operations related to estates, plots, and ownership"),
        (name = "Payment Management", description = "Operations related to payment tracking and updates"),
        (name = "Artifact Management", description = "Operations related to archival of artifacts"),
        (name = "Health Check", description = "Operations related to health of the service")
    )
)]
    pub struct ApiDoc;
}
