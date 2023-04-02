use anyhow::Result;
use std::{time::Duration, sync::Arc};

use futures_channel::mpsc::UnboundedSender;
use webrtc::{peer_connection::{RTCPeerConnection, peer_connection_state::RTCPeerConnectionState}, data_channel::{data_channel_message::DataChannelMessage, RTCDataChannel}};

use crate::game::MessageToGame;

pub struct RemoteCommunicator {
    peer_connection: RTCPeerConnection,
    sender_to_game: UnboundedSender<MessageToGame>,
}

impl RemoteCommunicator {
    pub fn new(peer_connection: RTCPeerConnection, sender_to_game: UnboundedSender<MessageToGame>) -> Self {
        print!("new player could connect");
        Self {
            peer_connection,
            sender_to_game,
        }
    }

    pub async fn start(&mut self) {
        self.peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            println!("Peer Connection State has changed: {s}");
    
            if s == RTCPeerConnectionState::Failed {
                println!("Peer Connection has gone to failed exiting");
            }
    
            Box::pin(async {})
        }));
    
        self.peer_connection
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
}
