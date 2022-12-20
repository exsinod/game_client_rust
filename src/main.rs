mod animator;
mod client_listener;
mod components;
mod health_checker;
mod keyboard;
mod physics;
mod sprites;
mod status;
mod ui;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};
use sdl2::EventPump;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::str;
use tokio;
// "self" imports the "image" module itself as well as everything else we listed
use sdl2::image::{self, InitFlag, LoadTexture};
use std::collections::{HashMap, VecDeque};

use specs::prelude::*;

use std::time::Duration;

use crate::components::*;

static SEND_SERVER_ADDR: &str = "127.0.0.1:8877";
static RECV_SERVER_ADDR: &str = "127.0.0.1:8878";
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Returns the row of the spritesheet corresponding to the given direction
fn direction_spritesheet_row(direction: Direction) -> i32 {
    match direction {
        Direction::Up => 3,
        Direction::Down => 0,
        Direction::Left => 1,
        Direction::Right => 2,
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

fn initialize_player(world: &mut World, player_spritesheet: usize) -> Entity {
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
        .with(Player::default())
        .with(Position(Point::new(0, 0)))
        .with(Velocity {
            speed: 0,
            direction: Direction::Right,
        })
        .with(Status {
            alive: true,
            health: 100,
        })
        .with(player_animation.right_frames[0].clone())
        .with(player_animation)
        .build()
}

fn initialize_enemy(world: &mut World, enemy_spritesheet: usize, position: Point) {
    let enemy_top_left_frame = Rect::new(0, 0, 32, 36);

    let enemy_animation = MovementAnimation {
        current_frame: 0,
        up_frames: character_animation_frames(
            enemy_spritesheet,
            enemy_top_left_frame,
            Direction::Up,
        ),
        down_frames: character_animation_frames(
            enemy_spritesheet,
            enemy_top_left_frame,
            Direction::Down,
        ),
        left_frames: character_animation_frames(
            enemy_spritesheet,
            enemy_top_left_frame,
            Direction::Left,
        ),
        right_frames: character_animation_frames(
            enemy_spritesheet,
            enemy_top_left_frame,
            Direction::Right,
        ),
    };

    world
        .create_entity()
        .with(Position(position))
        .with(Velocity {
            speed: 2,
            direction: Direction::Right,
        })
        .with(Status {
            alive: true,
            health: 100,
        })
        .with(enemy_animation.right_frames[0].clone())
        .with(enemy_animation)
        .build();
}

#[tokio::main]
async fn main() -> Result<()> {
    let runtime: ServerRuntime = ServerRuntime::new();
    let socket = &runtime.socket;
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = image::init(InitFlag::PNG | InitFlag::JPG)?;

    let window = video_subsystem
        .window("game tutorial", DIMENSION.width, DIMENSION.height)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window
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
        .with(physics::Physics, "Physics", &[])
        .with(animator::Animator, "Animator", &[])
        .build();

    let mut world = World::new();
    dispatcher.setup(&mut world);
    status::SystemData::setup(&mut world);
    sprites::SystemData::setup(&mut world);
    ui::SystemData::setup(&mut world);

    // Initialize resource
    // let player = Player::default();
    let client_command: Option<ClientCommand> = None;
    let server_update: Option<ServerUpdate> = None;
    let movement_command: Option<MovementCommand> = None;
    let shoot_command: Option<AttackCommand> = None;
    // world.insert(player);
    world.insert(movement_command);
    world.insert(server_update);
    world.insert(shoot_command);
    world.insert(client_command);

    let textures = [
        texture_creator.load_texture("assets/bardo.png")?,
        texture_creator.load_texture("assets/reaper.png")?,
    ];
    // First texture in textures array
    let player_spritesheet = 0;
    // Second texture in the textures array
    let enemy_spritesheet = 1;

    initialize_player(&mut world, player_spritesheet);
    // initialize_player(&mut world, player_spritesheet);
    // initialize_player(&mut world, player_spritesheet);
    // initialize_player(&mut world, player_spritesheet);

    // initialize_enemy(&mut world, enemy_spritesheet, Point::new(-150, -150));
    // initialize_enemy(&mut world, enemy_spritesheet, Point::new(150, -190));
    // initialize_enemy(&mut world, enemy_spritesheet, Point::new(-150, 170));

    world.create_entity().with(UiComponent {}).build();

    // let mut event_pump = sdl_context.event_pump()?;
    // let mut client_commands: VecDeque<ClientCommand> = VecDeque::new();
    // let mut movements: VecDeque<MovementCommand> = VecDeque::new();
    // let mut attacks: VecDeque<AttackCommand> = VecDeque::new();

    // let listener = UdpSocket::bind(SERVER_ADDR)?;
    socket.set_read_timeout(Some(Duration::new(0, 1_000_000_000u32 / 30)))?;
    socket.connect(RECV_SERVER_ADDR)?;
    socket.send(b"L1;blub_id")?;

    game_loop(
        world,
        canvas,
        &textures,
        dispatcher,
        sdl_context.event_pump()?,
        socket,
        VecDeque::new(),
        VecDeque::new(),
    )
    .await?;
    Ok(())
}

async fn game_loop<'a>(
    mut world: World,
    mut canvas: WindowCanvas,
    textures: &[Texture<'a>],
    mut dispatcher: Dispatcher<'a, 'a>,
    mut event_pump: EventPump,
    socket: &UdpSocket,
    mut movements: VecDeque<MovementCommand>,
    mut attacks: VecDeque<AttackCommand>,
) -> Result<()> {
    let mut entities: HashMap<String, Entity> = HashMap::new();
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
                println!("move command: {:?}", msg);
                socket.connect(RECV_SERVER_ADDR)?;
                match socket.send(&msg.into_bytes()) {
                    Ok(_) => match socket.recv(&mut []) {
                        Ok(_) => {}
                        _ => {}
                    },
                    _ => {}
                }
            }
            MovementCommand::Stop => {}
        }

        let shoot_command: Option<AttackCommand> =
            Some(attacks.pop_front().unwrap_or(AttackCommand::Stop));
        *world.write_resource() = shoot_command;

        let server_update: Option<ServerUpdate> = Some(update_from_server(&socket)?);
        let server_update_clone = server_update.clone();
        *world.write_resource() = server_update;

        match &server_update_clone.unwrap() {
            ServerUpdate::Nothing => {
                //     entities
                //         .values()
                //         .for_each(|entity| remove_entities(&mut world, *entity));
                //     entities.clear();
            }
            ServerUpdate::Login(player_id) => {
                // if !entities.contains_key(player_id) {
                //     let new_player = initialize_player(&mut world, 0);
                //     entities.insert(player_id.to_string(), new_player);
                // }
                println!("Login for {:?}", player_id);
            }
            ServerUpdate::Update(player) => {
                // if !entities.contains_key(&player.id) {
                //     let new_player = initialize_player(&mut world, 0);
                //     entities.insert(player.id.to_string(), new_player);
                // }
                println!("update with {:?}", player);
            }
        }
        // let p_en = initialize_player(&mut world, 0);
        // println!(
        //     "{:?} --- player: {:?}",
        //     world.entities().join().collect::<Vec<_>>(),
        //     world.entities().entity(p_en.id())
        // );
        // let player_entity = world.entities().entity(99);
        // println!(
        //     "{:?} --- {:?}",
        //     world.entities().join().collect::<Vec<_>>(),
        //     player_entity.gen().is_alive()
        // );
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
fn remove_entities(world: &mut World, entity: Entity) {
    world.entities().delete(entity).unwrap();
}

fn update_from_server(socket: &UdpSocket) -> Result<ServerUpdate> {
    let mut buf = [0; 128];
    socket.connect(SEND_SERVER_ADDR)?;
    let (number_of_bytes, src) = socket.recv_from(&mut buf).unwrap_or((
        1,
        SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 1),
    ));
    socket.send_to(&[0], src)?;
    if number_of_bytes == 1 {
        return Ok(ServerUpdate::Nothing);
    } else {
        match get_operation_from(&buf) {
            "L1;" => {
                let player_id: &str = get_context_from(&buf, number_of_bytes);
                return Ok(ServerUpdate::Login(player_id.to_string()));
            }
            "P0;" => {
                let player = Player::from_str(get_context_from(&buf, number_of_bytes));
                println!("update from server: P0; {:?}", player);
                return Ok(ServerUpdate::Update(player));
            }
            _ => Ok(ServerUpdate::Nothing),
        }
    }
}

