use crate::GifFrame::Frames as OtherFrames;
use crate::Util::{backpack_to_text, clear_png_files_in_directory, play_sound, DEFAULT_PNGS_PATH};
use rand::{thread_rng, Rng};
use robotics_lib::energy::Energy;
use robotics_lib::event::events::Event;
use robotics_lib::interface::Direction::{Down, Left, Right, Up};
use robotics_lib::interface::{destroy, go, robot_view, Direction};
use robotics_lib::interface::{put, robot_map};
use robotics_lib::runner::backpack::BackPack;
use robotics_lib::runner::{Robot, Runnable};
use robotics_lib::world::coordinates::Coordinate;
use robotics_lib::world::tile::Content::{Garbage, Rock};
use robotics_lib::world::tile::Tile;
use robotics_lib::world::World;
use std::sync::{Arc, Mutex};
use std::thread;

const PLAY_SOUNDS: bool = false;

pub trait Sentient: Runnable {
    fn act(&mut self, world: &mut World);
}

pub trait Visualizable {
    fn get_init_frames(&self) -> Arc<Mutex<OtherFrames>>;
    fn get_current_robot_map(&self) -> Arc<Mutex<Option<Vec<Vec<Option<Tile>>>>>>;
    fn get_current_robot_view(&self) -> Arc<Mutex<Vec<Vec<Option<Tile>>>>>;
    fn get_current_robot_backpack(&self) -> Arc<Mutex<String>>;
    fn get_score(&self) -> Arc<Mutex<f32>>;
    fn get_current_robot_coordinates(&self) -> Arc<Mutex<(usize, usize)>>;
}

pub struct ExampleRobot {
    robot: Robot,
    pub iterations: Arc<Mutex<usize>>,
    init_frames: Arc<Mutex<OtherFrames>>,
    current_robot_map: Arc<Mutex<Option<Vec<Vec<Option<Tile>>>>>>,
    current_robot_view: Arc<Mutex<Vec<Vec<Option<Tile>>>>>,
    current_robot_backpack: Arc<Mutex<String>>,
    score: Arc<Mutex<f32>>,
    current_robot_coordinates: Arc<Mutex<(usize, usize)>>,
}
impl Visualizable for ExampleRobot {
    fn get_init_frames(&self) -> Arc<Mutex<OtherFrames>> {
        self.init_frames.clone()
    }
    fn get_current_robot_map(&self) -> Arc<Mutex<Option<Vec<Vec<Option<Tile>>>>>> {
        self.current_robot_map.clone()
    }
    fn get_current_robot_view(&self) -> Arc<Mutex<Vec<Vec<Option<Tile>>>>> {
        self.current_robot_view.clone()
    }
    fn get_current_robot_backpack(&self) -> Arc<Mutex<String>> {
        self.current_robot_backpack.clone()
    }
    fn get_score(&self) -> Arc<Mutex<f32>> {
        self.score.clone()
    }
    fn get_current_robot_coordinates(&self) -> Arc<Mutex<(usize, usize)>> {
        self.current_robot_coordinates.clone()
    }
}
impl ExampleRobot {
    pub fn new(robot: Robot, iterations: Arc<Mutex<usize>>) -> Self {
        Self {
            robot,
            iterations,
            init_frames: Arc::new(Mutex::new(OtherFrames::new())),
            current_robot_map: Arc::new(Mutex::new(None)),
            current_robot_view: Arc::new(Mutex::new(vec![vec![None; 3]; 3])),
            current_robot_backpack: Arc::new(Mutex::new(String::new())),
            score: Arc::new(Mutex::new(0.0)),
            current_robot_coordinates: Arc::new(Mutex::new((0, 0))),
        }
    }
}
impl Sentient for ExampleRobot {
    fn act(&mut self, world: &mut World) {
        let index = *self.iterations.lock().unwrap(); //for debug

        //example of AI to degub visualizer
        //let environmental_conditions = look_at_sky(world);
        let direction;
        if index == 0 {
             direction=Direction::Right;
        } else {
             direction=Direction::Down;
        }
        /*
        if index <= 25 {
            direction = Direction::Right;
        } else if index > 25 && index <= 27 {
            direction = Direction::Down;
        } else if index > 27 && index <= 52 {
            direction = Direction::Left;
        } else if index > 52 && index <= 54 {
            direction = Direction::Down;
        } else if index > 54 && index <= 79 {
            direction = Direction::Right;
        } else if index > 79 && index <= 81 {
            direction = Direction::Down;
        } else if index > 81 && index <= 83 {
            direction = Direction::Left;
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
         */
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
        //debug
        println!("{}", index);
        *self.iterations.lock().unwrap() = index + 1;
    }
}

impl Runnable for ExampleRobot {
    fn process_tick(&mut self, world: &mut World) {
        self.act(world);
        //non modificare le seguenti righe
        //the following is something like *INITIAL_ROBOT_MAP.lock().unwrap() = robot_map(world);
        if let Err(e) = update_robot_map(self, world) {
            eprintln!("{}", e)
        }
        if let Err(e) = update_robot_view(self, world) {
            eprintln!("{}", e)
        }
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
                if let Err(e) = update_robot_coord(self, new_coord) {
                    eprintln!(
                        "couldnt lock CURRENT_ROBOT_COORDINATES in HandleEvent(Moved): {}",
                        e
                    )
                }

                println!("moved");

                //INIT_FRAMES.lock().unwrap().add_frame(&CURRENT_ROBOT_MAP.lock().unwrap());
                match self.get_init_frames().lock() {
                    Ok(mut init_frame_lock) => match &self.get_current_robot_map().lock() {
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
                if let Err(e) = update_robot_backpack(self, current_backpack) {
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
                if let Err(e) = update_robot_backpack(self, current_backpack) {
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

pub fn update_robot_view<'a, R>(robot: &'a R, world: &'a World) -> Result<(), String>
where
    R: Visualizable + Runnable,
{
    let new_view = robot_view(robot, world);
    match robot.get_current_robot_view().lock() {
        Ok(lock) => {
            let mut view_lock = lock;
            *view_lock = new_view;
            Ok(())
        }
        Err(_) => Err("Mutex was poisoned".to_string()),
    }
}

pub fn update_robot_map<'a, R>(robot: &'a R, world: &'a World) -> Result<(), String>
where
    R: Visualizable + Runnable,
{
    let new_map = robot_map(world); // Doing it before the assignment in order to reduce the lock() time
    match robot.get_current_robot_map().lock() {
        Ok(lock) => {
            let mut map_lock = lock;
            *map_lock = new_map;
            Ok(())
        }
        Err(_) => Err("Mutex was poisoned".to_string()),
    }
}

pub fn update_robot_coord<'a, R>(robot: &'a R, new_coord: &'a Coordinate) -> Result<(), String>
where
    R: Visualizable + Runnable,
{
    match robot.get_current_robot_coordinates().lock() {
        Ok(mut lock) => {
            *lock = (new_coord.get_row(), new_coord.get_col());
            Ok(())
        }
        Err(_) => Err("Mutex was poisoned".to_string()),
    }
}

pub fn update_robot_backpack<'a, R>(robot: &'a R, back_pack: &'a BackPack) -> Result<(), String>
where
    R: Visualizable + Runnable,
{
    match robot.get_current_robot_backpack().lock() {
        Ok(mut lock) => {
            *lock = backpack_to_text(back_pack);
            Ok(())
        }
        Err(_) => Err("Mutex was poisoned".to_string()),
    }
}
