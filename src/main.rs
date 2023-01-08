mod client_listener;
mod components;
mod keyboard;
mod renderer;
mod syncer;
mod ui;

use chrono::Utc;
use futures::StreamExt;
use log::{debug, error, info, trace};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::sys::{CurrentTime, SDL_GetWindowSize};
use sdl2::EventPump;
use std::hash::Hash;
use std::net::{SocketAddr, UdpSocket};
use std::process::exit;
use std::sync::Arc;
use std::{env, str};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
// "self" imports the "image" module itself as well as everything else we listed
use sdl2::image::{self, InitFlag, LoadTexture};
use std::collections::{HashMap, HashSet, VecDeque};

use specs::prelude::*;

use std::time::Duration;

use crate::components::*;

fn initialize_main_player(world: &mut World, player: Player) -> Entity {
    world
        .create_entity()
        .with(KeyboardControlled)
        .with(EntityResource(EntityType::Main))
        .with(player)
        .build()
}

fn initialize_player(world: &mut World, player: Player) -> Entity {
    world
        .create_entity()
        .with(EntityResource(EntityType::Other))
        .with(player)
        .build()
}

fn main() {
    env_logger::init();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .worker_threads(10)
        .build()
        .unwrap();
    let rt = runtime.handle();
    rt.block_on(async move {
        let (tx_game_state, rx_game_state) = tokio::sync::mpsc::channel::<String>(200);
        let (tx_move_command, mut rx_move_command) = tokio::sync::mpsc::channel::<String>(200);
        let sender_task = rt.spawn(async move {
            println!("started sender task");
            let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 9978))).unwrap();
            socket.connect("127.0.0.1:8877").unwrap();
            loop {
                match rx_move_command.try_recv() {
                    Ok(msg) => {
                        // println!("received move command");
                        match socket.send(&msg.into_bytes()) {
                            Ok(_) => {}
                            Err(error) => {
                                error!("sending command: {}", error)
                            }
                        }
                    }
                    Err(_) => {
                        // println!("got nothing")
                    }
                }
                ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 20));
            }
        });

        let receiver_task = rt.spawn(async move {
            println!("started receiver task");
            let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 9979))).unwrap();
            socket.set_nonblocking(false).unwrap();
            let tx_game_state_clone = tx_game_state.clone();
            let mut timer = Utc::now().timestamp();
            loop {
                let mut buf = [0; 2000];
                match socket.recv(&mut buf) {
                    Ok(number_of_bytes) => {
                        debug!("Receiver got {} bytes.", number_of_bytes);
                        if number_of_bytes == 1 {
                            // return Ok(ServerUpdate::Nothing);
                        } else {
                            match get_operation_from(&buf) {
                                "L1;" => {
                                    let player_id: &str = get_context_from(&buf, number_of_bytes);
                                    debug!("get op L1; {}", player_id);
                                    // return Ok(ServerUpdate::Login(player_id.to_string()));
                                }
                                "P0;" => {
                                    // println!("P0; {}", get_context_from(&buf, number_of_bytes));
                                    let players_updates =
                                        match serde_json::from_str::<HashMap<String, Player>>(
                                            get_context_from(&buf, number_of_bytes),
                                        ) {
                                            Ok(value) => {
                                                // println!("{value:?}");
                                                value
                                            }
                                            Err(error) => {
                                                println!("{error}");
                                                HashMap::new()
                                            }
                                        };

                                    // if timer > Utc::now().timestamp() - 300 {
                                    // println!(
                                    //     "update from server: P0; {:?}",
                                    //     serde_json::to_string(&players_updates).unwrap()
                                    // );
                                    match tx_game_state_clone
                                        .send(serde_json::to_string(&players_updates).unwrap())
                                        .await
                                    {
                                        Ok(value) => {}
                                        Err(error) => {
                                            println!("error sending: {}", error)
                                        }
                                    }
                                    timer = Utc::now().timestamp();
                                }
                                // return Ok(ServerUpdate::Update(players_updates));
                                // }
                                _ => {}
                            }
                        }
                    }
                    _ => {
                        trace!("Receiver found nothing.");
                        // Ok(ServerUpdate::Nothing)
                    }
                }
                ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 20));
            }
        });
        let game_loop_task = rt.spawn(async move {
            let sdl_context = sdl2::init().unwrap();
            let video_subsystem = sdl_context.video().unwrap();
            let _image_context = image::init(InitFlag::PNG | InitFlag::JPG).unwrap();

            let window = video_subsystem
                .window("game tutorial", DIMENSION.width, DIMENSION.height)
                .position(1000, 0)
                .build()
                .expect("could not initialize video subsystem");

            let canvas = window
                .into_canvas()
                .build()
                .expect("could not make a canvas");

            let texture_creator = canvas.texture_creator();

            let mut dispatcher = DispatcherBuilder::new()
                // .with(client_listener::ClientListener, "ClientListener", &[])
                // .with(health_checker::HealthChecker, "HealthChecker", &[])
                .with(keyboard::Keyboard, "Keyboard", &[])
                // .with(physics::Physics, "Physics", &["Keyboard"])
                // .with(animator::Animator, "Animator", &["Keyboard"])
                // .with(physics::Physics, "Physics", &[])
                // .with(animator::Animator, "Animator", &[])
                .build();

            let mut world = World::new();
            dispatcher.setup(&mut world);
            client_listener::SystemData::setup(&mut world);
            // status::SystemData::setup(&mut world);
            // sprites::SystemData::setup(&mut world);
            renderer::SystemData::setup(&mut world);
            ui::SystemData::setup(&mut world);

            // Initialize resource
            let server_update: Option<ServerUpdate> = None;
            let movement_command: Option<MovementCommand> = None;
            let shoot_command: Option<AttackCommand> = None;
            world.insert(movement_command);
            world.insert(server_update);
            world.insert(shoot_command);

            let main_player = initialize_main_player(&mut world, Player::default());
            world.insert(main_player);

            // world.create_entity().with(UiComponent {}).build();

            game_loop(
                main_player,
                rx_game_state,
                tx_move_command,
                world,
                canvas,
                dispatcher,
                sdl_context.event_pump().unwrap(),
            );
        });
        let (_, _, _) = tokio::join!(sender_task, receiver_task, game_loop_task);
    });
}

