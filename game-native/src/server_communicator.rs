use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::{future, pin_mut, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error};

use game_core::{
    protocol::{MessageToServer, SdpMessage, ServerMessage},
    MessageToGame, RoomId,
};

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

        debug!("WebSocket handshake has been successfully completed.");

        let (write, read) = ws_stream.split();

        let handle_server_messages = read.for_each(|message| async {
            debug!("Received message from server: {:?}.", message);
            let message = match parse_server_message(message) {
                Ok(message) => message,
                Err(err) => {
                    error!("Dropping invalid message from server: {err}");
                    return;
                }
            };
            match message {
                ServerMessage::SdpOffer(sdp_offer) => self
                    .sender_to_player_connector
                    .clone()
                    .start_send(sdp_offer)
                    .unwrap(),
                ServerMessage::RoomId(room_id) => self
                    .sender_to_game
                    .clone()
                    .start_send(MessageToGame::RoomId(RoomId(room_id)))
                    .unwrap(),
            }
        });

        let handle_answer = receiver
            .map(|answer| {
                debug!("Received answer from server_communicator.");
                Ok(Message::text(serde_json::to_string(&answer).unwrap()))
            })
            .forward(write);

        pin_mut!(handle_server_messages, handle_answer);
        future::select(handle_server_messages, handle_answer).await;
    }
}

fn parse_server_message(
    message: Result<Message, tokio_tungstenite::tungstenite::Error>,
) -> anyhow::Result<ServerMessage> {
    Ok(serde_json::from_str(message?.to_text()?)?)
}
