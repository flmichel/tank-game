use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use std::time::Duration;
use tokio::{sync::mpsc, time::timeout};
use tracing::debug;

use crate::games_api::{RoomMap, SdpOffer};

const ANSWER_TIMEOUT: Duration = Duration::from_millis(if cfg!(test) { 100 } else { 10_000 });

pub async fn post_sdp_session(
    State(room_map): State<RoomMap>,
    Path(id): Path<String>,
    body: String,
) -> impl IntoResponse {
    let body = body.replace("\"", "");
    debug!("received spd offer for room id {}", id);

    // Clone the sender out so the room map isn't locked while waiting for the game.
    let Some(tx_game) = room_map.lock().await.get(&id).cloned() else {
        return (StatusCode::NOT_FOUND, "room not found".to_owned());
    };
    let (tx, mut rx) = mpsc::channel(1);
    let request = SdpOffer {
        offer: body,
        return_channel: tx,
    };
    debug!("send offer to game");
    if tx_game.unbounded_send(request).is_err() {
        return (StatusCode::NOT_FOUND, "room closed".to_owned());
    }
    match timeout(ANSWER_TIMEOUT, rx.recv()).await {
        Ok(Some(description)) => (StatusCode::OK, serde_json::to_string(&description).unwrap()),
        Ok(None) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "problem with channel".to_owned(),
        ),
        Err(_) => (
            StatusCode::GATEWAY_TIMEOUT,
            "the game did not answer in time".to_owned(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn unanswered_offer_times_out_without_locking_the_room_map() {
        let room_map = RoomMap::default();
        let (tx_game, mut rx_game) = futures_channel::mpsc::unbounded();
        room_map.lock().await.insert("room".to_owned(), tx_game);

        let (response, _offer) = tokio::join!(
            post_sdp_session(State(room_map.clone()), Path("room".to_owned()), "offer".to_owned()),
            async {
                let offer = rx_game.next().await.unwrap();
                assert!(room_map.try_lock().is_ok(), "room map locked while waiting for the game");
                offer // keep the return channel open: the game never answers
            }
        );

        assert_eq!(response.into_response().status(), StatusCode::GATEWAY_TIMEOUT);
    }
}