fn game_loop<'a>(
    main_player: Entity,
    mut rx_game_state: Receiver<String>,
    tx_move_command: Sender<String>,
    mut world: World,
    mut canvas: WindowCanvas,
    mut dispatcher: Dispatcher<'a, 'a>,
    mut event_pump: EventPump,
) {
    let mut entities: HashSet<String> = HashSet::new();
    let mut movements: VecDeque<MovementCommand> = VecDeque::new();
    let mut attacks: VecDeque<AttackCommand> = VecDeque::new();
    let recv_socket: UdpSocket = UdpSocket::bind("127.0.0.1:9094").unwrap();
    recv_socket.connect("127.0.0.1:8877").unwrap();
    match recv_socket.send(format!("{};L1;blub_id1;1;player", Utc::now().timestamp()).as_bytes()) {
        Ok(number_of_bytes) => {
            trace!("sent {} bytes to login", number_of_bytes);
            // match recv_socket.recv_from(&mut []) {
            //     Ok(_) => {}
            //     Err(error) => {
            //         error!("recv login msg: {}", error)
            //     }
            // }
        }
        Err(error) => {
            error!("send login msg: {}", error)
        }
    }
    let mut last_movement_command: MovementCommand = MovementCommand::Stop;
    let sync_socket = UdpSocket::bind("127.0.0.1:9901").unwrap();
    sync_socket.connect("127.0.0.1:8866").unwrap();
    'running: loop {
        let mut ents = entities.clone();
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
            *movements.front_mut().unwrap_or(&mut MovementCommand::Stop);
        *world.write_resource() = Some(movement_command);
        if last_movement_command != movement_command {
            // println!("sending {movement_command:?} to tx_move_command");
            match movement_command {
                MovementCommand::Move(direction) => {
                    let msg = format!(
                        "{};M0;blub_id1;{}",
                        Utc::now().timestamp_millis(),
                        direction
                    );
                    // println!("moved {msg}");
                    match tx_move_command.try_send(msg) {
                        Ok(_) => {
                            // println!("sent to tx_move_command")
                        }
                        Err(error) => {
                            // println!("{error}")
                        }
                    }
                }
                MovementCommand::Stop => {
                    send_player_stationary(tx_move_command.clone());
                    syncer::sync_pos_to_server(&sync_socket, world.system_data()).unwrap();
                }
                _ => {}
            }
            last_movement_command = movement_command;
        }

        let update = serde_json::from_str::<HashMap<String, Player>>(
            &rx_game_state.try_recv().unwrap_or("no update".to_string()),
        )
        .unwrap_or(HashMap::new());
        // println!("game loop received: {:?}", update);
        // let entities = update
        //     .values()
        //     .map(|u| initialize_player(world, u.to_owned()));
        for player in update.values() {
            if player.id != "blub_id1" && !ents.contains(&player.id) {
                trace!("initialize_player");
                initialize_player(&mut world, player.clone());
                ents.insert(player.clone().id);
            }
        }
        world.insert(main_player);
        let players = update.into_values().collect::<Vec<Player>>();
        world.insert(Some(ServerUpdate::Update(players)));
        // Update
        dispatcher.dispatch(&mut world);
        world.maintain();

        // Render
        canvas.set_draw_color(Color::RGB(65, 64, 255));
        canvas.clear();

        // status::draw_to_canvas(&mut canvas, world.system_data()).unwrap();
        client_listener::draw_to_canvas(&mut canvas, world.system_data()).unwrap();
        renderer::draw_to_canvas(&mut canvas, world.system_data()).unwrap();
        ui::draw_to_canvas(&mut canvas, Color::RGB(65, 255, 255), world.system_data()).unwrap();

        canvas.present();
        // for ent in &ents {
        //     world.remove::<Entity>();
        // }

        // Time management!
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 20));
        entities = ents;
    }
    exit(0);
}

fn send_player_stationary(tx_move_command: Sender<String>) {
    let msg = format!(
        "{};M0;blub_id1;{}",
        Utc::now().timestamp(),
        Direction::Stationary
    );

    tx_move_command.try_send(msg).unwrap();
    // match socket.send(&msg.into_bytes()) {
    //     Ok(_) => {
    //         trace!("Send Stationary successful.");
    //         // match socket.recv(&mut []) {
    //         //     Ok(_) => {}
    //         //     Err(error) => {
    //         //         error!("ack Stationary command: {}", error)
    //         //     }
    //         // }
    //     }
    //     Err(error) => {
    //         error!("sending Stationary command: {}", error)
    //     }
    // }
}

fn get_operation_from(buffer: &[u8]) -> &str {
    str::from_utf8(&buffer[0..3]).unwrap()
}

fn get_context_from(buffer: &[u8], size: usize) -> &str {
    str::from_utf8(&buffer[3..size]).unwrap()
}
