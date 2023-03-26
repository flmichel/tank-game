use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use serde::{Deserialize, Serialize};

use futures_util::{future, pin_mut, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerMessage {
    RoomId(String),
    SdpOffer(SdpMessage),
}

#[derive(Serialize, Deserialize)]
pub struct SdpMessage {
    id: u32,
    data: String,
}

pub async fn connect_to_server(
    tx_server: UnboundedSender<ServerMessage>,
    rx_game: &mut UnboundedReceiver<SdpMessage>,
) {
    let (stdin_tx, stdin_rx) = tokio::sync::mpsc::channel::<Message>(1000);

    let (ws_stream, _) = connect_async("ws://localhost:5000")
        .await
        .expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();

    let handle_server_messages = read.for_each(|message| async {
        print!("message: {:?}", message);
        let message: ServerMessage =
            serde_json::from_str(&message.unwrap().to_text().unwrap()).unwrap();
        tx_server.unbounded_send(message).unwrap();
    });

    let handle_answer = rx_game
        .map(|answer| Message::text(serde_json::to_string(&answer).unwrap()))
        .map(Ok)
        .forward(write);

    pin_mut!(handle_server_messages, handle_answer);
    future::select(handle_server_messages, handle_answer).await;
}
