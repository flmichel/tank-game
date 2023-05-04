use server::startup::setup_logger;
use server::{configuration::get_configuration, startup::get_database_pool};
use server::{
    games_api::{start_game_application, RoomMap},
    startup::start_web_application,
};
use std::collections::HashMap;
use tokio::{sync::Mutex, task};

#[tokio::main]
async fn main() {
    let settings = get_configuration().expect("configuration error");

    let log_level = settings
        .get_log_level()
        .expect("failed to get log level from settings");
    setup_logger(log_level);

    let database_pool = get_database_pool(&settings.database.clone())
        .await
        .expect("database connection error");

    let room_map = RoomMap::new(Mutex::new(HashMap::new()));

    let room_map_clone = room_map.clone();
    let application_settings_clone = settings.application.clone();
    task::spawn(async move {
        start_game_application(room_map_clone, &application_settings_clone)
            .await
            .expect("could not start websocket");
    });

    start_web_application(database_pool, room_map, &settings.application)
        .await
        .expect("failed to start server");
}
