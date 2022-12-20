use std::{fmt, net::UdpSocket};

use sdl2::rect::{Point, Rect};
use specs::{prelude::*, rayon::vec};
use specs_derive::Component;

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
    Update(Player),
    Login(String),
    Nothing,
}

#[derive(Copy, Clone, Debug)]
pub enum ClientCommand {
    Stop,
    Action(),
    Move(Direction),
}

#[derive(Copy, Clone, Debug)]
pub enum MovementCommand {
    Stop,
    Move(Direction),
}

#[derive(Copy, Clone, Debug)]
pub enum AttackCommand {
    Stop,
    Cast(),
}
#[derive(Component, Debug)]
pub struct ServerRuntime {
    pub socket: UdpSocket,
    players: Vec<Player>,
}

impl ServerRuntime {
    pub fn new() -> Self {
        Self {
            socket: UdpSocket::bind(&"127.0.0.1:8767").unwrap(),
            players: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
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
        };
        f.write_str(&direction.to_string())
    }
}

#[derive(Component, Clone, Debug)]
#[storage(VecStorage)]
pub struct Player {
    pub id: String,
    char_name: String,
    skin: usize,
    pub pos: Point,
    team: u8,
    world_pos: Point,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: "MTI3LjAuMC4xOjg3Njc=".to_string(),
            char_name: String::default(),
            skin: usize::default(),
            pos: Point::new(0, 0),
            team: u8::default(),
            world_pos: Point::new(0, 0),
        }
    }
}

impl Player {
    fn new(
        id: String,
        char_name: String,
        skin: usize,
        world_pos: Point,
        pos: Point,
        team: u8,
    ) -> Self {
        Self {
            id,
            char_name,
            skin,
            pos,
            team,
            world_pos,
        }
    }
    pub fn from_str(string: &str) -> Self {
        let mut parts = string.split(";").into_iter();
        Self {
            id: parts.next().unwrap_or(&String::new()).to_string(),
            char_name: parts.next().unwrap_or(&String::new()).to_string(),
            skin: parts.next().unwrap_or("0").parse::<usize>().unwrap_or(0),
            pos: Point::new(
                parts.next().unwrap_or("0").parse::<i32>().unwrap_or(0),
                parts.next().unwrap_or("0").parse::<i32>().unwrap_or(0),
            ),
            team: parts.next().unwrap_or("0").parse::<u8>().unwrap_or(0),
            world_pos: Point::new(
                parts.next().unwrap_or("0").parse::<i32>().unwrap_or(0),
                parts.next().unwrap_or("0").parse::<i32>().unwrap_or(0),
            ),
        }
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
