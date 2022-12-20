use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use specs::prelude::*;

use crate::{components::*, DIMENSION};

// Type alias for the data needed by the renderer
pub type SystemData<'a> = ReadStorage<'a, UiComponent>;

pub fn draw_to_canvas(
    canvas: &mut WindowCanvas,
    background: Color,
    _data: SystemData,
) -> Result<(), String> {
    canvas.set_draw_color(background);
    canvas.fill_rect(Rect::new(
        0,
        (DIMENSION.height - 100) as i32,
        DIMENSION.width as u32,
        100,
    ))?;

    Ok(())
}
