mod animator;
mod client_listener;
mod components;
mod health_checker;
mod keyboard;
mod physics;
mod sprites;
mod status;
mod ui;

use log::{debug, error, trace};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};
use sdl2::EventPump;
use std::net::{SocketAddr, UdpSocket};
use std::{env, str};
// "self" imports the "image" module itself as well as everything else we listed
use sdl2::image::{self, InitFlag, LoadTexture};
use std::collections::{HashMap, VecDeque};

use specs::prelude::*;

use rand::Rng;
use std::time::Duration;

use crate::components::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Returns the row of the spritesheet corresponding to the given direction
fn direction_spritesheet_row(direction: Direction) -> i32 {
    match direction {
        Direction::Up => 3,
        Direction::Down => 0,
        Direction::Left => 1,
        Direction::Right => 2,
        Direction::Stationary => 4,
    }
}

/// Create animation frames for the standard character spritesheet
fn character_animation_frames(
    spritesheet: usize,
    top_left_frame: Rect,
    direction: Direction,
) -> Vec<Sprite> {
    let (frame_width, frame_height) = top_left_frame.size();
    let y_offset = top_left_frame.y() + frame_height as i32 * direction_spritesheet_row(direction);

    let mut frames = Vec::new();
    for i in 0..3 {
        frames.push(Sprite {
            spritesheet,
            region: Rect::new(
                top_left_frame.x() + frame_width as i32 * i,
                y_offset,
                frame_width,
                frame_height,
            ),
        })
    }

    frames
}

fn initialize_player(world: &mut World, player_id: String, player_spritesheet: usize) -> Entity {
    let player_top_left_frame = Rect::new(0, 0, 26, 36);

    let player_animation = MovementAnimation {
        current_frame: 0,
        up_frames: character_animation_frames(
            player_spritesheet,
            player_top_left_frame,
            Direction::Up,
        ),
        down_frames: character_animation_frames(
            player_spritesheet,
            player_top_left_frame,
            Direction::Down,
        ),
        left_frames: character_animation_frames(
            player_spritesheet,
            player_top_left_frame,
            Direction::Left,
        ),
        right_frames: character_animation_frames(
            player_spritesheet,
            player_top_left_frame,
            Direction::Right,
        ),
    };

    world
        .create_entity()
        .with(KeyboardControlled)
        .with(ExternalControlled)
        .with(Player::new(
            player_id,
            "".to_string(),
            1,
            Point::new(0, 0),
            Point::new(0, 0),
            Direction::Stationary,
            1,
        ))
        .with(Position(Point::new(0, 0)))
        .with(Status {
            alive: true,
            health: 100,
        })
        .with(player_animation.right_frames[0].clone())
        .with(player_animation)
        .build()
}

