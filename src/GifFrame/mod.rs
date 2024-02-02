use std::sync::{Arc};
use std::time::Duration;
use image::{DynamicImage, ImageError, Rgba};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::thread::sleep;

use crate::Util::{gif_creator, id_to_path_string, match_color_to_type};
use robotics_lib::world::tile::{Tile, TileType};

const MAX_FAILURE_TOLLERANCE: usize = 5;
const MAX_WAITING_CYCLES: usize = 10;

///received_frames: to count how much time is requested to save a frame
/// processed_frames: it counts the amount of frames that are successfully saved.
/// fails: it counts the amount of frames that cant be saved due to errors (handled in the following methods)
/// the type Arc<AtomicUsize> is due to the interaction of multiple threads with the parameters of the struct.
pub struct Frames {
    received_frames: usize,
    saved_frames: Arc<AtomicUsize>,
    fails: Arc<AtomicUsize>
    //sender: Option<mpsc::Sender<Frame>>,
}

impl Frames {
    pub fn new() -> Self {

        //let (sender, receiver) = mpsc::channel::<Frame>();
        // Spawn a worker thread that listens for frames to save -> it was to slow
        /*
        thread::spawn(move || {
            for frame in receiver {
                // Replace this with your actual logic
                frame.save_frame();
            }
        });
         */

        Self {
            received_frames: 0,
            saved_frames: Arc::new(AtomicUsize::new(0)),
            fails: Arc::new(AtomicUsize::new(0))
            //sender: Some(sender)
        }
    }
    pub fn add_frame(&mut self, robot_map: &Option<Vec<Vec<Option<Tile>>>>) {
        if robot_map.is_some() {
            println!("VALUES: {} {}", self.received_frames, self.saved_frames.load(Ordering::SeqCst));
            let frame = Frame::new_from_robot_map(robot_map, self.received_frames);
            self.received_frames += 1;

            let arc_processed_frames = self.saved_frames.clone();
            let arc_fails= self.fails.clone();

            let save_handle = thread::spawn(move || {
                match frame.save_frame() {
                    Ok(_) => {
                        println!("frame {} saved", frame.id);
                        //Ordering::Relaxed is enough for my aim -> Ordering::SeqCst has too much constrains that i dont need in this case.
                        //I just need to ensure that all the threads increments the counter but i dont care about the increasing order.
                        arc_processed_frames.fetch_add(1, Ordering::Relaxed);
                    },
                    Err(e) => {
                        match e {
                            //if the following errors occur, in my opinion it doesnt make sense to re-try savig
                            ImageError::Decoding(_) => {
                                arc_fails.fetch_add(1,Ordering::Relaxed);
                            }
                            ImageError::Encoding(_) => {
                                arc_fails.fetch_add(1,Ordering::Relaxed);
                            }
                            ImageError::Unsupported(_) => {
                                arc_fails.fetch_add(1,Ordering::Relaxed);

                            }
                            ImageError::IoError(_) => {
                                arc_fails.fetch_add(1,Ordering::Relaxed);

                            }
                            //in the other case it make sense to re-try (just 1 time but it can be decided)
                            _ => {
                                match frame.save_frame() {
                                    Ok(_) => {
                                        println!("frame {} saved", frame.id);
                                        arc_processed_frames.fetch_add(1, Ordering::Relaxed);
                                    }
                                    Err(_) => {
                                        eprintln!("frame {} cant be saved", frame.id);
                                        arc_fails.fetch_sub(1,Ordering::Relaxed);
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }
    }

    pub fn from_frames_to_gif(&self) -> Result<(), String> {
        let mut waiting_counter: usize = 0;
        let received_frames = self.received_frames;
        let mut atomic_processed_frames = self.saved_frames.clone().load(Ordering::SeqCst);
        let polling_interval = Duration::from_millis(100);
        let atomic_fails = self.fails.clone().load(Ordering::SeqCst);

        while received_frames!=atomic_processed_frames {
            atomic_processed_frames = self.saved_frames.clone().load(Ordering::SeqCst);
            println!("from_frames_to_gif method is waiting. Received={} saved={}",received_frames,atomic_processed_frames);
            sleep(polling_interval);
            waiting_counter += 1;
            if waiting_counter == MAX_WAITING_CYCLES {
                if atomic_fails >= MAX_FAILURE_TOLLERANCE {
                    return Err(String::from("too many failure"))
                }
            }
        }
        gif_creator()
    }

    /*
    pub fn save_frames(mut self) {
        let handles: Vec<_> = self.frames.into_iter().map(|frame| {
            thread::spawn(move || {
                // Replace this with your actual logic
                frame.save_frame()
            })
        }).collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
    }
     */
}

const N_OF_BYTES_PER_PIXEL: usize = 4;
pub struct Frame {
    image: DynamicImage,
    id: usize,
}

impl Frame {
    pub fn new_from_robot_map(robot_map: &Option<Vec<Vec<Option<Tile>>>>, id: usize) -> Self {
        Self {
            image: Self::robotMap_to_dynamicImage(robot_map),
            id
        }
    }

    fn map_to_dynamicImage(map: &Vec<Vec<TileType>>) -> DynamicImage {
        let width = map[0].len();
        let height = map.len();

        let mut pixel_data: Vec<u8> = Vec::with_capacity(width * height * N_OF_BYTES_PER_PIXEL);
        //total number of pixels in the image, 4 bytes per pixel (R,G,B and A=transparency)

        for row in map {
            for tile in row {
                let color_rgba = match_color_to_type(tile);
                pixel_data.push(color_rgba.0);
                pixel_data.push(color_rgba.1);
                pixel_data.push(color_rgba.2);
                pixel_data.push(color_rgba.3);
            }
        }

        let image_buffer =
            image::ImageBuffer::<Rgba<u8>, _>::from_vec(width as u32, height as u32, pixel_data)
                .expect("Failed to create ImageBuffer");
        DynamicImage::ImageRgba8(image_buffer)
    }

    fn robotMap_to_dynamicImage(map: &Option<Vec<Vec<Option<Tile>>>>) -> DynamicImage {
        let dim = map.clone().unwrap_or(vec![]).len();

        let mut pixel_data: Vec<u8> = Vec::with_capacity(dim * dim * N_OF_BYTES_PER_PIXEL);
        //total number of pixels in the image, 4 bytes per pixel (R,G,B and A=transparency)

        let mut color_rgba:(u8, u8, u8, u8);
        for row in map {
            for tile in row {
                for _tile in tile {
                    match &_tile.clone() {
                        Some(tile) => color_rgba = match_color_to_type(&_tile.clone().unwrap().tile_type),
                        None => color_rgba = (0,0,0,0) //transparent
                    }
                    pixel_data.push(color_rgba.0);
                    pixel_data.push(color_rgba.1);
                    pixel_data.push(color_rgba.2);
                    pixel_data.push(color_rgba.3);
                }
            }
        }

        let image_buffer =
            image::ImageBuffer::<Rgba<u8>, _>::from_vec(dim.clone() as u32, dim.clone() as u32, pixel_data)
                .expect("Failed to create ImageBuffer");
        DynamicImage::ImageRgba8(image_buffer)
    }

    pub fn save_frame(&self) -> Result<(),ImageError> {
        match self.image.save(id_to_path_string(self.id)) {
            Ok(_) => {
                println!("Frame n.{} saved", self.id);
                Ok(())
            },
            Err(e) => {
                eprintln!("{}",format!("Failed to save PNG image: {}", e));
                Err(e)
            }
        }
    }
}