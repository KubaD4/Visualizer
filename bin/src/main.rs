use lazy_static::lazy_static;
use piston_window::{
    clear, Button, G2d, Glyphs, Key, MouseScrollEvent, OpenGL, PistonWindow, PressEvent,
    ReleaseEvent, Size, WindowSettings,
};
use rand::{thread_rng, Rng};
use robotics_lib::energy::Energy;
use robotics_lib::event::events::Event;
use robotics_lib::interface::Direction::{Down, Left, Right, Up};
use robotics_lib::interface::Tools;
use robotics_lib::interface::{destroy, go, robot_view, Direction};
use robotics_lib::interface::{put, robot_map};
use robotics_lib::runner::backpack::BackPack;
use robotics_lib::runner::{Robot, Runnable, Runner};
use robotics_lib::world::coordinates::Coordinate;
use robotics_lib::world::environmental_conditions::EnvironmentalConditions;
use robotics_lib::world::environmental_conditions::WeatherType::{Rainy, Sunny};
use robotics_lib::world::tile::Content::{
    Bank, Bin, Building, Bush, Coin, Crate, Fire, Fish, Garbage, JollyBlock, Market, Rock,
    Scarecrow, Tree, Water,
};
use robotics_lib::world::tile::TileType::{
    DeepWater, Grass, Hill, Lava, Mountain, Sand, ShallowWater, Snow, Street, Teleport,
};
use robotics_lib::world::tile::{Content, Tile};
use robotics_lib::world::world_generator::Generator;
use robotics_lib::world::World;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{mpsc, Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::Duration;
use Visualizer_Tests::GifFrame::Frames as OtherFrames;
use Visualizer_Tests::Grid::*;
use Visualizer_Tests::Util::{
    backpack_to_text, clear_png_files_in_directory, convert_content_to_color_matrix,
    convert_to_color_matrix, play_sound, Infos, DEFAULT_PNGS_PATH,
};

lazy_static! {
    /// List of robot_map converted into frames to create a gif of all the movements
    static ref INIT_FRAMES: Mutex<OtherFrames> = Mutex::new(OtherFrames::new());

    /// List of coordinates that the robot has seen so far
    static ref CURRENT_ROBOT_MAP: Mutex<Option<Vec<Vec<Option<Tile>>>>>  = Mutex::new(None);

    static ref CURRENT_ROBOT_VIEW: Mutex<Vec<Vec<Option<Tile>>>> = Mutex::new(vec![vec![None; 3]; 3]);

    static ref CURRENT_ROBOT_BACKPACK: Mutex<String>  = Mutex::new(String::new());  //it cant be mutex of Backpack because i cant use Backpack::new(), that is pub(crate)

    static ref SCORE: Mutex<f32> = Mutex::new(0.0);

    static ref CURRENT_ROBOT_COORDINATES: Mutex<(usize,usize)> = Mutex::new((0,0));
}

const DEFAULT_FONT_PATH: &str = "../font/font.otf";

pub const MAP_DIM: usize = MAP_SIZE;
const PLAY_SOUNDS: bool = false;
struct MyRobot {
    robot: Robot,
    iterations: Rc<Cell<usize>>,
}

fn main() {
    // Channel to send to the visualizer the robot_map while the robot moves in the process_tick()
    let (matrix_sender, matrix_receiver) = mpsc::channel();

    //IMPLEMENTATION OF THE WORLDGENERATOR AND PROCESS TICK
    thread::spawn(|| {
        struct WorldGenerator {
            size: usize,
        }
        impl WorldGenerator {
            fn init(size: usize) -> Self {
                WorldGenerator { size }
            }
        }
        impl Generator for WorldGenerator {
            fn gen(
                &mut self,
            ) -> (
                Vec<Vec<Tile>>,
                (usize, usize),
                EnvironmentalConditions,
                f32,
                Option<HashMap<Content, f32>>,
            ) {
                let mut rng = thread_rng();
                let mut map: Vec<Vec<Tile>> = Vec::new();
                // Initialize the map with default tiles
                for _ in 0..self.size {
                    let mut row: Vec<Tile> = Vec::new();
                    for _ in 0..self.size {
                        let i_tiletype = 3; //rng.gen_range(2..=5);//rng.gen_range(0..TileType::iter().len());
                        let i_content = rng.gen_range(0..=2); //rng.gen_range(0..Content::iter().len());
                        let i_size = rng.gen_range(0..=2);
                        let tile_type = match i_tiletype {
                            0 => DeepWater,
                            1 => ShallowWater,
                            2 => Sand,
                            3 => Grass,
                            4 => Street,
                            5 => Hill,
                            6 => Mountain,
                            7 => Snow,
                            8 => Lava,
                            9 => Teleport(false),
                            _ => Grass,
                        };
                        let content = match i_content {
                            0 => Rock(i_size),
                            1 => Tree(i_size),
                            2 => Garbage(i_size),
                            3 => Fire,
                            4 => Coin(2),
                            5 => Bin(2..3),
                            6 => Crate(2..3),
                            7 => Bank(3..54),
                            8 => Water(20),
                            10 => Fish(3),
                            11 => Market(20),
                            12 => Building,
                            13 => Bush(2),
                            14 => JollyBlock(2),
                            15 => Scarecrow,
                            _ => Content::None,
                        };
                        row.push(Tile {
                            tile_type,
                            content,
                            elevation: 0,
                        });
                    }
                    map.push(row);
                }
                let environmental_conditions =
                    EnvironmentalConditions::new(&[Sunny, Rainy], 15, 12).unwrap();

                let max_score = rand::random::<f32>();

                (map, (0, 0), environmental_conditions, max_score, None)
            }
        }
        impl Runnable for MyRobot {
            fn process_tick(&mut self, world: &mut World) {
                let index = self.iterations.get(); //for debug

                //example of AI to degub visualizer
                //let environmental_conditions = look_at_sky(world);
                let direction;
                if index <= 25 {
                    direction = Direction::Right;
                } else if index > 25 && index <= 50 {
                    direction = Direction::Down;
                } else if index > 50 && index <= 70 {
                    direction = Direction::Up;
                } else if index > 70 && index <= 140 {
                    direction = Direction::Left;
                } else if index > 140 && index <= 180 {
                    direction = Direction::Down;
                } else if index > 180 && index <= 300 {
                    direction = Direction::Right;
                } else if index > 450 && index <= 470 {
                    direction = Direction::Up;
                } else {
                    let mut rng = rand::thread_rng();
                    let rand_direction = rng.gen_range(0..=3);
                    match rand_direction {
                        0 => direction = Direction::Right,
                        1 => direction = Direction::Left,
                        2 => direction = Direction::Down,
                        3 => direction = Direction::Up,
                        _ => direction = Direction::Right,
                    }
                }
                let _ = go(self, world, direction);
                if index % 2 == 0 {
                    let _ = destroy(self, world, Direction::Right);
                    let _ = destroy(self, world, Direction::Left);
                    let _ = destroy(self, world, Direction::Up);
                    let _ = destroy(self, world, Direction::Down);
                } else {
                    let boh = thread_rng().gen_range(0..=1);
                    if boh == 0 {
                        if let Err(_) = put(self, world, Rock(1), 1, Right) {
                            if let Err(_) = put(self, world, Rock(1), 1, Left) {
                                if let Err(_) = put(self, world, Rock(1), 1, Down) {
                                    if let Err(_) = put(self, world, Rock(1), 1, Up) {}
                                }
                            }
                        }
                    } else {
                        if let Err(_) = put(self, world, Garbage(1), 1, Right) {
                            if let Err(_) = put(self, world, Garbage(1), 1, Left) {
                                if let Err(_) = put(self, world, Garbage(1), 1, Down) {
                                    if let Err(_) = put(self, world, Garbage(1), 1, Up) {}
                                }
                            }
                        }
                    }
                }

                //non modificare le seguenti righe
                //the following is something like *INITIAL_ROBOT_MAP.lock().unwrap() = robot_map(world);
                if let Err(e) = update_robot_map(world) {
                    eprintln!("{}", e)
                }
                if let Err(e) = update_robot_view(self, world) {
                    eprintln!("{}", e)
                }

                //debug
                println!("{}", index);
                self.iterations.set(index + 1);
            }

            //non modificare le seguenti righe (potete aggiungere roba se vi serve per debug ma non rimuovete le chiamate a metodi ecc)
            fn handle_event(&mut self, event: Event) {
                match event {
                    Event::Ready => {
                        //clears the path were pngs are writted/read from to produce the gif
                        if let Err(e) = clear_png_files_in_directory(DEFAULT_PNGS_PATH) {
                            eprintln!("Couldnt clear png path: {}", e)
                        }
                    }
                    Event::Terminated => {}
                    Event::TimeChanged(_) => {
                        if PLAY_SOUNDS {
                            thread::spawn(|| {
                                if let Err(e) = play_sound("/prova.ogg", 0.5) {
                                    eprintln!("error playing sound for TimeChanged: {}", e)
                                }
                            });
                        }
                    }
                    Event::DayChanged(_) => {}
                    Event::EnergyRecharged(_) => {}
                    Event::EnergyConsumed(_) => {}
                    Event::Moved(_, _) => {
                        let new_coord = self.get_coordinate();
                        if let Err(e) = update_robot_coord(new_coord) {
                            eprintln!(
                                "couldnt lock CURRENT_ROBOT_COORDINATES in HandleEvent(Moved): {}",
                                e
                            )
                        }

                        println!("moved");

                        //INIT_FRAMES.lock().unwrap().add_frame(&CURRENT_ROBOT_MAP.lock().unwrap());
                        match INIT_FRAMES.lock() {
                            Ok(mut init_frame_lock) => match &CURRENT_ROBOT_MAP.lock() {
                                Ok(current_map_lock) => init_frame_lock.add_frame(current_map_lock),
                                Err(e) => {
                                    eprintln!(
                                        "Couldnt lock CURRENT_ROBOT_MAP in HandleEvent(Moved): {}",
                                        e
                                    )
                                }
                            },
                            Err(e) => {
                                eprintln!("Couldnt lock INIT_FRAMES in HandleEvent(Moved): {}", e)
                            }
                        }
                    }
                    Event::TileContentUpdated(_, _) => {}
                    Event::AddedToBackpack(_, _) => {
                        let current_backpack = self.get_backpack();
                        if let Err(e) = update_robot_backpack(current_backpack) {
                            eprintln!("Couldnt update backpack: {}", e)
                        }
                        //the function must sleep for a while to allow the sound to play (0.2s)
                        // so it is better to do it in another thread and let the main function keep going
                        if PLAY_SOUNDS {
                            thread::spawn(|| {
                                if let Err(e) = play_sound("/AddedToBackpack.ogg", 0.5) {
                                    eprintln!("error playing sound for AddedToBackpack: {}", e)
                                }
                            });
                        }
                    }
                    Event::RemovedFromBackpack(_, _) => {
                        let current_backpack = self.get_backpack();
                        if let Err(e) = update_robot_backpack(current_backpack) {
                            eprintln!("Couldnt update backpack: {}", e)
                        }
                        if PLAY_SOUNDS {
                            thread::spawn(|| {
                                if let Err(e) = play_sound("/RemovedFromBackpack.ogg", 0.5) {
                                    eprintln!("error playing sound for AddedToBackpack: {}", e)
                                }
                            });
                        }
                    }
                }
            }

            fn get_energy(&self) -> &Energy {
                &self.robot.energy
            }
            fn get_energy_mut(&mut self) -> &mut Energy {
                &mut self.robot.energy
            }

            fn get_coordinate(&self) -> &Coordinate {
                &self.robot.coordinate
            }
            fn get_coordinate_mut(&mut self) -> &mut Coordinate {
                &mut self.robot.coordinate
            }

            fn get_backpack(&self) -> &BackPack {
                &self.robot.backpack
            }
            fn get_backpack_mut(&mut self) -> &mut BackPack {
                &mut self.robot.backpack
            }
        }

        let r = MyRobot {
            robot: Robot::new(),
            iterations: Rc::new(Cell::new(0)),
        };
        struct Tool;
        impl Tools for Tool {}
        let mut generator = WorldGenerator::init(MAP_DIM);
        let i = r.iterations.clone();
        let mut run = Runner::new(Box::new(r), &mut generator);
        loop {
            match run {
                Ok(ref mut runner) => {
                    let _ = runner.game_tick();
                    if i.get() > 500 {
                        match INIT_FRAMES.lock() {
                            Ok(lock) => {
                                if let Err(e) = lock.from_frames_to_gif() {
                                    println!("error creating the gif: {}", e)
                                }
                            }
                            Err(e) => {
                                eprintln!("Couldnt lock INIT_FRAMES implies impossible to create a gif: {}",e)
                            }
                        }
                        break;
                    }
                }
                Err(e) => println!("{:?}", e),
            }
        }
    });

    let window_size = Size::from((WINDOW_SIZE.0 as u32, WINDOW_SIZE.1 as u32));
    println!("building window");
    let mut window: PistonWindow = WindowSettings::new("Grid", window_size)
        .exit_on_esc(true)
        .graphics_api(OpenGL::V3_2)
        .build()
        .unwrap();

    //let mut glyphs = window.load_font("/Users/kuba/CLionProjects/Patrignani_Project_copia2/Visualizer_Tests_Andrea/font/font.otf").unwrap();
    let mut glyphs = match window.load_font(DEFAULT_FONT_PATH) {
        Ok(_glyphs) => Some(_glyphs),
        Err(e) => {
            eprintln!("Couldnt load glyphs: {}", e);
            None
        }
    };
    // initiate color matrix
    let initial_color_matrix = Arc::new(Mutex::new(vec![
        vec![[0.0, 0.0, 0.0, 1.0]; MAP_DIM];
        MAP_DIM
    ]));
    let initial_content_color_matrix =
        Arc::new(Mutex::new(vec![
            vec![[0.0, 0.0, 0.0, 1.0]; MAP_DIM];
            MAP_DIM
        ]));

    //send the new map
    let matrix_sender = matrix_sender.clone();
    thread::spawn(move || {
        loop {
            // Get the updated tile matrix
            let updated_tile_matrix = match CURRENT_ROBOT_MAP.lock() {
                Ok(lock) => lock.clone(),
                Err(e) => {
                    eprintln!("Couldnt lock CURRENT_ROBOT_MAP in sender thread: {} -> value has been set to a default value", e);
                    None
                }
            };

            //create a matrix containing the colors for each tile_type/contend at index [i][j]
            convert_content_to_color_matrix(&updated_tile_matrix, &initial_content_color_matrix);
            convert_to_color_matrix(&updated_tile_matrix, &initial_color_matrix);

            let matrix_to_be_sent = initial_color_matrix.lock().unwrap().clone();
            let matrix_content_to_be_sent = initial_content_color_matrix.lock().unwrap().clone();
            let coord_to_be_sent = match CURRENT_ROBOT_COORDINATES.lock() {
                Ok(lock) => *lock,
                Err(e) => {
                    eprintln!("Couldnt lock CURRENT_ROBOT_COORDINATES in sender thread: {} -> coordinates has been set to a default value:(0,0)", e);
                    (0, 0)
                }
            };

            let view_to_be_sent = match CURRENT_ROBOT_VIEW.lock() {
                Ok(lock) => lock.clone(),
                Err(e) => {
                    eprintln!("Couldnt lock CURRENT_ROBOT_VIEW in sender thread: {} -> robot_view has been set to a default value", e);
                    vec![vec![None; 3]; 3]
                }
            };

            let backpack_to_be_sent = match CURRENT_ROBOT_BACKPACK.lock() {
                Ok(lock) => lock.clone(),
                Err(e) => {
                    eprintln!("Couldnt lock CURRENT_ROBOT_BACKPACK in sender thread: {} -> robot_backpack has been set to a default value", e);
                    String::new()
                }
            };

            match matrix_sender.send((
                matrix_to_be_sent,
                matrix_content_to_be_sent,
                coord_to_be_sent,
                view_to_be_sent,
                backpack_to_be_sent,
            )) {
                Ok(_) => {
                    // The send was successful
                }
                Err(e) => {
                    // Handle the error
                    eprintln!("Failed to send data through the channel: {}\nWindow will use the default value for each informaton", e);
                }
            }
            thread::sleep(Duration::from_secs_f64(0.02));
        }
    });

    let mut current_tuple_information: Infos = (
        vec![vec![[0.0, 0.0, 0.0, 1.0]; MAP_DIM]; MAP_DIM],
        vec![vec![[0.0, 0.0, 0.0, 1.0]; MAP_DIM]; MAP_DIM],
        (0, 0),
        vec![vec![None; 3]; 3],
        String::new(),
    );

    let mut scroll_offset = [0.0, 0.0];
    let mut zoom_factor = 1.0;
    //let mut zoom_in_pressed = false;
    //let mut zoom_out_pressed = false;

    while let Some(event) = window.next() {
        if let Ok(updated_information) = matrix_receiver.try_recv() {
            current_tuple_information = updated_information;
        }

        let coord_text = format!(
            "robot coordinates:({},{})",
            current_tuple_information.2 .0, current_tuple_information.2 .1
        );
        //handle the + and - key keep pressed to zoom in or out
        /*
        event.update(|_| {
            if zoom_in_pressed {
                zoom_factor += ZOOM_AMOUNT;
            }
            if zoom_out_pressed {
                zoom_factor -= ZOOM_AMOUNT;
                zoom_factor = zoom_factor.max(0.1); // Prevent zooming out too much
            }
        });
         */

        //keyboard-scroll handling
        if let Some(Button::Keyboard(key)) = event.press_args() {
            match key {
                Key::Up => scroll_offset[1] += SCROLL_AMOUNT,
                Key::Down => scroll_offset[1] -= SCROLL_AMOUNT,
                Key::Left => scroll_offset[0] -= SCROLL_AMOUNT,
                Key::Right => scroll_offset[0] += SCROLL_AMOUNT,
                _ => {}
            }
        }

        //mouse-zoom handling
        if let Some(mouse_event) = event.mouse_scroll_args() {
            zoom_factor += mouse_event[1] * ZOOM_AMOUNT;
            zoom_factor = zoom_factor.max(0.1); // Prevent zooming out too much
        }

        //keyboard-zoom handling
        if let Some(Button::Keyboard(key)) = event.press_args() {
            match key {
                Key::Equals => {
                    //oom_in_pressed = true;
                    zoom_factor += ZOOM_AMOUNT;
                }
                Key::Minus => {
                    //zoom_out_pressed = true;
                    zoom_factor -= ZOOM_AMOUNT;
                    zoom_factor = zoom_factor.max(0.1); // Prevent zooming out too much
                }
                _ => {}
            }

            //handling the +/- release-button acction to stop zoomming in or out and the scroll
            if let Some(Button::Keyboard(key)) = event.release_args() {
                match key {
                    //Key::Equals => zoom_in_pressed = false,
                    //Key::Minus => zoom_out_pressed = false,
                    _ => {}
                }
            }
        }

        window.draw_2d(&event, |context, graphics, device| {
            clear([0.0, 0.0, 0.0, 1.0], graphics);

            draw_optimized_grid(
                &current_tuple_information.0,
                /*&current_tuple_matrix.1 ,*/ context,
                graphics,
                (MAP_DIM, MAP_DIM),
                RECT_SIZE,
                scroll_offset,
                zoom_factor,
            );

            //draw_optimized_grid_with_limited_circles(&current_tuple_information.0, &current_tuple_information.1 ,context, graphics, RECT_SIZE, scroll_offset, zoom_factor, current_tuple_information.2);

            if let Some(ref mut glyphs) = glyphs {
                //Draw text
                /*
                draw_text(
                    &context,
                    graphics,
                    glyphs,
                    [1.0; 4],
                    [50, (MAP_DIM as f64 * RECT_SIZE * zoom_factor - scroll_offset[1]) as u32],
                    coord_text.as_str(),
                );
                 */
                let starting_text_x:u32 = 50;
                //let starting_text_x:u32 = -scroll_offset[0] as u32 + 50;
                //let starting_text_y:u32 = 490;
                let starting_text_y:u32 = ((MAP_DIM as f64 * RECT_SIZE /* * zoom_factor - scroll_offset[1]*/)+35.0) as u32;
                //coordinates
                draw_text(
                    &context,
                    graphics,
                    glyphs,
                    [1.0; 4],
                    [starting_text_x, starting_text_y],
                    coord_text.as_str(),
                );

                //robot view
                draw_texts(
                    &current_tuple_information.3.get(0),
                    &current_tuple_information.3.get(1),
                    &current_tuple_information.3.get(2),
                    &context,
                    graphics,
                    glyphs,
                    starting_text_y
                );

                //backpack
                draw_text(
                    &context,
                    graphics,
                    glyphs,
                    [1.0; 4],
                    [starting_text_x, 30+starting_text_y+25*5],
                    current_tuple_information.4.as_str(),
                );
                glyphs.factory.encoder.flush(device);
            }
        });
    }
}

fn update_robot_view<'a>(
    robot: &'a impl Runnable,
    world: &'a World,
) -> Result<(), PoisonError<MutexGuard<'a, Vec<Vec<Option<Tile>>>>>> {
    let new_view = robot_view(robot, world);
    match CURRENT_ROBOT_VIEW.lock() {
        Ok(lock) => {
            let mut view_lock = lock;
            *view_lock = new_view;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn update_robot_map(
    world: &World,
) -> Result<(), PoisonError<MutexGuard<Option<Vec<Vec<Option<Tile>>>>>>> {
    let new_map = robot_map(world); // Doing it before the assignment in order to reduce the lock() time
    match CURRENT_ROBOT_MAP.lock() {
        Ok(lock) => {
            let mut map_lock = lock;
            *map_lock = new_map;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn update_robot_coord(
    new_coord: &Coordinate,
) -> Result<(), PoisonError<MutexGuard<(usize, usize)>>> {
    match CURRENT_ROBOT_COORDINATES.lock() {
        Ok(mut lock) => {
            *lock = (new_coord.get_row(), new_coord.get_col());
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn update_robot_backpack(back_pack: &BackPack) -> Result<(), PoisonError<MutexGuard<String>>> {
    match CURRENT_ROBOT_BACKPACK.lock() {
        Ok(mut lock) => {
            *lock = backpack_to_text(back_pack);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn draw_texts(
    vec1: &Option<&Vec<Option<Tile>>>,
    vec2: &Option<&Vec<Option<Tile>>>,
    vec3: &Option<&Vec<Option<Tile>>>,
    context: &piston_window::Context,
    graphics: &mut G2d,
    glyphs: &mut Glyphs,
    starting_text_y: u32,
) {
    let start_x: u32 = 50;
    let start_y: u32 = starting_text_y + 35;
    let offset: u32 = 25;
    draw_text(
        context,
        graphics,
        glyphs,
        [1.0; 4],
        [start_x, start_y],
        "Robot view :",
    );

    if let Some(maybe_vec) = vec1 {
        draw_text(
            context,
            graphics,
            glyphs,
            [1.0; 4],
            [start_x, start_y + offset],
            create_text_view(maybe_vec).as_str(),
        );
    }

    if let Some(maybe_vec) = vec2 {
        draw_text(
            context,
            graphics,
            glyphs,
            [1.0; 4],
            [start_x, start_y + offset * 2],
            create_text_view(maybe_vec).as_str(),
        );
    }

    if let Some(maybe_vec) = vec3 {
        draw_text(
            context,
            graphics,
            glyphs,
            [1.0; 4],
            [start_x, start_y + offset * 3],
            create_text_view(maybe_vec).as_str(),
        );
    }
}

fn create_text_view(vec: &Vec<Option<Tile>>) -> String {
    let mut result = String::new();
    for maybe_tile in vec {
        let content_str = match maybe_tile {
            Some(tile) => {
                format!("{:?} ", tile.content)
            } // Use Display formatting
            None => "[x]".to_string(), // Placeholder for None
        };
        result += &format!("{:<10} ", content_str); // Adjusted for uniform spacing
    }
    result
}
