use log::trace;

use sdl2::{pixels::Color, rect::Rect};
use specs::prelude::*;

use crate::components::*;

use super::MovementCommand;

pub struct Keyboard;

impl<'a> System<'a> for Keyboard {
    type SystemData = (
        ReadExpect<'a, Option<MovementCommand>>,
        ReadStorage<'a, EntityResource>,
        ReadStorage<'a, KeyboardControlled>,
        WriteStorage<'a, Player>,
    );

    fn run(&mut self, mut data: Self::SystemData) {
        let movement_command = match &*data.0 {
            Some(movement_command) => movement_command,
            None => return, // no change
        };

        for entity_res in (data.1).join() {
            for player in (&mut data.3).join() {
                if entity_res.0 == EntityType::Main {
                    match movement_command {
                        MovementCommand::Move(direction) => match direction {
                            Direction::Up => {
                                player.pos.y -= 5;
                            }
                            Direction::Right => {
                                player.pos.x += 5;
                            }
                            Direction::Down => {
                                player.pos.y += 5;
                            }
                            Direction::Left => {
                                player.pos.x -= 5;
                            }
                            _ => {}
                        },
                        MovementCommand::Stop => {}
                        _ => {}
                    }
                }
            }
        }
    }
}
