use specs::prelude::*;

use crate::{components::*, DIMENSION};

pub struct Physics;

impl<'a> System<'a> for Physics {
    type SystemData = (ReadStorage<'a, Player>, WriteStorage<'a, Position>);

    fn run(&mut self, mut data: Self::SystemData) {
        let higher_horizontal_bound = DIMENSION.width as i32 / 2;
        let lower_horizontal_bound = -higher_horizontal_bound;
        let higher_vertical_bound = (DIMENSION.height as i32 - 150) / 2;
        let lower_vertical_bound = -higher_vertical_bound;
        for (player, pos) in (&data.0, &mut data.1).join() {
            // println!("setting bounds: {:?}", pos);
            pos.0.x = player.pos.x;
            pos.0.y = player.pos.y;
            if pos.0.x < lower_horizontal_bound {
                pos.0.x = higher_horizontal_bound;
            }
            if pos.0.x > higher_horizontal_bound {
                pos.0.x = lower_horizontal_bound;
            }
            if pos.0.y < lower_vertical_bound {
                pos.0.y = higher_vertical_bound;
            }
            if pos.0.y > higher_vertical_bound {
                pos.0.y = lower_vertical_bound;
            }

            // match vel.direction {
            //     Left => {
            //         pos.0 = pos.0.offset(-vel.speed, 0);
            //         if pos.0.x < lower_horizontal_bound {
            //             pos.0.x = higher_horizontal_bound;
            //         }
            //     }
            //     Right => {
            //         pos.0 = pos.0.offset(vel.speed, 0);
            //         if pos.0.x > higher_horizontal_bound {
            //             pos.0.x = lower_horizontal_bound;
            //         }
            //     }
            //     Up => {
            //         pos.0 = pos.0.offset(0, -vel.speed);
            //         if pos.0.y < lower_vertical_bound {
            //             pos.0.y = higher_vertical_bound;
            //         }
            //     }
            //     Down => {
            //         pos.0 = pos.0.offset(0, vel.speed);
            //         if pos.0.y > higher_vertical_bound {
            //             pos.0.y = lower_vertical_bound;
            //         }
            //     }
            // }
        }
    }
}
