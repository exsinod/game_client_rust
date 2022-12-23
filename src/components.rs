use std::{
    fmt,
    net::{SocketAddr, UdpSocket},
    str::FromStr,
};

use sdl2::rect::{Point, Rect};
use specs::prelude::*;
use specs_derive::Component;

pub static RECV_SERVER_PORT: u16 = 8877;
pub static SEND_SERVER_PORT: u16 = 8878;

static CLIENT_ADDR: [u8; 4] = [0, 0, 0, 0];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Component, Clone, Debug)]
#[storage(VecStorage)]
pub struct Player {
    pub id: String,
    pub char_name: String,
    pub skin: usize,
    pub pos: Point,
    pub velocity: Direction,
    pub team: u8,
    pub world_pos: Point,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            // id: "MTI3LjAuMC4xOjg3Njc=".to_string(),
            id: String::default(),
            char_name: String::default(),
            skin: usize::default(),
            pos: Point::new(0, 0),
            velocity: Direction::Up,
            team: u8::default(),
            world_pos: Point::new(0, 0),
        }
    }
}

impl Player {
    pub fn new(
        id: String,
        char_name: String,
        skin: usize,
        world_pos: Point,
        pos: Point,
        velocity: Direction,
        team: u8,
    ) -> Self {
        Self {
            id,
            char_name,
            skin,
            pos,
            velocity,
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
            velocity: parts
                .next()
                .unwrap_or("4")
                .parse::<Direction>()
                .unwrap_or(Direction::Stationary),
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
