use anyhow::{Context, Result};
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::StreamExt;
use game_core::{
    protocol::{MessageToServer, Ping, Pong, SdpMessage},
    remotes::RemoteInput,
    MessageToGame,
};
use std::sync::Arc;
use tracing::{debug, error, trace, warn};
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors,
        media_engine::{MediaEngine, MIME_TYPE_OPUS},
        APIBuilder, API,
    },
    data_channel::{data_channel_init::RTCDataChannelInit, data_channel_message::DataChannelMessage, RTCDataChannel},
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, math_rand_alpha, peer_connection_state::RTCPeerConnectionState, sdp::session_description::RTCSessionDescription, RTCPeerConnection
    },
    rtp_transceiver::rtp_codec::{RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType},
};

use crate::signal::{decode, encode};

/// A remote's data channel, handed to the main loop so it can deliver the game's outbox.
pub type DataChannelRegistration = (u32, Arc<RTCDataChannel>);

pub struct PlayersConnector {
    sender_to_server: UnboundedSender<MessageToServer>,
    sender_to_game: UnboundedSender<MessageToGame>,
    sender_data_channel: UnboundedSender<DataChannelRegistration>,
    receiver: UnboundedReceiver<SdpMessage>,
    webrtc_util: Arc<WebRTCUtil>,
}

struct WebRTCUtil {
    api: API,
    configuration: RTCConfiguration,
}

impl WebRTCUtil {
    async fn start_peer_connection(
        &self,
        offer: String,
        sender_to_game: UnboundedSender<MessageToGame>,
        sender_data_channel: UnboundedSender<DataChannelRegistration>,
        socket_id: u32,
    ) -> Result<RTCPeerConnection> {
        let offer = serde_json::from_str::<RTCSessionDescription>(&decode(&offer)?)?;
        let peer_connection = self
            .api
            .new_peer_connection(self.configuration.clone())
            .await?;

        peer_connection.on_peer_connection_state_change(Box::new(
            move |s: RTCPeerConnectionState| {
                debug!("Peer Connection State with socketId \"{socket_id}\" has changed: {s}.");

                if s == RTCPeerConnectionState::Failed {
                    debug!(
                        "Peer Connection with socketId \"{socket_id}\" has gone to failed exiting."
                    );
                }

                Box::pin(async {})
            },
        ));

        peer_connection.on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
            let d_label = d.label().to_owned();
            let d_id = d.id();
            debug!("New DataChannel {d_label} {d_id} with socket id \"{socket_id}\".");

            let sender_to_game = sender_to_game.clone();
            // The game's outbox goes over the reliable "channel"; others (e.g. "inputs") are inbound only.
            if d_label == "channel" {
                sender_data_channel
                    .unbounded_send((socket_id, Arc::clone(&d)))
                    .unwrap();
            }

            Box::pin(async move {
                let d_label2 = d_label.clone();
                d.on_open(Box::new(move || {
                    debug!("Data channel '{d_label2}'-'{d_id}' open.");
                    Box::pin(async {})
                }));

                // Register text message handling
                let d_for_pongs = Arc::clone(&d);
                d.on_message(Box::new(move |msg: DataChannelMessage| {
                    let msg_str = String::from_utf8_lossy(&msg.data);
                    trace!("Message from DataChannel '{d_label}' {socket_id}: '{msg_str}'");
                    // Answered here, not by the game loop, so the remote measures the network alone
                    if let Ok(Ping { ping }) = serde_json::from_slice(&msg.data) {
                        let channel = Arc::clone(&d_for_pongs);
                        return Box::pin(async move {
                            let pong = serde_json::to_string(&Pong { pong: ping }).unwrap();
                            if let Err(err) = channel.send_text(pong).await {
                                warn!("Failed to answer a ping on socket id \"{socket_id}\": {err}");
                            }
                        });
                    }
                    match serde_json::from_slice::<RemoteInput>(&msg.data) {
                        Ok(remote_input) => sender_to_game
                            .unbounded_send(MessageToGame::from_remote(socket_id, remote_input))
                            .unwrap(),
                        Err(err) => warn!(
                            "Dropping invalid message from DataChannel '{d_label}' {socket_id}: {err}"
                        ),
                    }
                    Box::pin(async {})
                }));
            })
        }));

        peer_connection.set_remote_description(offer).await?;

        Ok(peer_connection)
    }

    async fn create_answer(&self, peer_connection: &RTCPeerConnection) -> Result<String> {
        let answer = peer_connection.create_answer(None).await?;

        // Create channel that is blocked until ICE Gathering is complete
        let mut gather_complete = peer_connection.gathering_complete_promise().await;

        // Sets the LocalDescription, and starts our UDP listeners
        peer_connection.set_local_description(answer).await?;

        // Block until ICE Gathering is complete, disabling trickle ICE
        // we do this because we only can exchange one signaling message
        // in a production application you should exchange ICE Candidates via OnICECandidate
        let _ = gather_complete.recv().await;

        let answer = peer_connection
            .local_description()
            .await
            .context("no local description")?;

        Ok(serde_json::to_string(&answer)?)
    }
}

