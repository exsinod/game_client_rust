use log::trace;
use specs::prelude::*;

use crate::{
    components::{ExternalControlled, Player, Position},
    ServerUpdate,
};

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
            ServerUpdate::Update(updated_players) => {
                trace!("server update: {:?}", updated_players);
                for (mut player, position) in (&mut data.2, &mut data.3).join() {
                    let current_player = updated_players.get(&player.id);
                    match current_player {
                        Some(current_player) => {
                            if player.id == current_player.id {
                                position.0.x = current_player.pos.x;
                                position.0.y = current_player.pos.y;
                                player.id = current_player.id.clone();
                                player.char_name = current_player.id.clone();
                                player.pos = current_player.pos.clone();
                                player.velocity = current_player.velocity;
                            }
                        }
                        None => {}
                    }
                }
            }
            _ => {}
        };
    }
}
