pub enum MessageToGame {
    Input(Input),
    RoomId(String),
}

pub enum Input {
    A,
    B,
}
