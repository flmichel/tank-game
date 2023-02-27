use crate::authentication::hash_password;
use crate::repository::{Account, AccountData, Repo};
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use serde::Deserialize;

pub async fn create_user(
    Extension(account_repo): Extension<Repo<Account>>,
    Json(payload): Json<CreateUser>,
) -> impl IntoResponse {
    if account_repo.name_exists(&payload.name).await {
        return StatusCode::FORBIDDEN;
    }

    if account_repo.email_exists(&payload.name).await {
        return StatusCode::FORBIDDEN;
    }

    account_repo
        .add(AccountData {
            name: payload.name,
            email: payload.email,
            password_hash: hash_password(payload.password),
        })
        .await;
    StatusCode::CREATED
}

#[derive(Deserialize)]
pub struct CreateUser {
    email: String,
    name: String,
    password: String,
}

pub async fn is_name_available(
    Extension(account_repo): Extension<Repo<Account>>,
    Json(name): Json<String>,
) -> impl IntoResponse {
    if account_repo.name_exists(&name).await {
        return StatusCode::NOT_ACCEPTABLE;
    }
    StatusCode::OK
}

pub async fn is_email_available(
    Extension(account_repo): Extension<Repo<Account>>,
    Json(name): Json<String>,
) -> impl IntoResponse {
    if account_repo.email_exists(&name).await {
        return StatusCode::NOT_ACCEPTABLE;
    }
    StatusCode::OK
}
