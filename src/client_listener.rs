use log::trace;
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
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, mut data: Self::SystemData) {
        let server_update = match &*data.0 {
            Some(server_update) => server_update,
            None => return, // no change
        };

        match server_update {
            ServerUpdate::Update(updated_player) => {
                trace!("server update: {:?}", updated_player);
                for (mut player, position) in (&mut data.2, &mut data.3).join() {
                    if player.id == updated_player.id {
                        position.0.x = updated_player.pos.x;
                        position.0.y = updated_player.pos.y;
                        player.id = updated_player.id.clone();
                        player.char_name = updated_player.id.clone();
                        player.pos = updated_player.pos;
                    }
                }
            }
            _ => {}
            // ServerUpdate::Nothing => &default_player,
            // ServerUpdate::Login(_) => &default_player,
        };
    }
}
