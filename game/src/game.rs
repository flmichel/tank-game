use futures_channel::mpsc::UnboundedSender;
use specs::{Component, VecStorage};
use specs_derive::Component;
use webrtc::data_channel::RTCDataChannel;
use std::sync::Arc;

use crate::remotes::PlayerInput;

pub enum MessageToGame {
    PlayerInput(PlayerInput),
    RoomId(RoomId),
    SynchronizePlayer(SynchronizePlayer)
}


#[derive(Component)]
#[storage(VecStorage)]

pub struct SynchronizePlayer {
    pub socket_id: u32,
    pub player_id: String,
    pub sender_to_remote: UnboundedSender<MessageToRemote>,
    pub data_channel: Arc<RTCDataChannel>
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct RoomId(pub String);

#[derive(serde::Serialize)]
pub enum MessageToRemote {
    GameOver,
    GameStart,
    PlayerDied,
}
