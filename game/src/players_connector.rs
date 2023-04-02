use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::spawn;
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine, APIBuilder,
        API,
    },
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, sdp::session_description::RTCSessionDescription,
        RTCPeerConnection,
    },
};

use crate::{
    game::MessageToGame,
    remote_communicator::RemoteCommunicator,
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
    async fn start_peer_connection(&self, offer: String) -> RTCPeerConnection {
        let peer_connection = self
            .api
            .new_peer_connection(self.configuration.clone())
            .await
            .expect("peer connection creation failed");

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
        m.register_default_codecs().expect("codecs not registered");
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
                let peer_connection = self.webrtc_util.start_peer_connection(offer.data).await;
                let answer = self.webrtc_util.create_answer(&peer_connection).await;
                println!("got an answer in players_connector");
                let answer = MessageToServer::SdpAnswer(SdpMessage {
                    data: encode(&answer),
                    id: offer.id,
                });

                let sender_to_game = self.sender_to_game.clone();
                spawn(async move {
                    let mut remote_communicator =
                        RemoteCommunicator::new(peer_connection, sender_to_game);
                    remote_communicator.start().await
                });
                println!("try to send the answer back to the server_communicator");
                self.sender_to_server.unbounded_send(answer).unwrap();
            }
        }
    }
}
