use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, SinkExt, StreamExt};

use serde::{Deserialize, Serialize};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::Sender,
    sync::Mutex,
};
use tokio_tungstenite::tungstenite::Message;

pub type Tx = UnboundedSender<SdpOffer>;
pub type RoomMap = Arc<Mutex<HashMap<String, Tx>>>;

type RequestMap = Arc<std::sync::Mutex<HashMap<u32, Sender<String>>>>;

pub struct SdpOffer {
    pub offer: String,
    pub return_channel: Sender<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GameMessage {
    RoomId(String),
    SdpOffer(SdpMessage),
}

#[derive(Serialize, Deserialize)]
pub struct SdpMessage {
    pub data: String,
    pub id: u32,
}

pub async fn connect_to_game_instances(room_map: RoomMap) {
    let addr = "127.0.0.1:5000";

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(room_map.clone(), stream, addr));
    }
}

async fn handle_connection(room_map: RoomMap, raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let mut ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    let id = generate_id();
    let (tx, rx) = unbounded();
    room_map.lock().await.insert(id.clone(), tx);

    println!(
        "message sent: {}",
        serde_json::to_string(&GameMessage::RoomId(id.clone())).unwrap()
    );

    ws_stream
        .send(Message::text(
            serde_json::to_string(&GameMessage::RoomId(id.clone())).unwrap(),
        ))
        .await
        .unwrap();

    let (outgoing, incoming) = ws_stream.split();

    let request_map = RequestMap::new(std::sync::Mutex::new(HashMap::new()));
    let mut request_counter = 0;

    let send_sdp_answer = incoming.try_for_each(|msg| {
        println!(
            "Received a message from {}: {}",
            addr,
            msg.to_text().unwrap()
        );
        let sdp_answer = msg.to_text().unwrap();
        let sdp_answer: SdpMessage = serde_json::from_str(&sdp_answer).unwrap();
        print!("hellooooo");
        request_map
            .lock()
            .unwrap()
            .get(&sdp_answer.id)
            .unwrap()
            .try_send(sdp_answer.data)
            .unwrap();
        future::ok(())
    });

    let receive_sdp_offers = rx
        .map(|request| {
            request_map
                .lock()
                .unwrap()
                .insert(request_counter, request.return_channel);
            let offer_message = SdpMessage {
                data: request.offer,
                id: request_counter,
            };
            let offer_message = serde_json::to_string(&offer_message).unwrap();
            request_counter += 1;
            Ok(Message::text(offer_message))
        })
        .forward(outgoing);

    pin_mut!(send_sdp_answer, receive_sdp_offers);
    future::select(send_sdp_answer, receive_sdp_offers).await;

    println!("{} disconnected", &addr);
    room_map.lock().await.remove(&id);
}

fn generate_id() -> String {
    let random_bytes = rand::random::<[u8; 16]>().to_vec();
    let id = base64_url::encode(&random_bytes);
    print!("{}", id);
    return id;
}
