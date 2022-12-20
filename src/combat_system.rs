// use specs::prelude::*;
//
// use crate::{components::*, AttackCommand};
//
// pub struct CombatSystem;
//
// impl<'a> System<'a> for CombatSystem {
//     type SystemData = (
//         ReadExpect<'a, Option<AttackCommand>>,
//         ReadStorage<'a, Player>,
//     );
//
//     fn run(&mut self, mut data: Self::SystemData) {
//         let attack_command = match &*data.0 {
//             Some(attack_command) => attack_command,
//             None => return, // no change
//         };
//     }
// }
