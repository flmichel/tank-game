use serde::Deserialize;

use crate::remotes::RemoteInput;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum MessageToGame {
    RemoteInput(RemoteInput),
    RoomId(String),
}