impl PlayersConnector {
    pub fn new(
        sender_to_server: UnboundedSender<MessageToServer>,
        sender_to_game: UnboundedSender<MessageToGame>,
        receiver: UnboundedReceiver<SdpMessage>,
        sender_data_channel: UnboundedSender<DataChannelRegistration>,
    ) -> Self {
        let mut m = MediaEngine::default();
        let fastest_codec = RTCRtpCodecParameters {
            capability: RTCRtpCodecCapability {
                mime_type: MIME_TYPE_OPUS.to_owned(),
                clock_rate: 48000,
                channels: 2,
                sdp_fmtp_line: "minptime=10;useinbandfec=1".to_owned(),
                rtcp_feedback: vec![],
            },
            payload_type: 111,
            ..Default::default()
        };
        m.register_codec(fastest_codec, RTPCodecType::Audio)
            .expect("codecs not registered");
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut m)
            .expect("could not register interceptors");

        let api = APIBuilder::new()
            .with_media_engine(m)
            .with_interceptor_registry(registry)
            .build();

        let configuration = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        Self {
            sender_to_server,
            sender_to_game,
            sender_data_channel,
            receiver,
            webrtc_util: Arc::new(WebRTCUtil { api, configuration }),
        }
    }

    pub async fn start(&mut self) {
        let mut next_player_socket_id: u32 = 0;
        while let Some(offer) = self.receiver.next().await {
            debug!("Received an offer in players_connector.");
            let socket_id = next_player_socket_id;
            next_player_socket_id += 1;
            let webrtc_util = Arc::clone(&self.webrtc_util);
            let sender_to_game = self.sender_to_game.clone();
            let sender_data_channel = self.sender_data_channel.clone();
            let sender_to_server = self.sender_to_server.clone();
            // One task per offer, so a slow ICE gathering doesn't hold up other phones.
            tokio::spawn(async move {
                let answer = match webrtc_util
                    .start_peer_connection(offer.data, sender_to_game, sender_data_channel, socket_id)
                    .await
                {
                    Ok(peer_connection) => webrtc_util.create_answer(&peer_connection).await,
                    Err(err) => Err(err),
                };
                let answer = match answer {
                    Ok(answer) => answer,
                    Err(err) => {
                        error!("Failed to answer offer {}: {err:#}", offer.id);
                        return;
                    }
                };
                debug!("Received an answer in players_connector.");
                let answer = MessageToServer::SdpAnswer(SdpMessage {
                    data: encode(&answer),
                    id: offer.id,
                });

                debug!("Try to send the answer back to the server_communicator.");
                match sender_to_server.unbounded_send(answer) {
                    Ok(()) => debug!("The message was successfully sent."),
                    Err(err) => debug!("failed to send message {}", err.to_string()),
                }
            });
        }
    }
}