fn main() -> Result<()> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let server_addr_parts: &Vec<u8> = &args[1]
        .split(".")
        .map(|part| u8::from_str_radix(part, 10).unwrap())
        .collect::<Vec<u8>>();
    let server_addr = [
        server_addr_parts[0],
        server_addr_parts[1],
        server_addr_parts[2],
        server_addr_parts[3],
    ];
    let client_addr = &args[2];
    debug!("server addr: {:?}", server_addr);
    let random_socket_port = rand::thread_rng().gen_range(8877..65535);
    debug!("socket_port: {}", random_socket_port);
    let runtime: ServerRuntime = ServerRuntime::new(random_socket_port);
    let send_socket = &runtime.send_socket;
    let recv_socket = &runtime.recv_socket;
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = image::init(InitFlag::PNG | InitFlag::JPG)?;

    let window = video_subsystem
        .window("game tutorial", DIMENSION.width, DIMENSION.height)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let canvas = window
        .into_canvas()
        .build()
        .expect("could not make a canvas");

    let texture_creator = canvas.texture_creator();

    let mut dispatcher = DispatcherBuilder::new()
        .with(client_listener::ClientListener, "ClientListener", &[])
        .with(health_checker::HealthChecker, "HealthChecker", &[])
        .with(keyboard::Keyboard, "Keyboard", &[])
        // .with(physics::Physics, "Physics", &["Keyboard"])
        // .with(animator::Animator, "Animator", &["Keyboard"])
        // .with(physics::Physics, "Physics", &[])
        .with(animator::Animator, "Animator", &[])
        .build();

    let mut world = World::new();
    dispatcher.setup(&mut world);
    status::SystemData::setup(&mut world);
    sprites::SystemData::setup(&mut world);
    ui::SystemData::setup(&mut world);

    // Initialize resource
    let server_update: Option<ServerUpdate> = None;
    let movement_command: Option<MovementCommand> = None;
    let shoot_command: Option<AttackCommand> = None;
    world.insert(movement_command);
    world.insert(server_update);
    world.insert(shoot_command);

    let bardo = include_bytes!("../assets/bardo.png");
    let reaper = include_bytes!("../assets/reaper.png");
    let textures = [
        texture_creator.load_texture_bytes(bardo)?,
        texture_creator.load_texture_bytes(reaper)?,
    ];
    // First texture in textures array
    let _player_spritesheet = 0;
    // Second texture in the textures array
    let _enemy_spritesheet = 1;

    // initialize_enemy(&mut world, enemy_spritesheet, Point::new(-150, -150));
    // initialize_enemy(&mut world, enemy_spritesheet, Point::new(150, -190));
    // initialize_enemy(&mut world, enemy_spritesheet, Point::new(-150, 170));

    // Create UI
    world.create_entity().with(UiComponent {}).build();

    send_socket.set_read_timeout(Some(Duration::new(0, 1_000)))?;
    send_socket.set_write_timeout(Some(Duration::new(0, 1_000)))?;
    recv_socket.set_read_timeout(Some(Duration::new(0, 1_000)))?;
    recv_socket.set_write_timeout(Some(Duration::new(0, 1_000)))?;
    send_socket.connect(SocketAddr::from((server_addr, SEND_SERVER_PORT)))?;
    recv_socket.connect(SocketAddr::from((server_addr, RECV_SERVER_PORT)))?;
    match recv_socket.send(&format!("S0;{}", client_addr).into_bytes()) {
        Ok(number_of_bytes) => {
            trace!("sent {} bytes to sync", number_of_bytes);
            match recv_socket.recv_from(&mut []) {
                Ok(_) => {}
                Err(error) => {
                    error!("recv sync: {}", error)
                }
            }
        }
        Err(error) => {
            error!("send sync: {}", error)
        }
    }
    match recv_socket.send(b"L1;blub_id") {
        Ok(number_of_bytes) => {
            trace!("sent {} bytes to login", number_of_bytes);
            match recv_socket.recv_from(&mut []) {
                Ok(_) => {}
                Err(error) => {
                    error!("recv login msg: {}", error)
                }
            }
        }
        Err(error) => {
            error!("send login msg: {}", error)
        }
    }

    game_loop(
        world,
        canvas,
        &textures,
        dispatcher,
        sdl_context.event_pump()?,
        send_socket,
        recv_socket,
        VecDeque::new(),
        VecDeque::new(),
    )?;
    Ok(())
}

