use specs::prelude::*;

use crate::components::*;

pub struct Animator;

impl<'a> System<'a> for Animator {
    type SystemData = (
        WriteStorage<'a, MovementAnimation>,
        WriteStorage<'a, Sprite>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, mut data: Self::SystemData) {
        use self::Direction::*;
        //TODO: This code can be made nicer and more idiomatic using more pattern matching.
        // Look up "rust irrefutable patterns" and use them here.
        for (anim, sprite, player) in (&mut data.0, &mut data.1, &data.2).join() {
            if player.velocity == 5 {
                continue;
            }

            let frames = match player.velocity {
                0 => &anim.up_frames,
                1 => &anim.right_frames,
                2 => &anim.down_frames,
                3 => &anim.left_frames,
                4 => &anim.left_frames,
                _ => &anim.left_frames,
            };

            anim.current_frame = (anim.current_frame + 1) % frames.len();
            *sprite = frames[anim.current_frame].clone();
        }
    }
}
