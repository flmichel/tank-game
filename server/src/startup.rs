use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use crate::{
    configuration::DatabaseSettings,
    result::{Error, ErrorKind::*},
};

pub async fn get_database_pool(settings: &DatabaseSettings) -> Result<Pool<Postgres>, Error> {
    PgPoolOptions::new()
        .connect(&settings.get_connection_string())
        .await
        .map_err(|err| Error::from(err, ConfigurationError).explain("fail to obtain database pool"))
}
