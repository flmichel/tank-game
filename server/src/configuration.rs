use crate::result::{Error, ErrorKind::*};
use config;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    host: String,
    port: u16,
    name: String,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    host: String,
    web_port: u16,
    games_port: u16,
}

pub fn get_configuration() -> Result<Settings, Error> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("configuration"))
        .build()
        .map_err(|err| {
            Error::from(err, ConfigurationError).explain("configuration file not found")
        })?;

    settings.try_deserialize::<Settings>().map_err(|err| {
        Error::from(err, ConfigurationError).explain("configuration could not be deserialized")
    })
}

impl DatabaseSettings {
    pub fn get_connection_string(&self) -> String {
        format!("{}:{}/{}", self.host, self.port, self.name)
    }
}

impl ApplicationSettings {
    pub fn get_game_path(&self) -> String {
        format!("{}:{}", self.host, self.games_port)
    }

    pub fn get_web_path(&self) -> String {
        format!("{}:{}", self.host, self.web_port)
    }
}
