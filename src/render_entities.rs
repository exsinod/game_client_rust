use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};
use specs::prelude::*;

use crate::{components::*, ServerUpdate};

// Type alias for the data needed by the renderer
pub type SystemData<'a> = (
    ReadExpect<'a, Option<ServerUpdate>>,
    ReadStorage<'a, Position>,
    ReadStorage<'a, Player>,
    WriteStorage<'a, Sprite>,
);

pub fn draw_to_canvas(
    canvas: &mut WindowCanvas,
    textures: &[Texture],
    mut data: SystemData,
) -> Result<(), String> {
    let (width, height) = canvas.output_size()?;

    let bla = &*data.0;
    for (pos, player, sprite) in (&data.1, &data.2, &mut data.3).join() {
        println!("rendering: {:?}", player);
        let current_frame = sprite.region;
        // Treat the center of the screen as the (0, 0) coordinate
        let screen_position = pos.0 + Point::new(width as i32 / 2, (height as i32 - 100) / 2);
        let screen_rect = Rect::from_center(
            screen_position,
            current_frame.width() * 2,
            current_frame.height() * 2,
        );
        canvas.copy(&textures[sprite.spritesheet], current_frame, screen_rect)?;
    }

    Ok(())
}
