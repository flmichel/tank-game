use std::net::SocketAddr;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tower_http::cors::CorsLayer;
use tracing::{info, Level};

use crate::{
    configuration::{ApplicationSettings, DatabaseSettings},
    games_api::RoomMap,
    repository::{Account, Repo},
    result::{Error, ErrorKind::*},
    route::{
        login, post_sdp_session,
        registration::{create_user, is_email_available, is_name_available},
    },
};

pub fn setup_logger(log_level: Level) {
    tracing_subscriber::fmt().with_max_level(log_level).init();
    info!("Logging initialized with {} as maximum level", log_level);
}

pub async fn get_database_pool(settings: &DatabaseSettings) -> Result<Pool<Postgres>, Error> {
    PgPoolOptions::new()
        .connect(&settings.get_connection_string())
        .await
        .map_err(|err| Error::from(err, ConfigurationError).explain("fail to obtain database pool"))
}

pub async fn start_web_application(
    database_pool: Pool<Postgres>,
    room_map: RoomMap,
    settings: &ApplicationSettings,
) -> Result<(), Error> {
    let app = Router::new()
        .route("/login", post(login))
        .route("/user/registration", post(create_user))
        .route("/user/{name}", get(is_name_available))
        .route("/user/email/{email}}", get(is_email_available))
        .route("/game/:id", post(post_sdp_session))
        .layer(CorsLayer::permissive())
        .layer(Extension(Repo::<Account>::new(database_pool)))
        .with_state(room_map);

    let addr_str = settings.get_web_path();
    let addr = addr_str.parse::<SocketAddr>().map_err(|err| {
        Error::from(err, ConfigurationError).explain("Failed to parse the socket address")
    })?;

    info!("Start web application at address {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|err| Error::from(err, ConfigurationError).explain("failed to start serving"))
}
