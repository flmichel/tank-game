use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::{future, pin_mut, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::game::MessageToGame;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
pub enum ServerMessage {
    RoomId(String),
    SdpOffer(SdpMessage),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct SdpMessage {
    pub id: u32,
    pub data: String,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum MessageToServer {
    SdpAnswer(SdpMessage),
}

pub struct ServerCommunicator {
    sender_to_game: UnboundedSender<MessageToGame>,
    sender_to_player_connector: UnboundedSender<SdpMessage>,
    url: String,
}

impl ServerCommunicator {
    pub fn new<S: Into<String>>(
        sender_to_game: UnboundedSender<MessageToGame>,
        sender_to_player_connector: UnboundedSender<SdpMessage>,
        url: S,
    ) -> Self {
        Self {
            sender_to_game,
            sender_to_player_connector,
            url: url.into(),
        }
    }

    pub async fn start(self, receiver: UnboundedReceiver<MessageToServer>) {
        let (ws_stream, _) = connect_async(&self.url).await.expect("Failed to connect");

        println!("WebSocket handshake has been successfully completed");

        let (write, read) = ws_stream.split();

        let handle_server_messages = read.for_each(|message| async {
            print!("receive message from server: {:?}", message);
            let message = message.unwrap();
            let message: ServerMessage = serde_json::from_str(&message.to_text().unwrap()).unwrap();
            match message {
                ServerMessage::SdpOffer(sdp_offer) => self
                    .sender_to_player_connector
                    .clone()
                    .start_send(sdp_offer)
                    .unwrap(),
                ServerMessage::RoomId(room_id) => self
                    .sender_to_game
                    .clone()
                    .start_send(MessageToGame::RoomId(room_id))
                    .unwrap(),
            }
        });

        let handle_answer = receiver
            .map(|answer| {
                println!("answer received by server_communicator");
                Ok(Message::text(serde_json::to_string(&answer).unwrap()))
            })
            .forward(write);

        pin_mut!(handle_server_messages, handle_answer);
        future::select(handle_server_messages, handle_answer).await;
    }
}
