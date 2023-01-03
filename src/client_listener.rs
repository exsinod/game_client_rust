use log::{debug, trace};
use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
    render::WindowCanvas,
};
use specs::prelude::*;

use crate::{
    components::{ExternalControlled, Player, Position},
    ServerUpdate,
};

// pub struct ClientListener;
//
// impl<'a> System<'a> for ClientListener {
//     type SystemData = (
//         ReadExpect<'a, Option<ServerUpdate>>,
//         ReadStorage<'a, ExternalControlled>,
//         WriteStorage<'a, Player>,
//         WriteStorage<'a, Position>,
//     );
//
// Type alias for the data needed by the renderer
pub type SystemData<'a> = (
    ReadExpect<'a, Option<ServerUpdate>>,
    WriteStorage<'a, Player>,
    WriteStorage<'a, Position>,
);

pub fn draw_to_canvas(canvas: &mut WindowCanvas, mut data: SystemData) -> Result<(), String> {
    let (width, height) = canvas.output_size()?;
    let server_update = match &*data.0 {
        Some(server_update) => server_update,
        None => return Ok(()), // no change
    };

    match server_update {
        ServerUpdate::Update(updated_players) => {
            trace!("server update: {:?}", updated_players);
            for (player, position) in (&mut data.1, &mut data.2).join() {
                let current_player = updated_players.get(&player.id);
                match current_player {
                    Some(current_player) => {
                        if player.id == current_player.id {
                            position.0.x = current_player.pos.x;
                            position.0.y = current_player.pos.y;
                            player.id = current_player.id.clone();
                            player.char_name = current_player.id.clone();
                            player.pos = current_player.pos.clone();
                            player.velocity = current_player.velocity;
                        }
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
                    None => {}
                }
            }
        }
        _ => {}
    };
    Ok(())
}
