use std::{fmt::format, net::UdpSocket};

use chrono::Utc;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use specs::prelude::*;

use crate::{components::*, syncer, DIMENSION};

// Type alias for the data needed by the renderer
pub type SystemData<'a> = (
    ReadExpect<'a, Option<MovementCommand>>,
    ReadStorage<'a, Player>,
);

pub fn sync_pos_to_server(socket: &UdpSocket, data: SystemData) -> Result<(), String> {
    let movement_command = match &*data.0 {
        Some(movement_command) => movement_command,
        None => return Ok(()), // no change
    };
    if movement_command == &MovementCommand::Stop {
        for player in (data.1).join() {
            if player.id == String::from("blub_id1") {
                let sync_cmd = format!(
                    "{};S0;{};{}",
                    Utc::now().timestamp(),
                    player.id,
                    serde_json::to_string(&player.pos).unwrap()
                );
                println!("sending sync {}", sync_cmd);
                socket.send(sync_cmd.as_bytes()).unwrap();
            }
        }
    }
    Ok(())
}
