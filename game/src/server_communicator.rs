use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::{future, pin_mut, FutureExt, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
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
pub enum MessageToServer {
    SdpAnswer(SdpMessage),
}

pub struct ServerCommunicator {
    sender_to_game: UnboundedSender<MessageToGame>,
    sender_to_player_connector: UnboundedSender<SdpMessage>,
    receiver: UnboundedReceiver<MessageToServer>,
    url: String,
}

impl ServerCommunicator {
    /*pub fn builder() -> ServerCommunicatorBuilder {
        ServerCommunicatorBuilder::default()
    }*/

    pub fn new<S: Into<String>>(
        sender_to_game: UnboundedSender<MessageToGame>,
        sender_to_player_connector: UnboundedSender<SdpMessage>,
        receiver: UnboundedReceiver<MessageToServer>,
        url: S,
    ) -> Self {
        Self {
            sender_to_game,
            sender_to_player_connector,
            receiver,
            url: url.into(),
        }
    }

    pub async fn start(mut self) {
        let (mut ws_stream, _) = connect_async(&self.url).await.expect("Failed to connect");
        ws_stream.send(Message::text("lol")).await.unwrap();

        println!("WebSocket handshake has been successfully completed");

        let (write, read) = ws_stream.split();

        let handle_server_messages = read.for_each(|message| async {
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

        let handle_answer = self
            .receiver
            .map(|answer| {
                println!("answer received by server_communicator");
                Message::text(serde_json::to_string(&answer).unwrap())
            })
            .map(Ok)
            .forward(write);

        pin_mut!(handle_server_messages, handle_answer);
        future::select(handle_server_messages, handle_answer).await;
    }
}

/*
#[derive(Default)]
pub struct ServerCommunicatorBuilder {
    sender_to_game: Option<Sender<MessageToGame>>,
    sender_to_player_connector: Option<Sender<SdpMessage>>,
    receiver: Option<Receiver<MessageToServer>>,
    url: Option<String>,
}

impl ServerCommunicatorBuilder {
    pub fn sender_to_game(&mut self, mut sender_to_game: Sender<MessageToGame>) -> &mut Self {
        self.sender_to_game = Some(sender_to_game);
        self
    }

    pub fn sender_to_player_connector(
        &mut self,
        mut sender_to_player_connector: Sender<SdpMessage>,
    ) -> &mut Self {
        self.sender_to_player_connector = Some(sender_to_player_connector);
        self
    }

    pub fn receiver(&mut self, mut receiver: Receiver<MessageToServer>) -> &mut Self {
        self.receiver = Some(receiver);
        self
    }

    pub fn url<S: Into<String>>(&mut self, url: S) -> &mut Self {
        self.url = Some(url.into());
        self
    }

    pub fn build(&mut self) -> Option<ServerCommunicator> {
        Some(ServerCommunicator {
            sender_to_game: self.sender_to_game?,
            sender_to_player_connector: self.sender_to_player_connector?,
            receiver: self.receiver?,
            url: self.url?,
        })
    }
}
*/
