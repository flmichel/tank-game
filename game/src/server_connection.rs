use futures_channel::mpsc::{Receiver, Sender};

pub struct ServerConnection {
    sender_to_game: Sender<ServerMessage>,
    sender_to_remote_creator: Sender<SdpOffer>,
    receiver: Receiver<MessageToServer>,
}

pub enum MessageToServer {
    SdpAnswer(SdpAnswer),
}
