use crate::{
    configuration::ApplicationSettings,
    result::{Error, ErrorKind::ConfigurationError},
};

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
use tracing::{event, span, Level};

pub type Tx = UnboundedSender<SdpOffer>;
pub type RoomMap = Arc<Mutex<HashMap<String, Tx>>>;

type RequestMap = Arc<std::sync::Mutex<HashMap<u32, Sender<String>>>>;

pub struct SdpOffer {
    pub offer: String,
    pub return_channel: Sender<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
pub enum GameMessage {
    RoomId(String),
    SdpOffer(SdpMessage),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SdpMessage {
    pub data: String,
    pub id: u32,
}

pub async fn start_game_application(
    room_map: RoomMap,
    settings: &ApplicationSettings,
) -> Result<(), Error> {
    // Create the event loop and TCP listener we'll accept connections on.
    let listener = TcpListener::bind(&settings.get_game_path())
        .await
        .map_err(|err| {
            Error::from(err, ConfigurationError).explain("failed to create tcp listener")
        })?;

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, game_room_address)) = listener.accept().await {
        tokio::spawn(handle_connection(
            room_map.clone(),
            stream,
            game_room_address,
        ));
    }
    Ok(())
}

async fn handle_connection(
    room_map: RoomMap,
    raw_stream: TcpStream,
    game_room_address: SocketAddr,
) {
    let id = generate_id();
    let _span = span!(Level::INFO, "game room", id);

    event!(
        Level::INFO,
        "Incoming TCP connection from the address {}",
        game_room_address
    );

    let mut ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!(
        "WebSocket connection established with the address: {}",
        game_room_address
    );

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
            "Received an answer from {}: {}",
            game_room_address,
            msg.to_text().unwrap()
        );
        let sdp_answer = msg.to_text().unwrap();
        let sdp_answer: SdpMessage = serde_json::from_str(sdp_answer).unwrap();
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
            println!("offer recieved by server communicator");
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

    println!("{} disconnected", &game_room_address);
    room_map.lock().await.remove(&id);
}

fn generate_id() -> String {
    let random_bytes = rand::random::<[u8; 16]>().to_vec();
    let id = base64_url::encode(&random_bytes);
    print!("{}", id);
    return id;
}
