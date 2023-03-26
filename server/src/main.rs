mod authentication;
mod games_communication;
mod repository;
mod route;
use axum::{
    routing::{get, post},
    Extension, Router,
};

use games_communication::{connect_to_game_instances, RoomMap};
use repository::{Account, Repo};
use route::login::*;
use route::post_sdp_session;
use route::user::registration::*;
use sqlx::postgres::PgPoolOptions;
use std::{collections::HashMap, net::SocketAddr};
use tokio::{sync::Mutex, task};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let room_map = RoomMap::new(Mutex::new(HashMap::new()));

    let room_map_clone = room_map.clone();
    task::spawn(async move {
        connect_to_game_instances(room_map_clone).await;
    });

    let database_url = String::from("postgres://postgres:password@localhost:5432/tank-game");
    //std::env::var("DATABASE_URL").expect("set DATABASE_URL environment variable");
    let db = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("unable to connect to database");

    let account_repo = Repo::<Account>::new(db);

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
