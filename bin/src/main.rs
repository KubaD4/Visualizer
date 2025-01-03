use std::collections::HashMap;
use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use std::time::Duration;

use piston_window::{Button, clear, G2d, Glyphs, Key, OpenGL, PistonWindow, PressEvent, ReleaseEvent, Size, UpdateEvent, WindowSettings};
use rand::{Rng, thread_rng};

use robotics_lib::interface::Tools;
use robotics_lib::runner::{Robot, Runner};
use robotics_lib::world::environmental_conditions::EnvironmentalConditions;
use robotics_lib::world::environmental_conditions::WeatherType::{Rainy, Sunny};
use robotics_lib::world::tile::{Content, Tile};
use robotics_lib::world::tile::Content::{
    Bank, Bin, Building, Bush, Coin, Crate, Fire, Fish, Garbage, JollyBlock, Market, Rock,
    Scarecrow, Tree, Water,
};
use robotics_lib::world::tile::TileType::{
    DeepWater, Grass, Hill, Lava, Mountain, Sand, ShallowWater, Snow, Street, Teleport,
};
use robotics_lib::world::world_generator::Generator;
use Visualizer::grid::*;
//use worldgen_unwrap::*;
//use worldgen_unwrap::*;
use Visualizer::robot::{ExampleRobot, Visualizable};
use Visualizer::util::{convert_content_to_color_matrix, convert_robot_content_view_to_color_matrix, convert_robot_view_to_color_matrix, convert_to_color_matrix, Infos};

const DEFAULT_FONT_PATH: &str = "../font/font.otf";

pub const MAP_DIM: usize = MAP_SIZE;