fn check_for_client_update(socket: &UdpSocket) -> Result<ClientCommand> {
    let mut buf = [0; 128];
    let number_of_bytes = socket.recv(&mut buf).unwrap_or(1);
    if number_of_bytes == 1 {
        return Ok(ClientCommand::Stop);
    }
    let src = "ip addr";
    println!("handling message from {}", src);
    match get_operation_from(&buf) {
        "L0;" => {
            new_login(get_context_from(&buf, number_of_bytes));
            Ok(ClientCommand::Stop)
        }
        "M0;" => {
            get_context_from(&buf, number_of_bytes);
            Ok(ClientCommand::Move(Direction::Up))
        }
        _ => {
            println!("Unknown command: {:?}", buf);
            Ok(ClientCommand::Stop)
        }
    }
}

fn new_login(player_data: &str) {
    let player_data_split: Vec<&str> = player_data.split(";").collect();
    println!("{:?}", player_data_split);
    // players.push(Playerx::new(player_data_split[0].to_string(), 1));
    println!("New user logs in.  Current players: ");
}

fn get_operation_from(buffer: &[u8]) -> &str {
    str::from_utf8(&buffer[0..3]).unwrap()
}

fn get_context_from(buffer: &[u8], size: usize) -> &str {
    str::from_utf8(&buffer[3..size]).unwrap()
}
