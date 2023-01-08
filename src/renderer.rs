use log::{debug, trace};
use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
    render::WindowCanvas,
};
use specs::prelude::*;

use crate::components::Player;

// Type alias for the data needed by the renderer
pub type SystemData<'a> = WriteStorage<'a, Player>;

pub fn draw_to_canvas(canvas: &mut WindowCanvas, mut data: SystemData) -> Result<(), String> {
    let (width, height) = canvas.output_size()?;

    for player in (&mut data).join() {
        debug!("rendering: {:?}", player);
        // Treat the center of the screen as the (0, 0) coordinate
        let screen_position = Point::new(player.pos.x, player.pos.y)
            + Point::new(width as i32 / 2, (height as i32 - 100) / 2);
        let screen_rect = Rect::from_center(screen_position, 10, 10);
        let color = match player.skin {
            3 => Color::BLACK,
            4 => Color::WHITE,
            5 => Color::YELLOW,
            6 => Color::BLUE,
            7 => Color::CYAN,
            8 => Color::GREEN,
            9 => Color::RED,
            10 => Color::RGB(255, 255, 10),
            _ => Color::GRAY,
        };
        canvas.set_draw_color(color);
        canvas.fill_rect(screen_rect).unwrap();
    }
    Ok(())
}
