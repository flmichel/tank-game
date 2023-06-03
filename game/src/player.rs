use std::{io, sync::Arc};

use actix::{
    Actor, Addr, Arbiter, AsyncContext, Context, Handler, Message, ResponseFuture, WrapFuture,
};
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine, APIBuilder,
        API,
    },
    data_channel::{data_channel_message::DataChannelMessage, RTCDataChannel},
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{
        self, configuration::RTCConfiguration, sdp::session_description::RTCSessionDescription,
        RTCPeerConnection,
    },
};

use crate::{game::Game, signal};

#[derive(Message)]
#[rtype(result = "")]
pub struct AddPlayer(pub String);

pub struct PlayersHandler {
    game_address: Addr<Game>,
}

impl PlayersHandler {
    pub fn new(game_address: Addr<Game>) -> Self {
        Self { game_address }
    }
}

impl Actor for PlayersHandler {
    type Context = Context<Self>;
}

impl Handler<AddPlayer> for PlayersHandler {
    type Result = ();

    fn handle(&mut self, add_player: AddPlayer, ctx: &mut Context<Self>) -> Self::Result {
        let game_addr = self.game_address.clone();
        let fut = Box::pin(async move {
            let offer = serde_json::from_str::<RTCSessionDescription>(&add_player.0)
                .expect("failed to unmarshal description");
            let (peer_connection, answer) =
                WebRTCData::start_peer_connection(offer.to_owned()).await;

            Player::new(game_addr, peer_connection).start();
        });
        print!("hello");
        let actor = fut.into_actor(self);
        let spawn = ctx.spawn(actor);
    }
}

pub struct TerminalInputActor {
    addr: Addr<PlayersHandler>,
}

impl TerminalInputActor {
    pub fn new(addr: Addr<PlayersHandler>) -> Self {
        println!("new terminal input actor");
        Self { addr }
    }
}

impl Actor for TerminalInputActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        loop {
            print!("enter new sdp>: ");
            let line = signal::must_read_stdin().expect("failed to read stdin");
            let sdp_offer = signal::decode(line.as_str()).expect("failed to decode");
            self.addr.send(AddPlayer(sdp_offer));
        }
    }
}

pub struct Player {
    peer_connection: RTCPeerConnection,
    game_address: Addr<Game>,
}

impl Player {
    fn new(game_address: Addr<Game>, peer_connection: RTCPeerConnection) -> Self {
        peer_connection.on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
            let d_label = d.label().to_owned();
            let d_id = d.id();
            println!("New DataChannel {d_label} {d_id}");

            // Register channel opening handling
            Box::pin(async move {
                let d2 = Arc::clone(&d);
                let d_label2 = d_label.clone();
                let d_id2 = d_id;

                // Register text message handling
                d.on_message(Box::new(move |msg: DataChannelMessage| {
                    let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                    println!("Message from DataChannel '{d_label}': '{msg_str}'");
                    Box::pin(async {})
                }));
            })
        }));

        Player {
            peer_connection,
            game_address,
        }
    }
}

impl Actor for Player {
    type Context = Context<Self>;
}

struct Ping;

struct MyOtherActor {}

impl Actor for MyOtherActor {
    type Context = Context<Self>;

    // Called when an actor gets polled the first time.
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.notify(AddPlayer("lol".to_owned()));
    }
}

impl Handler<AddPlayer> for MyOtherActor {
    type Result = ();

    fn handle(&mut self, _msg: AddPlayer, ctx: &mut Context<Self>) -> Self::Result {
        let fut = Box::pin(async {
            println!("Easy task done!");
        });

        let actor_future = fut.into_actor(self);

        // Still using `wait` here.
        ctx.wait(actor_future);
    }
}
