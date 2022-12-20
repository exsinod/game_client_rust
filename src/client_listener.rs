use specs::prelude::*;

use crate::{
    components::{ExternalControlled, Player, Position, Velocity},
    ClientCommand, ServerUpdate,
};
const PLAYER_MOVEMENT_SPEED: i32 = 10;

pub struct ClientListener;

impl<'a> System<'a> for ClientListener {
    type SystemData = (
        ReadExpect<'a, Option<ServerUpdate>>,
        ReadStorage<'a, ExternalControlled>,
        WriteStorage<'a, Player>,
    );

    fn run(&mut self, mut data: Self::SystemData) {
        let server_update = match &*data.0 {
            Some(server_update) => server_update,
            None => return, // no change
        };

        // let updated_player = match server_update {
        match server_update {
            ServerUpdate::Update(updated_player) => {
                println!("server update: {:?}", updated_player);
                    for mut player in (&mut data.2).join() {
                    player.pos = updated_player.pos;
                }
            }
            _ => {}
            // ServerUpdate::Nothing => &default_player,
            // ServerUpdate::Login(_) => &default_player,
        };
    }
}
