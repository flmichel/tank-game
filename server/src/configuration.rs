use std::str::FromStr;

use crate::result::{Error, ErrorKind::*};
use config;
use tracing::Level;

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub log: Option<Log>,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    host: String,
    port: u16,
    name: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    host: String,
    web_port: u16,
    games_port: u16,
}

#[derive(serde::Deserialize, Clone)]
pub struct Log {
    level: Option<String>,
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

impl Settings {
    pub fn get_log_level(&self) -> Result<Level, Error> {
        if let Some(level) = self.log.clone().and_then(|log| log.level) {
            Level::from_str(&level).map_err(|err| {
                Error::from(err, ConfigurationError)
                    .explain(format!("log level \"{}\" does not exist", level))
            })
        } else {
            Ok(Level::INFO)
        }
    }
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
