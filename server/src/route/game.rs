use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use serde::Deserialize;
use serde_json::to_string;
use tokio::sync::mpsc;

use crate::games_communication::{RoomMap, SdpOffer};

pub async fn post_sdp_session(
    State(room_map): State<RoomMap>,
    Path(id): Path<String>,
    Json(body): Json<String>,
) -> impl IntoResponse {
    println!("room id {}", id);
    room_map
        .lock()
        .await
        .keys()
        .for_each(|key| print!("{}", key));
    if let Some(tx_game) = room_map.lock().await.get(&id) {
        let (tx, mut rx) = mpsc::channel(1);
        let request = SdpOffer {
            offer: body,
            return_channel: tx.clone(),
        };
        tx_game.unbounded_send(request).unwrap();
        match rx.recv().await {
            Some(description) => (StatusCode::OK, description),
            None => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "problem with channel".to_owned(),
            ),
        }
    } else {
        (StatusCode::NOT_FOUND, "room not found".to_owned())
    }
}
