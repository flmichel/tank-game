use crate::{
    configuration::ApplicationSettings,
    result::{
        Error,
        ErrorKind::{ConfigurationError, NetworkError, ParsingError},
    },
};

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{
    future::{self, Ready},
    pin_mut,
    stream::TryStreamExt,
    SinkExt, StreamExt,
};
use serde::{de, Deserialize, Serialize};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::Sender,
    sync::Mutex,
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{debug, error, info};

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
    let game_path = settings.get_game_path();
    // Create the event loop and TCP listener we'll accept connections on.
    let listener = TcpListener::bind(&game_path).await.map_err(|err| {
        Error::from(err, ConfigurationError).explain("failed to create tcp listener")
    })?;

    info!("Starting listening for game room at address {}", game_path);

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

    debug!(
        "Incoming TCP connection from the address {}",
        game_room_address
    );

    let mut ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    debug!(
        "WebSocket connection established with the address: {}",
        game_room_address
    );

    let (message_sender, message_receiver) = unbounded();
    room_map.lock().await.insert(id.clone(), message_sender);

    if let Err(err) = send_id_to_game_room(&mut ws_stream, id.clone()).await {
        error!(
            "failed to send id to game room with address {} ({:?}), stopping websocket connection",
            game_room_address, err
        );
        return;
    }

    let (outgoing, incoming) = ws_stream.split();

    let request_map = RequestMap::new(std::sync::Mutex::new(HashMap::new()));
    let mut request_counter = 0;

    let send_sdp_answer = incoming.try_for_each(|message| {
        handle_game_message(message, game_room_address, request_map.clone())
    });

    let receive_sdp_offers = message_receiver
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

    debug!(
        "the room {} from the address {} was disconnected",
        &id, &game_room_address
    );
    room_map.lock().await.remove(&id);
}

fn generate_id() -> String {
    let random_bytes = rand::random::<[u8; 16]>().to_vec();
    let id = base64_url::encode(&random_bytes);
    return id;
}

async fn send_id_to_game_room(
    ws_stream: &mut WebSocketStream<TcpStream>,
    id: String,
) -> Result<(), Error> {
    debug!("sending message to game room containing the id: {}", &id);
    ws_stream
        .send(Message::text(
            serde_json::to_string(&GameMessage::RoomId(id)).map_err(|err| {
                Error::from(err, ParsingError).explain("failed to parse the Game Message")
            })?,
        ))
        .await
        .map_err(|err| {
            Error::from(err, NetworkError).explain("failed send id message to the game room")
        })
}

fn parse_message<'a, T>(message: &'a Message) -> Result<T, Error>
where
    T: de::Deserialize<'a>,
{
    let message = message.to_text().map_err(|err| {
        Error::from(err, ParsingError)
            .explain(format!("failed to convert message {} to string", message))
    })?;
    serde_json::from_str(message).map_err(|err| {
        Error::from(err, ParsingError).explain(format!(
            "failed to convert message {} to some struct",
            message
        ))
    })
}

fn handle_game_message<E>(
    message: Message,
    game_room_address: SocketAddr,
    request_map: RequestMap,
) -> Ready<Result<(), E>> {
    debug!(
        "Received a sdp answer from {}: {}",
        game_room_address, message
    );
    let sdp_answer: SdpMessage = parse_message(&message).unwrap();
    request_map
        .lock()
        .unwrap()
        .get(&sdp_answer.id)
        .unwrap()
        .try_send(sdp_answer.data)
        .unwrap();
    future::ok(())
}
