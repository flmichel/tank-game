use crate::authentication::hash_is_correct;
use crate::repository::{Account, Repo};
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use serde::Deserialize;

pub async fn login(
    Extension(account_repo): Extension<Repo<Account>>,
    Json(payload): Json<Credentials>,
) -> impl IntoResponse {
    if !account_repo.email_exists(&payload.email).await {
        return StatusCode::NOT_FOUND;
    }

    let expected_password_hash = account_repo.get_password_hash(&payload.email).await;
    if !hash_is_correct(expected_password_hash, payload.password) {
        return StatusCode::UNAUTHORIZED;
    }
    StatusCode::OK
}

#[derive(Deserialize)]
pub struct Credentials {
    email: String,
    password: String,
}
