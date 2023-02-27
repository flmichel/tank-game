mod authentication;
mod repository;
mod route;

use axum::{
    routing::{get, post},
    Extension, Router,
};

use repository::{Account, Repo};
use route::login::*;
use route::user::registration::*;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

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
        .layer(CorsLayer::permissive())
        .layer(Extension(account_repo));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to start server");
}