fn game_loop<'a>(
    mut world: World,
    mut canvas: WindowCanvas,
    textures: &[Texture<'a>],
    mut dispatcher: Dispatcher<'a, 'a>,
    mut event_pump: EventPump,
    send_socket: &UdpSocket,
    recv_socket: &UdpSocket,
    mut movements: VecDeque<MovementCommand>,
    mut attacks: VecDeque<AttackCommand>,
) -> Result<()> {
    let mut entities: HashMap<String, Entity> = HashMap::new();
    // let mut sync_trigger
    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Tab),
                    repeat: false,
                    ..
                } => {} //cycle through enemies
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    repeat: false,
                    ..
                } => attacks.push_front(AttackCommand::Cast()),
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    repeat: false,
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::A),
                    repeat: false,
                    ..
                } => {
                    movements.push_front(MovementCommand::Move(Direction::Left));
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    repeat: false,
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::D),
                    repeat: false,
                    ..
                } => {
                    movements.push_front(MovementCommand::Move(Direction::Right));
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    repeat: false,
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::W),
                    repeat: false,
                    ..
                } => {
                    movements.push_front(MovementCommand::Move(Direction::Up));
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    repeat: false,
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::S),
                    repeat: false,
                    ..
                } => {
                    movements.push_front(MovementCommand::Move(Direction::Down));
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Left),
                    repeat: false,
                    ..
                }
                | Event::KeyUp {
                    keycode: Some(Keycode::A),
                    repeat: false,
                    ..
                } => {
                    let index = movements
                        .iter()
                        .position(|item| matches!(item, MovementCommand::Move(Direction::Left)))
                        .unwrap_or(usize::MAX);
                    movements.remove(index);
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    repeat: false,
                    ..
                }
                | Event::KeyUp {
                    keycode: Some(Keycode::D),
                    repeat: false,
                    ..
                } => {
                    let index = movements
                        .iter()
                        .position(|item| matches!(item, MovementCommand::Move(Direction::Right)))
                        .unwrap_or(usize::MAX);
                    movements.remove(index);
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Up),
                    repeat: false,
                    ..
                }
                | Event::KeyUp {
                    keycode: Some(Keycode::W),
                    repeat: false,
                    ..
                } => {
                    let index = movements
                        .iter()
                        .position(|item| matches!(item, MovementCommand::Move(Direction::Up)))
                        .unwrap_or(usize::MAX);
                    movements.remove(index);
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Down),
                    repeat: false,
                    ..
                }
                | Event::KeyUp {
                    keycode: Some(Keycode::S),
                    repeat: false,
                    ..
                } => {
                    let index = movements
                        .iter()
                        .position(|item| matches!(item, MovementCommand::Move(Direction::Down)))
                        .unwrap_or(usize::MAX);
                    movements.remove(index);
                }
                _ => {}
            }
        }

        let movement_command: MovementCommand =
            *movements.front().unwrap_or(&MovementCommand::Stop);
        match movement_command {
            MovementCommand::Move(direction) => {
                let msg = format!("M0;blub_id;{}", direction);
                match recv_socket.send(&msg.into_bytes()) {
                    Ok(_) => {
                        trace!("send successful");
                        match recv_socket.recv(&mut []) {
                            Ok(_) => {}
                            Err(error) => {
                                error!("ack Move command: {}", error)
                            }
                        }
                    }
                    Err(error) => {
                        error!("sending Move command: {}", error)
                    }
                }
            }
            MovementCommand::Stop => send_player_stationary(recv_socket),
        }

        let shoot_command: Option<AttackCommand> =
            Some(attacks.pop_front().unwrap_or(AttackCommand::Stop));
        *world.write_resource() = shoot_command;

        match update_from_server(&send_socket) {
            Ok(server_update) => {
                match &server_update {
                    ServerUpdate::Update(player_update) => {
                        if !entities.contains_key(&player_update.id) {
                            let new_player =
                                initialize_player(&mut world, player_update.id.clone(), 0);
                            entities.insert(player_update.id.to_string(), new_player);
                        }
                    }
                    _ => {}
                }
                *world.write_resource() = Some(server_update);
            }
            _ => {}
        }

        // Update
        dispatcher.dispatch(&mut world);
        world.maintain();

        // Render
        canvas.set_draw_color(Color::RGB(65, 64, 255));
        canvas.clear();

        status::draw_to_canvas(&mut canvas, world.system_data())?;
        sprites::draw_to_canvas(&mut canvas, &textures, world.system_data())?;
        ui::draw_to_canvas(&mut canvas, Color::RGB(65, 255, 255), world.system_data())?;

        canvas.present();

        // Time management!
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 20));
    }
    Ok(())
}

fn _remove_entities(world: &mut World, entity: Entity) {
    world.entities().delete(entity).unwrap();
}

fn send_player_stationary(socket: &UdpSocket) {
    let msg = format!("M0;blub_id;{}", Direction::Stationary);
    match socket.send(&msg.into_bytes()) {
        Ok(_) => {
            trace!("Send Stationary successful.");
            match socket.recv(&mut []) {
                Ok(_) => {}
                Err(error) => {
                    error!("ack Stationary command: {}", error)
                }
            }
        }
        Err(error) => {
            error!("sending Stationary command: {}", error)
        }
    }
}

fn update_from_server(socket: &UdpSocket) -> Result<ServerUpdate> {
    let mut buf = [0; 200];
    match socket.recv(&mut buf) {
        Ok(number_of_bytes) => {
            trace!("update from server; {}", number_of_bytes);
            if number_of_bytes == 1 {
                return Ok(ServerUpdate::Nothing);
            } else {
                match get_operation_from(&buf) {
                    "L1;" => {
                        let player_id: &str = get_context_from(&buf, number_of_bytes);
                        debug!("get op L1; {}", player_id);
                        return Ok(ServerUpdate::Login(player_id.to_string()));
                    }
                    "P0;" => {
                        let player = Player::from_str(get_context_from(&buf, number_of_bytes));
                        debug!("update from server: P0; {:?}", player);
                        return Ok(ServerUpdate::Update(player));
                    }
                    _ => Ok(ServerUpdate::Nothing),
                }
            }
        }
        _ => {
            trace!("update from server, nothing to do");
            Ok(ServerUpdate::Nothing)
        }
    }
}

fn get_operation_from(buffer: &[u8]) -> &str {
    str::from_utf8(&buffer[0..3]).unwrap()
}

fn get_context_from(buffer: &[u8], size: usize) -> &str {
    str::from_utf8(&buffer[3..size]).unwrap()
}
