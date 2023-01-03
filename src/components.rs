use std::{
    collections::HashMap,
    fmt,
    net::{SocketAddr, UdpSocket},
    str::FromStr,
};

use chrono::Utc;
use sdl2::rect::Rect;
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs_derive::Component;

pub static RECV_SERVER_PORT: u16 = 8877;
pub static SEND_SERVER_PORT: u16 = 8878;

static CLIENT_ADDR: [u8; 4] = [127, 0, 0, 1];

pub struct Dimension {
    pub width: u32,
    pub height: u32,
}
pub const DIMENSION: Dimension = Dimension {
    width: 800,
    height: 600,
};

#[derive(Clone, Debug)]
pub enum ServerUpdate {
    Update(HashMap<String, Player>),
    Login(String),
    Nothing,
}

#[derive(Copy, Clone, Debug)]
pub enum MovementCommand {
    Stop,
    Stationary,
    Move(Direction),
}

#[derive(Copy, Clone, Debug)]
pub enum AttackCommand {
    Stop,
    Cast(),
}
#[derive(Component, Debug)]
pub struct ServerRuntime {
    pub send_socket: UdpSocket,
    pub recv_socket: UdpSocket,
}

impl ServerRuntime {
    pub fn new(client_socket_port: u16) -> Self {
        Self {
            recv_socket: UdpSocket::bind(SocketAddr::from((CLIENT_ADDR, client_socket_port)))
                .unwrap(),
            send_socket: UdpSocket::bind(SocketAddr::from((CLIENT_ADDR, client_socket_port + 1)))
                .unwrap(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Stationary,
    Up,
    Right,
    Down,
    Left,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let direction = match self {
            Self::Up => 0,
            Self::Right => 1,
            Self::Down => 2,
            Self::Left => 3,
            Self::Stationary => 4,
        };
        f.write_str(&direction.to_string())
    }
}
impl FromStr for Direction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Direction::Up),
            "1" => Ok(Direction::Right),
            "2" => Ok(Direction::Down),
            "3" => Ok(Direction::Left),
            "4" => Ok(Direction::Stationary),
            _ => Err("".to_string()),
        }
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct Player {
    pub id: String,
    pub char_name: String,
    pub skin: usize,
    pub logged_in: bool,
    pub pos: Point,
    pub velocity: u8,
    pub team: u8,
    pub world_pos: Point,
    pub last_update: i64,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            // id: "MTI3LjAuMC4xOjg3Njc=".to_string(),
            id: String::default(),
            char_name: String::default(),
            skin: usize::default(),
            logged_in: true,
            pos: Point::new(0, 0),
            velocity: 0,
            team: u8::default(),
            world_pos: Point::new(0, 0),
            last_update: Utc::now().timestamp(),
        }
    }
}

impl Player {
    pub fn new(
        id: String,
        char_name: String,
        skin: usize,
        logged_in: bool,
        world_pos: Point,
        pos: Point,
        velocity: u8,
        team: u8,
        last_update: i64,
    ) -> Self {
        Self {
            id,
            char_name,
            skin,
            logged_in,
            pos,
            velocity,
            team,
            world_pos,
            last_update,
        }
    }
    pub fn from_str(string: &str) -> Self {
        let mut parts = string.split(";").into_iter();
        Self {
            id: parts.next().unwrap_or(&String::new()).to_string(),
            char_name: parts.next().unwrap_or(&String::new()).to_string(),
            skin: parts.next().unwrap_or("0").parse::<usize>().unwrap_or(0),
            logged_in: true,
            pos: Point::new(
                parts.next().unwrap_or("0").parse::<i32>().unwrap_or(0),
                parts.next().unwrap_or("0").parse::<i32>().unwrap_or(0),
            ),
            velocity: 0,
            // velocity: parts
            //     .next()
            //     .unwrap_or("4")
            //     .parse::<Direction>()
            //     .unwrap_or(Direction::Stationary),
            team: parts.next().unwrap_or("0").parse::<u8>().unwrap_or(0),
            world_pos: Point::new(
                parts.next().unwrap_or("0").parse::<i32>().unwrap_or(0),
                parts.next().unwrap_or("0").parse::<i32>().unwrap_or(0),
            ),
            last_update: 0,
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&*format!("{};{}", self.x, self.y))
    }
}

#[derive(Component, Debug, Default)]
#[storage(NullStorage)]
pub struct ExternalControlled;

#[derive(Component, Debug, Default)]
#[storage(NullStorage)]
pub struct KeyboardControlled;

/// The current position of a given entity
#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Position(pub Point);

/// The current speed and direction of a given entity
#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Velocity {
    pub speed: i32,
    pub direction: Direction,
}

#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct UiComponent {}

#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct Status {
    pub alive: bool,
    pub health: u32,
}

#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct Sprite {
    /// The specific spritesheet to render from
    pub spritesheet: usize,
    /// The current region of the spritesheet to be rendered
    pub region: Rect,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct MovementAnimation {
    // The current frame in the animation of the direction this entity is moving in
    pub current_frame: usize,
    pub up_frames: Vec<Sprite>,
    pub down_frames: Vec<Sprite>,
    pub left_frames: Vec<Sprite>,
    pub right_frames: Vec<Sprite>,
}
