use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::WindowCanvas;
use specs::prelude::*;

use crate::components::*;

// Type alias for the data needed by the renderer
pub type SystemData<'a> = (ReadStorage<'a, Position>, ReadStorage<'a, Status>);

pub fn draw_to_canvas(canvas: &mut WindowCanvas, data: SystemData) -> Result<(), String> {
    let (width, height) = canvas.output_size()?;

    for (pos, status) in (&data.0, &data.1).join() {
        let screen_position = pos.0 + Point::new(width as i32 / 2, (height as i32 - 100) / 2);
        canvas.set_draw_color(Color::RGB(255, 100, 100));
        canvas.fill_rect(Rect::from_center(screen_position.offset(0, 34), 304, 24))?;
        canvas.set_draw_color(Color::GRAY);
        canvas.fill_rect(Rect::from_center(screen_position.offset(0, 34), 300, 20))?;
        canvas.set_draw_color(Color::GREEN);
        canvas.fill_rect(Rect::from_center(
            screen_position.offset(0, 34),
            status.health * 3,
            18,
        ))?;
    }

    Ok(())
}