fn main() {
    // ROBOT ANDREA
    /*
    use andrea_ai::AndreaRobot;
    use clap::Parser;
    let args = andrea_ai::Args::parse();
    let r = AndreaRobot::new(Robot::new(), Arc::new(Mutex::new(0)), args);
    */
    // Channel to send to the visualizer the robot_map while the robot moves in the process_tick()
    let (matrix_sender, matrix_receiver) = mpsc::channel();
    let r = ExampleRobot::new(Robot::new(), Arc::new(Mutex::new(0)));
    let init_frames = r.get_init_frames().clone();
    let current_robot_map = r.get_current_robot_map().clone();
    let current_robot_view = r.get_current_robot_view().clone();
    let current_robot_backpack = r.get_current_robot_backpack().clone();
    let _score = r.get_score().clone();
    let current_robot_coordinates = r.get_current_robot_coordinates().clone();
    let current_robot_energy = r.get_current_energy().clone();

    //IMPLEMENTATION OF THE WORLDGENERATOR AND PROCESS TICK
    thread::spawn(move || {
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
                for i in 0..self.size {
                    let mut row: Vec<Tile> = Vec::new();
                    for _ in 0..self.size {
                        let i_tiletype;// = 3;//rng.gen_range(0..=9);//rng.gen_range(0..TileType::iter().len());
                        let i_content;//=rng.gen_range(0..=2); //rng.gen_range(0..Content::iter().len());


                        if i == 0 {
                            i_content = 16;  //first row
                        } else if i == 1 {
                            i_content = 1;  //second row
                        } else if i == 2 {
                            i_content = 16; //third row
                        } else {
                            i_content = 16  //other rows
                        }

                        /*
                        if i == 0 {
                            i_tiletype = 2;
                        } else if i == 2 {
                            i_tiletype = 1;
                        } else {
                            i_tiletype = 3
                        }   //first row filled with Sand, third row filled with Street, other rows are Grass
                         */

                        i_tiletype = 3; //only grass for better debug


                        let i_size = rng.gen_range(0..=20);
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
                            1 => Coin(i_size),
                            2 => Garbage(i_size),
                            3 => Fire,
                            4 => Tree(i_size),
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
        let mut generator = WorldGenerator::init(MAP_DIM);
        //fine

        // WorldGenerator del nostro gruppo
        //let mut generator = worldgen_unwrap::public::WorldgeneratorUnwrap::init(false, None);

        struct Tool;
        impl Tools for Tool {}
        let i = r.iterations.clone();
        let mut run = Runner::new(Box::new(r), &mut generator);
        loop {
            match run {
                Ok(ref mut runner) => {
                    //sleep(Duration::from_secs_f64(0.2));    //se si vuole che il robot vada più lento, scommentare + modificare il valore all'interno (f64)
                    let _ = runner.game_tick();
                    if *i.lock().unwrap() > 2000 {
                        match init_frames.lock() {
                            Ok(lock) => {
                                if let Err(e) = lock.convert_frames_to_gif() {
                                    println!("error creating the gif: {}", e)
                                }
                            }
                            Err(e) => {
                                eprintln!("Couldnt lock INIT_FRAMES implies impossible to create a gif: {}", e)
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
    let mut window: PistonWindow = WindowSettings::new("grid", window_size)
        .exit_on_esc(true)
        .resizable(false)
        .graphics_api(OpenGL::V3_2)
        .build()
        .unwrap();

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
            let updated_energy_to_be_sent = match current_robot_energy.lock() {
                Ok(lock) => lock.clone(),
                Err(e) => {
                    eprintln!("Couldnt lock CURRENT_ROBOT_ENERGY in sender thread: {} -> value has been set to a default value", e);
                    0
                }
            };

            let updated_score_to_be_sent = match _score.lock() {
                Ok(lock) => lock.clone(),
                Err(e) => {
                    eprintln!("Couldnt lock SCORE in sender thread: {} -> value has been set to a default value", e);
                    0.0f32
                }
            };

            let updated_tile_matrix = match current_robot_map.lock() {
                Ok(lock) => lock.clone(),
                Err(e) => {
                    eprintln!("Couldnt lock CURRENT_ROBOT_MAP in sender thread: {} -> value has been set to a default value", e);
                    None
                }
            };

            //create a matrix containing the colors for each tile_type/contend at index [i][j]
            convert_content_to_color_matrix(&updated_tile_matrix, &initial_content_color_matrix);
            convert_to_color_matrix(&updated_tile_matrix, &initial_color_matrix);

            let matrix_to_be_sent = match initial_color_matrix.lock() {
                Ok(lock) => lock.clone(),
                Err(e) => {
                    eprintln!("Couldnt lock in sender thread: {} -> default value", e);
                    vec![vec![[0.0,0.0,0.0,1.0];MAP_DIM];MAP_DIM]
                }
            };

            let matrix_content_to_be_sent = match initial_content_color_matrix.lock() {
                Ok(lock) => lock.clone(),
                Err(e) => {
                    eprintln!("Couldnt lock in sender thread: {} -> default value", e);
                    vec![vec![[0.0,0.0,0.0,1.0];MAP_DIM];MAP_DIM]
                }
            };

            let coord_to_be_sent = match current_robot_coordinates.lock() {
                Ok(lock) => *lock,
                Err(e) => {
                    eprintln!("Couldnt lock CURRENT_ROBOT_COORDINATES in sender thread: {} -> coordinates has been set to a default value:(0,0)", e);
                    (0, 0)
                }
            };

            let view_to_be_sent = match current_robot_view.lock() {
                Ok(lock) => lock.clone(),
                Err(e) => {
                    eprintln!("Couldnt lock CURRENT_ROBOT_VIEW in sender thread: {} -> robot_view has been set to a default value", e);
                    vec![vec![None; 3]; 3]
                }
            };

            let backpack_to_be_sent = match current_robot_backpack.lock() {
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
                updated_energy_to_be_sent,
                updated_score_to_be_sent
            )) {
                Ok(_) => {
                    // Successfully sent
                }
                Err(e) => {
                    // Handle the error
                    eprintln!("Failed to send data through the channel: {}\nWindow will use the default value for each informaton", e);
                }
            }
            thread::sleep(Duration::from_secs_f64(0.01));
            //thread::sleep(Duration::from_secs_f64(3.0));
        }
    });

    let mut current_tuple_information: Infos = (
        vec![vec![[0.0, 0.0, 0.0, 1.0]; MAP_DIM]; MAP_DIM],
        vec![vec![[0.0, 0.0, 0.0, 1.0]; MAP_DIM]; MAP_DIM],
        (0, 0),
        vec![vec![None; 3]; 3],
        String::new(),
        0usize,
        0.0f32
    );

    let mut scroll_offset = [0.0, 0.0];
    let mut zoom_factor = 1.0;
    //let mut zoom_in_pressed = false;
    //let mut zoom_out_pressed = false;
    let mut right_pressed = false;
    let mut left_pressed = false;
    let mut up_pressed = false;
    let mut down_pressed = false;
    let mut should_draw_robot_view = true;
    let mut should_draw_info_text = true;

    while let Some(event) = window.next() {
        if let Ok(updated_information) = matrix_receiver.try_recv() {
            current_tuple_information = updated_information;
        }

        let coord_text = format!(
            "robot coordinates:({},{})",
            current_tuple_information.2.1, current_tuple_information.2.0
        );
        let coord_as_f64 = (current_tuple_information.2.1 as f64, current_tuple_information.2.0 as f64);

        //key pressed handling
        if let Some(Button::Keyboard(key)) = event.press_args() {
            match key {
                Key::Up => {
                    scroll_offset[1] -= SCROLL_AMOUNT;
                    up_pressed = true;
                }
                Key::Down => {
                    scroll_offset[1] += SCROLL_AMOUNT;
                    down_pressed = true;
                }
                Key::Left => {
                    scroll_offset[0] -= SCROLL_AMOUNT;
                    left_pressed = true;
                }
                Key::Right => {
                    scroll_offset[0] += SCROLL_AMOUNT;
                    right_pressed = true;
                }
                Key::V => {
                    should_draw_robot_view = !should_draw_robot_view
                }
                Key::T => {
                    should_draw_info_text = !should_draw_info_text
                }
                _ => {}
            }
        }

        //scrolling with keys being keep pressed
        event.update(|_| {
            if left_pressed {
                scroll_offset[0] -= SCROLL_AMOUNT;
            }
            if right_pressed {
                scroll_offset[0] += SCROLL_AMOUNT;
            }
            if down_pressed {
                scroll_offset[1] += SCROLL_AMOUNT;
            }
            if up_pressed {
                scroll_offset[1] -= SCROLL_AMOUNT;
            }
        });

        //keys released -> stop scrolling
        if let Some(Button::Keyboard(key)) = event.release_args() {
            match key {
                Key::Up => up_pressed = false,
                Key::Down => down_pressed = false,
                Key::Left => left_pressed = false,
                Key::Right => right_pressed = false,
                _ => {}
            }
        }

        //keyboard-zoom handling
        if let Some(Button::Keyboard(key)) = event.press_args() {
            match key {
                Key::Equals => {
                    //zoom_in_pressed = true;
                    zoom_factor += ZOOM_AMOUNT;
                }
                Key::Plus => {
                    //zoom_in_pressed = true;
                    zoom_factor += ZOOM_AMOUNT;
                }
                Key::Minus => {
                    //zoom_out_pressed = true;
                    zoom_factor -= ZOOM_AMOUNT;
                    zoom_factor = zoom_factor.max(0.1); // Prevent zooming out too much
                }
                _ => {}
            }

        }

        window.draw_2d(&event, |context, graphics, device| {
            clear([0.0, 0.0, 0.0, 1.0], graphics);

            //draws a 3x3 grid with rectangles for the tile_type and circles for the content
           if should_draw_robot_view {
               draw_robot_view(
                   &convert_robot_view_to_color_matrix(&current_tuple_information.3),
                   &convert_robot_content_view_to_color_matrix(&current_tuple_information.3),
                   context,
                   graphics,
                   50.0,
               );
           }

            draw_optimized_grid(
                &current_tuple_information.0,
                context,
                graphics,
                (MAP_DIM, MAP_DIM),
                RECT_SIZE,
                scroll_offset,
                zoom_factor,
                //the following is used to draw the robot position
                coord_as_f64.0,
                coord_as_f64.1,
            );

            if should_draw_info_text {
                if let Some(ref mut glyphs) = glyphs {
                    let starting_text_x: u32 = 50;
                    let starting_text_y: u32 = 785;
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
                        starting_text_y,
                    );

                    //backpack
                    draw_text(
                        &context,
                        graphics,
                        glyphs,
                        [1.0; 4],
                        [starting_text_x, 30 + starting_text_y + 25 * 5],
                        current_tuple_information.4.as_str(),
                    );

                    draw_energy(
                        current_tuple_information.5,
                        &context,
                        graphics,
                        glyphs,
                    );

                    draw_score(
                        current_tuple_information.6,
                        &context,
                        graphics,
                        glyphs,
                    );

                    glyphs.factory.encoder.flush(device);
                }

            }
        });
    }
}

fn draw_score(
    score: f32,
    context: &piston_window::Context,
    graphics: &mut G2d,
    glyphs: &mut Glyphs,
) {

    draw_text(
        context,
        graphics,
        glyphs,
        [1.0; 4],
        [770, 20],
        "SCORE:",
    );
    draw_text(
        context,
        graphics,
        glyphs,
        [1.0; 4],
        [840, 20],
        score.floor().to_string().as_str(),
    );
}

fn draw_energy(
    energy: usize,
    context: &piston_window::Context,
    graphics: &mut G2d,
    glyphs: &mut Glyphs,
) {
    //ENERGY:
    draw_text(
        context,
        graphics,
        glyphs,
        [1.0; 4],
        [770, 55],
        "ENERGY:",
    );
    //actual energy value
    draw_text(
        context,
        graphics,
        glyphs,
        [1.0; 4],
        [845, 55],
        energy.to_string().as_str(),
    );
    //the rectangle
    draw_energy_level(
        energy,
        context,
        graphics,
        770.0,
        60.0
    )
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
