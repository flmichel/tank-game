use axum::{
    routing::{get, post},
    Extension, Router,
};

use server::games_api::{start_game_application, RoomMap};
use server::repository::{Account, Repo};
use server::route::login::*;
use server::route::post_sdp_session;
use server::route::user::registration::*;
use server::{configuration::get_configuration, startup::get_database_pool};
use std::{collections::HashMap, net::SocketAddr};
use tokio::{sync::Mutex, task};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let settings = get_configuration().expect("configuration error");

    let database_pool = get_database_pool(&settings.database)
        .await
        .expect("database connection error");

    let room_map = RoomMap::new(Mutex::new(HashMap::new()));

    let room_map_clone = room_map.clone();
    task::spawn(async move {
        start_game_application(room_map_clone, &settings.application)
            .await
            .expect("could not start websocket");
    });

    let account_repo = Repo::<Account>::new(database_pool);

    let app = Router::new()
        .route("/login", post(login))
        .route("/user/registration", post(create_user))
        .route("/user/{name}", get(is_name_available))
        .route("/user/email/{email}}", get(is_email_available))
        .route("/game/:id", post(post_sdp_session))
        .layer(CorsLayer::permissive())
        .layer(Extension(account_repo))
        .with_state(room_map.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to start server");
}
