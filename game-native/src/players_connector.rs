use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use game_core::{
    protocol::{MessageToServer, SdpMessage},
    remotes::RemoteInput,
    MessageToGame,
};
use std::sync::Arc;
use tracing::{debug, trace};
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
    webrtc_util: WebRTCUtil,
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
    ) -> RTCPeerConnection {
        let peer_connection = self
            .api
            .new_peer_connection(self.configuration.clone())
            .await
            .expect("peer connection creation failed");

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
            sender_data_channel
                .unbounded_send((socket_id, Arc::clone(&d)))
                .unwrap();

            Box::pin(async move {
                let d_label2 = d_label.clone();
                d.on_open(Box::new(move || {
                    debug!("Data channel '{d_label2}'-'{d_id}' open.");
                    Box::pin(async {})
                }));

                // Register text message handling
                d.on_message(Box::new(move |msg: DataChannelMessage| {
                    let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                    trace!("Message from DataChannel '{d_label}' {socket_id}: '{msg_str}'");
                    let remote_input: RemoteInput = serde_json::from_str(&msg_str).unwrap();

                    sender_to_game
                        .unbounded_send(MessageToGame::from_remote(socket_id, remote_input))
                        .unwrap();
                    Box::pin(async {})
                }));
            })
        }));

        let offer = serde_json::from_str::<RTCSessionDescription>(&decode(offer.as_str()).unwrap())
            .unwrap();
        peer_connection
            .set_remote_description(offer)
            .await
            .expect("failed to set remote sessionDescription");

        peer_connection
    }

    async fn create_answer(&self, peer_connection: &RTCPeerConnection) -> String {
        let answer = peer_connection
            .create_answer(None)
            .await
            .expect("failed to create answer");

        // Create channel that is blocked until ICE Gathering is complete
        let mut gather_complete = peer_connection.gathering_complete_promise().await;

        // Sets the LocalDescription, and starts our UDP listeners
        peer_connection
            .set_local_description(answer)
            .await
            .expect("failed to set local description");

        // Block until ICE Gathering is complete, disabling trickle ICE
        // we do this because we only can exchange one signaling message
        // in a production application you should exchange ICE Candidates via OnICECandidate
        let _ = gather_complete.recv().await;

        let answer = peer_connection
            .local_description()
            .await
            .expect("failed to get local description");

        serde_json::to_string(&answer).expect("failed to unmarshal")
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
            webrtc_util: WebRTCUtil { api, configuration },
        }
    }

    pub async fn start(&mut self) {
        let mut next_player_socket_id: u32 = 0;
        loop {
            if let Ok(Some(offer)) = self.receiver.try_next() {
                debug!("Received an offer in players_connector.");
                let peer_connection = self
                    .webrtc_util
                    .start_peer_connection(
                        offer.data,
                        self.sender_to_game.clone(),
                        self.sender_data_channel.clone(),
                        next_player_socket_id,
                    )
                    .await;
                next_player_socket_id += 1;
                let answer = self.webrtc_util.create_answer(&peer_connection).await;
                debug!("Received an answer in players_connector.");
                let answer = MessageToServer::SdpAnswer(SdpMessage {
                    data: encode(&answer),
                    id: offer.id,
                });

                debug!("Try to send the answer back to the server_communicator.");
                match self.sender_to_server.unbounded_send(answer) {
                    Ok(()) => debug!("The message was successfully sent."),
                    Err(err) => debug!("failed to send message {}", err.to_string()),
                }
            }
        }
    }
}
