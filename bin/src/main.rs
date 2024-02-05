use piston_window::{
    clear, Button, G2d, Glyphs, Key, MouseScrollEvent, OpenGL, PistonWindow, PressEvent,
    ReleaseEvent, Size, WindowSettings,
};
use rand::{thread_rng, Rng};
use robotics_lib::interface::Tools;
use robotics_lib::runner::{Robot, Runner};
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
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use worldgen_unwrap::*;
use Visualizer::robot::{ExampleRobot, Visualizable};
use Visualizer::Grid::*;
use Visualizer::Util::{
    convert_content_to_color_matrix, convert_to_color_matrix, Infos, DEFAULT_PNGS_PATH,
};

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

    //IMPLEMENTATION OF THE WORLDGENERATOR AND PROCESS TICK
    thread::spawn(move || {
        /*
        // WorldGenerator molto stupido di prova
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
        let mut generator = WorldGenerator::init(MAP_DIM);
        */

        // WorldGenerator del nostro gruppo
        let mut generator = worldgen_unwrap::public::WorldgeneratorUnwrap::init(false, None);

        struct Tool;
        impl Tools for Tool {}
        let i = r.iterations.clone();
        let mut run = Runner::new(Box::new(r), &mut generator);
        loop {
            match run {
                Ok(ref mut runner) => {
                    let _ = runner.game_tick();
                    if *i.lock().unwrap() > 500 {
                        match init_frames.lock() {
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

            let matrix_to_be_sent = initial_color_matrix.lock().unwrap().clone();
            let matrix_content_to_be_sent = initial_content_color_matrix.lock().unwrap().clone();
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
