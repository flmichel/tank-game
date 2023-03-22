use actix::{Actor, Context};

pub struct Game;

impl Game {
    pub fn new() -> Self {
        Self
    }
}
impl Actor for Game {
    type Context = Context<Self>;
}
/*impl Handler<PlayerInput> for Game {

}*/
