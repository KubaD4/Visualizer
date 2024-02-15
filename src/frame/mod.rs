use image::{DynamicImage, ImageError, Rgba};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use crate::util::{gif_creator, id_to_path_string, match_color_to_type};
use robotics_lib::world::tile::{Tile};

const MAX_FAILURE_TOLERANCE: usize = 5;
const MAX_WAITING_CYCLES: usize = 10;
const N_OF_BYTES_PER_PIXEL: usize = 4;

///received_frames: it counts the amount of requests to process a Frame that have been received.
/// processed_frames: it counts the amount of frames that are successfully saved.
/// fails: Count the amount of frames that could not be saved due to errors
/// the type Arc<AtomicUsize> is due to the interaction of multiple threads with the parameters of the struct.
pub struct Frames {
    received_frames: usize,
    saved_frames: Arc<AtomicUsize>,
    fails: Arc<AtomicUsize>,
}

impl Frames {
    pub fn new() -> Self {

        Self {
            received_frames: 0,
            saved_frames: Arc::new(AtomicUsize::new(0)),
            fails: Arc::new(AtomicUsize::new(0)),
        }
    }
    pub fn add_frame(&mut self, robot_map: &Option<Vec<Vec<Option<Tile>>>>) {
        if robot_map.is_some() {
            println!(
                "VALUES: {} {}",
                self.received_frames,
                self.saved_frames.load(Ordering::SeqCst)
            );
            let frame = Frame::new_from_robot_map(robot_map, self.received_frames);
            self.received_frames += 1;

            let arc_processed_frames = self.saved_frames.clone();
            let arc_fails = self.fails.clone();

            thread::spawn(move || {
                match frame.save_frame() {
                    Ok(_) => {
                        println!("frame {} saved", frame.id);
                        //Ordering::Relaxed is enough for my aim -> Ordering::SeqCst has too much constrains that i dont need in this case.
                        //I just need to ensure that all the threads increments the counter but i dont care about the increasing order.
                        arc_processed_frames.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        match e {
                            //if the following errors occur, in my opinion it doesnt make sense to re-try savig
                            ImageError::Decoding(_) => {
                                arc_fails.fetch_add(1, Ordering::Relaxed);
                            }
                            ImageError::Encoding(_) => {
                                arc_fails.fetch_add(1, Ordering::Relaxed);
                            }
                            ImageError::Unsupported(_) => {
                                arc_fails.fetch_add(1, Ordering::Relaxed);
                            }
                            ImageError::IoError(_) => {
                                arc_fails.fetch_add(1, Ordering::Relaxed);
                            }
                            //in the other case it make sense to re-try (just 1 time but it can be decided)
                            _ => match frame.save_frame() {
                                Ok(_) => {
                                    println!("frame {} saved", frame.id);
                                    arc_processed_frames.fetch_add(1, Ordering::Relaxed);
                                }
                                Err(_) => {
                                    eprintln!("frame {} cant be saved", frame.id);
                                    arc_fails.fetch_sub(1, Ordering::Relaxed);
                                }
                            },
                        }
                    }
                }
            });
        }
    }

    pub fn convert_frames_to_gif(&self) -> Result<(), String> {
        let mut waiting_counter: usize = 0;
        let received_frames = self.received_frames;
        let mut atomic_processed_frames = self.saved_frames.clone().load(Ordering::SeqCst);
        let polling_interval = Duration::from_millis(100);
        let atomic_fails = self.fails.clone().load(Ordering::SeqCst);

        while received_frames != atomic_processed_frames {
            atomic_processed_frames = self.saved_frames.clone().load(Ordering::SeqCst);
            println!(
                "from_frames_to_gif method is waiting. Received={} saved={}",
                received_frames, atomic_processed_frames
            );
            sleep(polling_interval);
            waiting_counter += 1;
            if waiting_counter == MAX_WAITING_CYCLES {
                if atomic_fails >= MAX_FAILURE_TOLERANCE {
                    return Err(String::from("too many failure"));
                }
            }
        }
        gif_creator()
    }
}

/// Represents a single frame in the robot's journey, encapsulating the visual state as an image.
///
/// Attributes:
/// - `image`: The visual representation of the robot's map state.
/// - `id`: A unique identifier for the frame, used for saving and referencing.
pub struct Frame {
    image: DynamicImage,
    id: usize,
}

impl Frame {
    /// Creates a new `Frame` instance from a given robot map state.
    ///
    /// Arguments:
    /// - `robot_map`: The robot's current discovered map to be visualized.
    /// - `id`: The unique identifier for the frame.
    pub fn new_from_robot_map(robot_map: &Option<Vec<Vec<Option<Tile>>>>, id: usize) -> Self {
        Self {
            image: Self::robot_map_to_dynamic_image(robot_map),
            id,
        }
    }

    /// Creates a `DynamicImage` from a given robot map
    fn robot_map_to_dynamic_image(map: &Option<Vec<Vec<Option<Tile>>>>) -> DynamicImage {
        let dim = map.clone().unwrap_or(vec![]).len();

        let mut pixel_data: Vec<u8> = Vec::with_capacity(dim * dim * N_OF_BYTES_PER_PIXEL);
        //total number of pixels in the image, 4 bytes per pixel (R,G,B and A=transparency)

        let mut color_rgba: (u8, u8, u8, u8);
        for row in map {
            for tile in row {
                for _tile in tile {
                    match &_tile.clone() {
                        Some(_) => {
                            color_rgba = match_color_to_type(&_tile.clone().unwrap().tile_type)
                        }
                        None => color_rgba = (0, 0, 0, 0), //transparent
                    }
                    pixel_data.push(color_rgba.0);
                    pixel_data.push(color_rgba.1);
                    pixel_data.push(color_rgba.2);
                    pixel_data.push(color_rgba.3);
                }
            }
        }

        let image_buffer =
            image::ImageBuffer::<Rgba<u8>, _>::from_vec(dim as u32, dim as u32, pixel_data)
                .expect("Failed to create ImageBuffer");
        DynamicImage::ImageRgba8(image_buffer)
    }

    /// Saves the frame to disk.
    ///
    /// This method attempts to save the frame's image to a predetermined location, using the frame's
    /// ID to generate a unique file name.
    ///
    /// Returns:
    /// - `Ok(())` if the image is successfully saved.
    /// - `Err(ImageError)` containing details of any error encountered during saving.
    pub fn save_frame(&self) -> Result<(), ImageError> {
        match self.image.save(id_to_path_string(self.id)) {
            Ok(_) => {
                println!("Frame n.{} saved", self.id);
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to save PNG image: {}", e);
                Err(e)
            }
        }
    }
}

