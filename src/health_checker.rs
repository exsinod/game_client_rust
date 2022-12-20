use specs::prelude::*;

use crate::{components::*, AttackCommand};

pub struct HealthChecker;

impl<'a> System<'a> for HealthChecker {
    type SystemData = (
        WriteStorage<'a, Status>,
        ReadStorage<'a, KeyboardControlled>,
        ReadExpect<'a, Option<AttackCommand>>,
    );

    fn run(&mut self, mut data: Self::SystemData) {
        let shoot_command = match &*data.2 {
            Some(shoot_command) => shoot_command,
            None => return, // no change
        };

        for status in (&mut data.0).join() {
            match shoot_command {
                AttackCommand::Cast() => {
                    status.health -= 10;
                }
                AttackCommand::Stop => {}
            }
        }
    }
}
