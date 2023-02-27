use super::{Repo, RepoKind};
use sqlx::query;

#[derive(Clone)]
pub struct Account;
impl RepoKind for Account {}

pub struct AccountData {
    pub name: String,
    pub email: String,
    pub password_hash: String,
}

impl Repo<Account> {
    pub async fn name_exists(&self, name: &str) -> bool {
        query!(
            r#"SELECT EXISTS(SELECT 1 FROM account WHERE name=$1)"#,
            name
        )
        .fetch_one(&self.poll)
        .await
        .unwrap()
        .exists
        .unwrap()
    }

    pub async fn email_exists(&self, email: &str) -> bool {
        query!(
            r#"SELECT EXISTS(SELECT 1 FROM account WHERE email=$1)"#,
            email
        )
        .fetch_one(&self.poll)
        .await
        .unwrap()
        .exists
        .unwrap()
    }

    pub async fn add(&self, account_data: AccountData) {
        query!(
            r#"INSERT INTO account (name, email, password_hash) VALUES ($1, $2, $3)"#,
            account_data.name,
            account_data.email,
            account_data.password_hash,
        )
        .fetch_all(&self.poll)
        .await
        .unwrap();
    }

    pub async fn get_password_hash(&self, email: &str) -> String {
        query!(r#"SELECT password_hash FROM account WHERE email=$1"#, email)
            .fetch_one(&self.poll)
            .await
            .unwrap()
            .password_hash
    }
}
