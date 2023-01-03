use log::trace;

use specs::prelude::*;

use crate::components::*;

use super::MovementCommand;

pub struct Keyboard;

impl<'a> System<'a> for Keyboard {
    type SystemData = (
        ReadExpect<'a, Option<MovementCommand>>,
        ReadStorage<'a, KeyboardControlled>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let movement_command = match &*data.0 {
            Some(movement_command) => movement_command,
            None => return, // no change
        };

        match movement_command {
            MovementCommand::Move(_direction) => {
                trace!("move: updating speed and vel");
            }
            MovementCommand::Stationary => {
                trace!("stationary: not updating speed and vel");
            }
            MovementCommand::Stop => {
                trace!("stop: not updating speed and vel");
            }
        }
    }
}
