use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use tokio::sync::mpsc;
use tracing::debug;

use crate::games_api::{RoomMap, SdpOffer};

pub async fn post_sdp_session(
    State(room_map): State<RoomMap>,
    Path(id): Path<String>,
    body: String,
) -> impl IntoResponse {
    let body = body.replace("\"", "");
    debug!("received spd offer for room id {}", id);

    if let Some(tx_game) = room_map.lock().await.get(&id) {
        let (tx, mut rx) = mpsc::channel(1);
        let request = SdpOffer {
            offer: body,
            return_channel: tx.clone(),
        };
        debug!("send offer to game");
        tx_game.unbounded_send(request).unwrap();
        match rx.recv().await {
            Some(description) => (StatusCode::OK, serde_json::to_string(&description).unwrap()),
            None => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "problem with channel".to_owned(),
            ),
        }
    } else {
        (StatusCode::NOT_FOUND, "room not found".to_owned())
    }
}
