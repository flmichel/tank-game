use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use std::sync::Arc;
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors,
        media_engine::{MediaEngine, MIME_TYPE_OPUS},
        APIBuilder, API,
    },
    data_channel::{data_channel_message::DataChannelMessage, RTCDataChannel},
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription, RTCPeerConnection,
    },
    rtp_transceiver::rtp_codec::{RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType},
};

use crate::{
    game::MessageToGame,
    server_communicator::{MessageToServer, SdpMessage},
    signal::{decode, encode},
};

pub struct PlayersConnector {
    sender_to_server: UnboundedSender<MessageToServer>,
    sender_to_game: UnboundedSender<MessageToGame>,
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
    ) -> RTCPeerConnection {
        let peer_connection = self
            .api
            .new_peer_connection(self.configuration.clone())
            .await
            .expect("peer connection creation failed");

        peer_connection.on_peer_connection_state_change(Box::new(
            move |s: RTCPeerConnectionState| {
                println!("Peer Connection State has changed: {s}");

                if s == RTCPeerConnectionState::Failed {
                    println!("Peer Connection has gone to failed exiting");
                }

                Box::pin(async {})
            },
        ));

        peer_connection.on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
            let d_label = d.label().to_owned();
            let d_id = d.id();
            println!("New DataChannel {d_label} {d_id}");

            let sender_to_game = sender_to_game.clone();
            // Register channel opening handling
            Box::pin(async move {
                // Register text message handling
                d.on_message(Box::new(move |msg: DataChannelMessage| {
                    let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                    println!("Message from DataChannel '{d_label}': '{msg_str}'");
                    let message: String = serde_json::from_str(&msg_str).unwrap();
                    sender_to_game
                        .unbounded_send(MessageToGame::RoomId(message))
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
            receiver,
            webrtc_util: WebRTCUtil { api, configuration },
        }
    }

    pub async fn start(&mut self) {
        loop {
            if let Ok(Some(offer)) = self.receiver.try_next() {
                println!("got an offer in players_connector");
                let peer_connection = self
                    .webrtc_util
                    .start_peer_connection(offer.data, self.sender_to_game.clone())
                    .await;
                let answer = self.webrtc_util.create_answer(&peer_connection).await;
                println!("got an answer in players_connector");
                let answer = MessageToServer::SdpAnswer(SdpMessage {
                    data: encode(&answer),
                    id: offer.id,
                });

                println!("try to send the answer back to the server_communicator");
                match self.sender_to_server.unbounded_send(answer) {
                    Ok(()) => println!("The message was successfully sent"),
                    Err(err) => println!("failed to send message {}", err.to_string()),
                }
            }
        }
    }
}
