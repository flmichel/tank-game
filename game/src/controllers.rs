use anyhow::Result;
use std::{io, sync::Arc, time::Duration};

use actix::{
    Actor, Addr, Arbiter, AsyncContext, Context, Handler, Message, ResponseFuture, WrapFuture,
};
use tokio::sync::mpsc::Sender;
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine, APIBuilder,
        API,
    },
    data_channel::{data_channel_message::DataChannelMessage, RTCDataChannel},
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{
        self, configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription, RTCPeerConnection,
    },
};

pub struct WebRTCData {
    api: API,
    configuration: RTCConfiguration,
}

impl WebRTCData {
    pub fn new() -> Self {
        // Create a MediaEngine object to configure the supported codec
        let mut m = MediaEngine::default();

        // Register default codecs
        m.register_default_codecs().expect("codecs not registered");

        // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
        // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
        // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
        // for each PeerConnection.
        let mut registry = Registry::new();

        // Use the default set of Interceptors
        registry = register_default_interceptors(registry, &mut m)
            .expect("could not register interceptors");

        // Create the API object with the MediaEngine
        let api = APIBuilder::new()
            .with_media_engine(m)
            .with_interceptor_registry(registry)
            .build();

        // Prepare the configuration
        let configuration = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        Self { api, configuration }
    }

    pub async fn create_answer(&self, peer_connection: &RTCPeerConnection) -> String {
        // Create an answer
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

pub async fn start_peer_connection(offer: String, sender: Sender<String>) {
    let data = WebRTCData::new();
    let offer = serde_json::from_str::<RTCSessionDescription>(offer.as_str())
        .expect("failed to retrieve session description");

    let peer_connection = data
        .api
        .new_peer_connection(data.configuration.clone())
        .await
        .expect("peer connection creation failed");

    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        println!("Peer Connection State has changed: {s}");

        if s == RTCPeerConnectionState::Failed {
            println!("Peer Connection has gone to failed exiting");
        }

        Box::pin(async {})
    }));

    peer_connection
        .on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
            let d_label = d.label().to_owned();
            let d_id = d.id();
            println!("New DataChannel {d_label} {d_id}");

            // Register channel opening handling
            Box::pin(async move {
                let d2 = Arc::clone(&d);
                let d_label2 = d_label.clone();
                let d_id2 = d_id;
                d.on_open(Box::new(move || {
                    println!("Data channel '{d_label2}'-'{d_id2}' open. Random messages will now be sent to any connected DataChannels every 5 seconds");

                    Box::pin(async move {
                        let mut result = Result::<usize>::Ok(0);
                        while result.is_ok() {
                            let timeout = tokio::time::sleep(Duration::from_secs(5));
                            tokio::pin!(timeout);

                            tokio::select! {
                                _ = timeout.as_mut() =>{
                                    let message = "hahahah".to_owned();
                                    println!("Sending '{message}'");
                                    result = d2.send_text(message).await.map_err(Into::into);
                                }
                            };
                        }
                    })
                }));

                // Register text message handling
                d.on_message(Box::new(move |msg: DataChannelMessage| {
                    let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                    println!("Message from DataChannel '{d_label}': '{msg_str}'");
                    Box::pin(async {})
                }));
            })
        }));

    peer_connection
        .set_remote_description(offer)
        .await
        .expect("failed to set remote sessionDescription");

    let answer = data.create_answer(&peer_connection).await;
    println!("{}", answer);
    sender.send(answer).await.expect("failed to send answer");
}

pub fn listen(peer_connection: RTCPeerConnection, sender: Sender<String>) {
    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        println!("Peer Connection State has changed: {s}");

        if s == RTCPeerConnectionState::Failed {
            println!("Peer Connection has gone to failed exiting");
        }

        Box::pin(async {})
    }));

    peer_connection
        .on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
            let d_label = d.label().to_owned();
            let d_id = d.id();
            println!("New DataChannel {d_label} {d_id}");

            // Register channel opening handling
            Box::pin(async move {
                let d2 = Arc::clone(&d);
                let d_label2 = d_label.clone();
                let d_id2 = d_id;
                d.on_open(Box::new(move || {
                    println!("Data channel '{d_label2}'-'{d_id2}' open. Random messages will now be sent to any connected DataChannels every 5 seconds");

                    Box::pin(async move {
                        let mut result = Result::<usize>::Ok(0);
                        while result.is_ok() {
                            let timeout = tokio::time::sleep(Duration::from_secs(5));
                            tokio::pin!(timeout);

                            tokio::select! {
                                _ = timeout.as_mut() =>{
                                    let message = "hahahah".to_owned();
                                    println!("Sending '{message}'");
                                    result = d2.send_text(message).await.map_err(Into::into);
                                }
                            };
                        }
                    })
                }));

                // Register text message handling
                d.on_message(Box::new(move |msg: DataChannelMessage| {
                    let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                    println!("Message from DataChannel '{d_label}': '{msg_str}'");
                    Box::pin(async {})
                }));
            })
        }));
}
