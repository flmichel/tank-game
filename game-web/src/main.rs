use std::{cell::RefCell, collections::HashMap, rc::Rc};

use base64::{engine::general_purpose::STANDARD, Engine};
use game_core::{
    protocol::{MessageToServer, Ping, Pong, SdpMessage, ServerMessage},
    remotes::RemoteInput,
    Color, Game, MessageToGame, Rect, Renderer, RoomId, Sprite,
};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement, MessageEvent, RtcConfiguration,
    RtcDataChannel, RtcDataChannelEvent, RtcIceGatheringState, RtcIceServer, RtcPeerConnection,
    RtcSdpType, RtcSessionDescriptionInit, UrlSearchParams, WebSocket,
};

const MAP: &str = include_str!("../../game-native/assets/map.txt");
// The server's `games_port`, on the host that serves this page
const SERVER_PORT: u16 = 5000;
const STUN_SERVER: &str = "stun:stun.l.google.com:19302";

/// State shared by the browser callbacks. JS is single-threaded, so a RefCell is enough.
struct Shell {
    game: Game,
    data_channels: HashMap<u32, RtcDataChannel>,
    // Held so the browser keeps the connections alive
    peer_connections: Vec<RtcPeerConnection>,
    next_socket_id: u32,
}

type SharedShell = Rc<RefCell<Shell>>;

/// RTCSessionDescription as JSON, the format the remote base64-encodes.
#[derive(Serialize, Deserialize)]
struct SessionDescription {
    #[serde(rename = "type")]
    kind: String,
    sdp: String,
}

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    if let Err(err) = start() {
        error!("failed to start the game: {err:?}");
    }
}

fn start() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let canvas: HtmlCanvasElement = window
        .document()
        .and_then(|document| document.get_element_by_id("game"))
        .ok_or("no #game canvas")?
        .dyn_into()?;

    // ponytail: size fixed at load, reload the page after resizing the window
    let width = window.inner_width()?.as_f64().unwrap_or(800.) as u32;
    let height = window.inner_height()?.as_f64().unwrap_or(600.) as u32;
    canvas.set_width(width);
    canvas.set_height(height);

    let shell = Rc::new(RefCell::new(Shell {
        game: Game::new(MAP, (width, height)),
        data_channels: HashMap::new(),
        peer_connections: Vec::new(),
        next_socket_id: 0,
    }));

    // `?server=ws://host:port/ws` overrides the default, e.g. for local testing
    let server_url = match UrlSearchParams::new_with_str(&window.location().search()?)?.get("server") {
        Some(url) => url,
        None => format!("ws://{}:{SERVER_PORT}/ws", window.location().hostname()?),
    };
    connect_to_server(&server_url, shell.clone())?;

    // Desynchronized: the browser may draw it straight to the screen, skipping a compositor frame
    let attributes = js_sys::Object::new();
    js_sys::Reflect::set(&attributes, &"desynchronized".into(), &JsValue::TRUE)?;
    let context: CanvasRenderingContext2d = canvas
        .get_context_with_context_options("2d", &attributes)?
        .ok_or("no 2d context")?
        .dyn_into()?;
    context.set_font("24px sans-serif");
    context.set_text_baseline("top");
    let renderer = CanvasRenderer {
        context,
        width: width as f64,
        height: height as f64,
        player_face: load_image("grin.png")?,
        missile: load_image("missile.png")?,
    };
    run_game_loop(shell, renderer);
    Ok(())
}

fn run_game_loop(shell: SharedShell, mut renderer: CanvasRenderer) {
    let frame: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let next_frame = frame.clone();
    let mut last_time: Option<f64> = None;

    *frame.borrow_mut() = Some(Closure::new(move |time: f64| {
        let dt = last_time.map_or(0., |last| (time - last) / 1000.);
        last_time = Some(time);

        let mut state = shell.borrow_mut();
        let Shell { game, data_channels, .. } = &mut *state;
        game.tick(dt);
        game.render(&mut renderer);

        for (socket_id, message) in game.drain_outbox() {
            let Some(channel) = data_channels.get(&socket_id) else {
                warn!("No data channel for socket id \"{socket_id}\".");
                continue;
            };
            if let Err(err) = channel.send_with_str(&serde_json::to_string(&message).unwrap()) {
                error!("error sending a message to a remote: {err:?}");
            }
        }

        request_animation_frame(next_frame.borrow().as_ref().unwrap());
    }));
    request_animation_frame(frame.borrow().as_ref().unwrap());
}

fn request_animation_frame(callback: &Closure<dyn FnMut(f64)>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .unwrap();
}

fn connect_to_server(url: &str, shell: SharedShell) -> Result<(), JsValue> {
    let websocket = WebSocket::new(url)?;
    let websocket_for_answers = websocket.clone();

    let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
        let Some(text) = event.data().as_string() else {
            return;
        };
        match serde_json::from_str::<ServerMessage>(&text) {
            Ok(ServerMessage::RoomId(id)) => {
                shell.borrow_mut().game.handle(MessageToGame::RoomId(RoomId(id)))
            }
            Ok(ServerMessage::SdpOffer(offer)) => {
                let socket_id = {
                    let mut state = shell.borrow_mut();
                    state.next_socket_id += 1;
                    state.next_socket_id - 1
                };
                let (shell, websocket) = (shell.clone(), websocket_for_answers.clone());
                spawn_local(async move {
                    let id = offer.id;
                    if let Err(err) = answer_offer(offer, socket_id, shell, websocket).await {
                        error!("failed to answer offer {id}: {err:?}");
                    }
                });
            }
            Err(err) => warn!("unexpected message from server \"{text}\": {err}"),
        }
    });
    websocket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();
    Ok(())
}

async fn answer_offer(
    offer: SdpMessage,
    socket_id: u32,
    shell: SharedShell,
    websocket: WebSocket,
) -> Result<(), JsValue> {
    let offer_json = STANDARD.decode(&offer.data).map_err(|err| err.to_string())?;
    let remote: SessionDescription = serde_json::from_slice(&offer_json).map_err(|err| err.to_string())?;

    let ice_server = RtcIceServer::new();
    ice_server.set_urls(&JsValue::from_str(STUN_SERVER));
    let configuration = RtcConfiguration::new();
    configuration.set_ice_servers(&js_sys::Array::of1(&ice_server));
    let peer_connection = RtcPeerConnection::new_with_configuration(&configuration)?;
    listen_to_remote(&peer_connection, socket_id, shell.clone());
    shell.borrow_mut().peer_connections.push(peer_connection.clone());

    let description = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
    description.set_sdp(&remote.sdp);
    JsFuture::from(peer_connection.set_remote_description(&description)).await?;

    let answer = JsFuture::from(peer_connection.create_answer()).await?;
    let description = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
    description.set_sdp(&js_sys::Reflect::get(&answer, &"sdp".into())?.as_string().unwrap_or_default());
    JsFuture::from(peer_connection.set_local_description(&description)).await?;

    // The answer goes back in a single signaling message, so it has to carry the candidates
    wait_for_ice_gathering(&peer_connection).await?;

    let local = peer_connection.local_description().ok_or("no local description")?;
    let answer = serde_json::to_string(&SessionDescription { kind: "answer".to_owned(), sdp: local.sdp() })
        .map_err(|err| err.to_string())?;
    let message = MessageToServer::SdpAnswer(SdpMessage { id: offer.id, data: STANDARD.encode(answer) });
    websocket.send_with_str(&serde_json::to_string(&message).map_err(|err| err.to_string())?)
}

// Gathering has to be good enough, not complete: an unreachable STUN server keeps the browser
// gathering for tens of seconds, long past the server's answer timeout, so a phone never joins.
// The candidates found by then go in the answer, and host candidates alone connect a phone on the
// same network. Where STUN answers, gathering completes well inside this and nothing is lost.
const ICE_GATHERING_TIMEOUT_MS: i32 = 1_000;

async fn wait_for_ice_gathering(peer_connection: &RtcPeerConnection) -> Result<(), JsValue> {
    if peer_connection.ice_gathering_state() == RtcIceGatheringState::Complete {
        return Ok(());
    }
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let watched = peer_connection.clone();
        let resolve_on_timeout = resolve.clone();
        let on_change = Closure::<dyn FnMut()>::new(move || {
            if watched.ice_gathering_state() == RtcIceGatheringState::Complete {
                let _ = resolve.call0(&JsValue::NULL);
            }
        });
        peer_connection.set_onicegatheringstatechange(Some(on_change.as_ref().unchecked_ref()));
        on_change.forget();

        // Resolving a promise twice is a no-op, so whichever comes first wins
        let give_up = Closure::<dyn FnMut()>::new(move || {
            let _ = resolve_on_timeout.call0(&JsValue::NULL);
        });
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                give_up.as_ref().unchecked_ref(),
                ICE_GATHERING_TIMEOUT_MS,
            );
        }
        give_up.forget();
    });
    JsFuture::from(promise).await.map(|_| ())
}

fn listen_to_remote(peer_connection: &RtcPeerConnection, socket_id: u32, shell: SharedShell) {
    let on_data_channel = Closure::<dyn FnMut(RtcDataChannelEvent)>::new(move |event: RtcDataChannelEvent| {
        let channel = event.channel();
        let shell_for_inputs = shell.clone();
        let channel_for_pongs = channel.clone();
        let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
            let Some(text) = event.data().as_string() else {
                return;
            };
            // Answered here, not by the game loop, so the remote measures the network alone
            if let Ok(Ping { ping }) = serde_json::from_str(&text) {
                let pong = serde_json::to_string(&Pong { pong: ping }).unwrap();
                if let Err(err) = channel_for_pongs.send_with_str(&pong) {
                    warn!("failed to answer a ping on socket id \"{socket_id}\": {err:?}");
                }
                return;
            }
            match serde_json::from_str::<RemoteInput>(&text) {
                Ok(input) => shell_for_inputs
                    .borrow_mut()
                    .game
                    .handle(MessageToGame::from_remote(socket_id, input)),
                Err(err) => warn!("ignoring input \"{text}\" from socket id \"{socket_id}\": {err}"),
            }
        });
        channel.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();
        // Inputs arrive on both channels; the game's messages go back on the reliable one
        if channel.label() == "channel" {
            shell.borrow_mut().data_channels.insert(socket_id, channel);
        }
    });
    peer_connection.set_ondatachannel(Some(on_data_channel.as_ref().unchecked_ref()));
    on_data_channel.forget();
}

fn load_image(src: &str) -> Result<HtmlImageElement, JsValue> {
    let image = HtmlImageElement::new()?;
    image.set_src(src);
    Ok(image)
}

/// Canvas 2D adapter for the core's drawing port.
struct CanvasRenderer {
    context: CanvasRenderingContext2d,
    width: f64,
    height: f64,
    player_face: HtmlImageElement,
    missile: HtmlImageElement,
}

fn css(color: Color) -> String {
    format!("rgb({},{},{})", color.r, color.g, color.b)
}

impl Renderer for CanvasRenderer {
    fn clear(&mut self, color: Color) {
        self.context.set_fill_style_str(&css(color));
        self.context.fill_rect(0., 0., self.width, self.height);
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.context.set_fill_style_str(&css(color));
        self.context.fill_rect(rect.x as f64, rect.y as f64, rect.w as f64, rect.h as f64);
    }

    fn draw_line(&mut self, from: (i32, i32), to: (i32, i32), color: Color) {
        self.context.set_stroke_style_str(&css(color));
        self.context.begin_path();
        self.context.move_to(from.0 as f64, from.1 as f64);
        self.context.line_to(to.0 as f64, to.1 as f64);
        self.context.stroke();
    }

    fn draw_sprite(&mut self, sprite: Sprite, rect: Rect) {
        let image = match sprite {
            Sprite::PlayerFace => &self.player_face,
            Sprite::Missile => &self.missile,
        };
        // Drawing an image that hasn't loaded yet does nothing
        let _ = self.context.draw_image_with_html_image_element_and_dw_and_dh(
            image,
            rect.x as f64,
            rect.y as f64,
            rect.w as f64,
            rect.h as f64,
        );
    }

    fn draw_text(&mut self, text: &str, x: i32, y: i32, color: Color) {
        self.context.set_fill_style_str(&css(color));
        let _ = self.context.fill_text(text, x as f64, y as f64);
    }
}
