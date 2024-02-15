use robotics_lib::runner::backpack::BackPack;
use rodio::source::Source;
use rodio::{Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, io};

use robotics_lib::world::tile::{Content, Tile, TileType};

///# Types
/// * `Infos': The type used to pass information from the robot's channel to the UI
///     * Tile Type converted to color matrix
///     * Content converted to color matrix
///     * Coordinates of the robot
///     * Robot View
///     * Backpack
///     * Energy
///     * Score
pub type Infos = (
    Vec<Vec<[f32; 4]>>,
    Vec<Vec<[f32; 4]>>,
    (usize, usize),
    Vec<Vec<Option<Tile>>>,
    String,
    usize,
    f32
);

///# Constants
///* `DEFAULT_PNGS_PATH`: The default directory path where PNG files are stored.
///* `DEFAULT_SOUNDS_PATH`: The default directory path where sound files are stored.
pub const DEFAULT_PNGS_PATH: &str = "../pngs";
pub const DEFAULT_SOUNDS_PATH: &str = "../sounds";

/// Converts a numerical ID to a file path string.
///
/// # Arguments
/// * `id` - The frame ID to convert.
///
/// # Returns
/// A file path string with leading zeroes based on the ID's value.
pub(crate) fn id_to_path_string(id: usize) -> String {
    match id {
        0..=9 => format!("{}/0000{}.png", DEFAULT_PNGS_PATH, id),
        10..=99 => format!("{}/000{}.png", DEFAULT_PNGS_PATH, id),
        100..=999 => format!("{}/00{}.png", DEFAULT_PNGS_PATH, id),
        1000..=9999 => format!("{}/0{}.png", DEFAULT_PNGS_PATH, id),
        10000..=99999 => format!("{}/{}.png", DEFAULT_PNGS_PATH, id),
        _ => format!("99999/{}.png", DEFAULT_PNGS_PATH),
    }
}

/// Generates a GIF animation from sequentially numbered PNG files found in the
/// default PNG directory. Utilizes FFmpeg for the conversion process.
///
/// # Returns
/// A result indicating success or failure of the GIF creation process.
pub fn gif_creator() -> Result<(), String> {
    // Specify the directory containing the PNG files
    let directory = DEFAULT_PNGS_PATH;
    // Specify the output video file name
    let output_video = "output_video.mp4";

    return match Command::new("/opt/homebrew/bin/ffmpeg")
        .args(&[
            "-y", // Overwrite output files without asking
            "-framerate",
            "60", // Set frame rate
            "-f",
            "image2", // Force format
            "-i",
            &format!("{}/%05d.png", directory), // Input file pattern
            "-vcodec",
            "libx264", // Video codec
            "-crf",
            "17", // Constant Rate Factor
            "-preset",
            "medium",     // Encoder preset
            output_video, // Output file
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(ffmpeg_cmd) => match ffmpeg_cmd.wait_with_output() {
            Ok(output) => {
                println!("ffmpeg finished with status: {}", output.status);
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    };
}

/// Clears all PNG files from the given directory path.
///
/// # Arguments
/// * `dir_path` - The path of the directory from which to remove PNG files.
///
/// # Returns
/// A result indicating the success or failure of the file removal operation.
pub fn clear_png_files_in_directory(dir_path: &str) -> io::Result<()> {
    let path = Path::new(dir_path);

    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Directory not found",
        ));
    }

    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Check if the entry is a file and has a .png extension
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("png") {
            fs::remove_file(path)?; // Delete the file
        }
    }
    Ok(())
}

/// Converts a `TileType` or `Content` to its corresponding RGBA color representation.
///
/// # Arguments
/// * `tile_type` or `content` - The tile type or content to convert.
///
/// # Returns
/// An RGBA color tuple.
pub(crate) fn match_color_to_type(tile_type: &TileType) -> (u8, u8, u8, u8) {
    match tile_type {
        TileType::Grass => (0, 255, 0, 255),
        TileType::Street => (0, 0, 0, 255),
        TileType::ShallowWater => (0, 0, 255, 255),
        TileType::DeepWater => (0, 0, 128, 255),
        TileType::Sand => (255, 255, 0, 255),
        TileType::Hill => (255, 128, 0, 255),
        TileType::Mountain => (128, 128, 128, 255),
        TileType::Wall => (255, 128, 0, 255),
        TileType::Teleport(_) => (255, 0, 255, 255),
        TileType::Lava => (255, 0, 0, 255),
        TileType::Snow => (255, 255, 255, 255),
    }
}
pub(crate) fn match_color_to_content(content: &Content) -> (u8, u8, u8, u8) {
    match content {
        // match on all variants giving a rgba color for each
        Content::Rock(_) => (112, 128, 144, 255),
        Content::Tree(_) => (0, 100, 0, 255),
        Content::Garbage(_) => (0, 0, 0, 255),
        Content::Fire => (255, 0, 0, 255),
        Content::Coin(_) => (255, 215, 0, 255),
        Content::Bin(_) => (70, 130, 180, 255),
        Content::Crate(_) => (255, 128, 0, 255),
        Content::Bank(_) => (128, 128, 128, 255),
        Content::Water(_) => (173, 216, 230, 255),
        Content::Market(_) => (255, 0, 255, 255),
        Content::Fish(_) => (64, 224, 208, 255),
        Content::Building => (204, 85, 0, 255),
        Content::Bush(_) => (50, 205, 50, 255),
        Content::JollyBlock(_) => (255, 192, 203, 255),
        Content::Scarecrow => (160, 82, 45, 255),
        Content::None => (0, 0, 0, 0),
    }
}

/// Converts a `TileType` or `Content` to a Piston engine compatible color array.
///
/// # Arguments
/// * `tile_type` or `content` - The tile type or content to convert.
///
/// # Returns
/// A Piston-compatible color array.
pub fn match_color_to_type_piston(tile_type: &TileType) -> [f32; 4] {
    let almost_result = match_color_to_type(tile_type);

    [
        (almost_result.0 as f32)/255.0,
        (almost_result.1 as f32)/255.0,
        (almost_result.2 as f32)/255.0,
        (almost_result.3 as f32)/255.0,
    ]
}
pub fn match_content_color_to_type_piston(tile_contet: &Content) -> [f32; 4] {
    let almost_result = match_color_to_content(tile_contet);
    [
        (almost_result.0 as f32)/255.0,
        (almost_result.1 as f32)/255.0,
        (almost_result.2 as f32)/255.0,
        (almost_result.3 as f32)/255.0,
    ]
}

/// Plays a sound file from the default sounds directory with an optional
/// amplification value.
///
/// # Arguments
/// * `path` - The path to the sound file, relative to the default sounds directory.
/// * `amplify_value` - The amplification factor for the sound playback.
///
/// # Returns
/// A result indicating success or failure of the sound playback operation.
pub fn play_sound(path: &str, amplify_value: f32) -> Result<(), String> {
    let final_path = format!("{}{}", DEFAULT_SOUNDS_PATH, path);
    // Try to get the default output stream
    let (_stream, stream_handle) =
        OutputStream::try_default().map_err(|e| format!("Error obtaining output stream: {}", e))?;

    // Try to open the file
    let file = BufReader::new(
        File::open(final_path).map_err(|e| format!("Error opening file '{}': {}", path, e))?,
    );

    // Try to decode the sound file
    let source = Decoder::new(file)
        .map_err(|e| format!("Error decoding '{}': {}", path, e))?
        .amplify(amplify_value);

    // Try to play the sound
    stream_handle
        .play_raw(source.convert_samples())
        .map_err(|e| format!("Error playing sound: {}", e))?;

    // Sleep for a while to allow the sound to play
    std::thread::sleep(Duration::from_secs_f64(0.5));
    Ok(())
}

/// Converts the contents of a `BackPack` object into a formatted string
/// representation, listing each item and its quantity.
///
/// # Arguments
/// * `backpack` - The `BackPack` object to convert.
///
/// # Returns
/// A string representation of the backpack's contents.
pub fn backpack_to_text(backpack: &BackPack) -> String {
    if backpack.get_size() > 0 && !backpack.get_contents().is_empty() {
        let mut result = format!("Backpack (Size: {}):  ", backpack.get_size());
        for (content, size) in backpack.get_contents() {
            if size > &(0usize) {
                result += &format!("{}: {}  ", content, size);
            }
        }
        result
    } else {
        "Empty backpack".to_string()
    }
}

pub fn convert_to_color_matrix(
    tile_matrix: &Option<Vec<Vec<Option<Tile>>>>,
    color_matrix: &Arc<Mutex<Vec<Vec<[f32; 4]>>>>,
) {
    if let Some(tile_rows) = tile_matrix {
        let mut color_matrix_guard = color_matrix.lock().unwrap();

        for (i, row) in tile_rows.iter().enumerate() {
            for (j, tile_option) in row.iter().enumerate() {
                let color = match tile_option {
                    Some(tile) => match_color_to_type_piston(&tile.tile_type),
                    None => [0.0, 0.0, 0.0, 0.0], // Default color for None
                };
                color_matrix_guard[j][i] = color;
            }
        }
    }
}

/// Converts a matrix of `Tile`s, `Content`, or robot views into a corresponding
/// color matrix for visualization purposes, using predefined color mappings.
///
/// # Arguments
/// * Various, depending on the function.
///
/// # Returns
/// A color matrix for use in visual representations.
pub fn convert_robot_view_to_color_matrix(view: &Vec<Vec<Option<Tile>>>) -> Vec<Vec<[f32; 4]>> {
    let mut result = vec![vec![[0.0, 0.0, 0.0, 0.0]; 3]; 3];


    for (i, row) in view.iter().enumerate() {
        for (j, tile_option) in row.iter().enumerate() {
            let color = match tile_option {
                Some(tile) => match_color_to_type_piston(&tile.tile_type),
                None => [105.0/255.0 , 105.0/255.0 , 105.0/255.0 , 1.0],
            };
            result[j][i] = color;
        }
    }

    result
}
pub fn convert_robot_content_view_to_color_matrix(view: &Vec<Vec<Option<Tile>>>) -> Vec<Vec<[f32; 4]>> {
    let mut result = vec![vec![[0.0, 0.0, 0.0, 1.0]; 3]; 3];


    for (i, row) in view.iter().enumerate() {
        for (j, tile_option) in row.iter().enumerate() {
            let color = match tile_option {
                Some(tile) => match_content_color_to_type_piston(&tile.content),
                None => [0.0, 0.0, 0.0, 0.0],
            };
            result[j][i] = color;
        }
    }

    result
}
pub fn convert_content_to_color_matrix(
    tile_matrix: &Option<Vec<Vec<Option<Tile>>>>,
    color_matrix: &Arc<Mutex<Vec<Vec<[f32; 4]>>>>,
) {
    if let Some(tile_rows) = tile_matrix {
        let mut color_matrix_guard = color_matrix.lock().unwrap();

        for (i, row) in tile_rows.iter().enumerate() {
            for (j, tile_option) in row.iter().enumerate() {
                let color = match tile_option {
                    Some(tile) => match_content_color_to_type_piston(&tile.content),
                    None => [0.0, 0.0, 0.0, 0.0], // Default color for None
                };
                color_matrix_guard[j][i] = color;
            }
        }
    }
}

/// Safely updates a shared resource, represented by a `Mutex`, with a new value.
///
/// # Type Parameters
/// * `T` - The type of the resource to update.
///
/// # Arguments
/// * `resource` - A reference to the `Mutex` guarding the resource.
/// * `new_value` - The new value to assign to the resource.
///
/// # Returns
/// A result indicating success or the failure reason.
pub fn update_resource<T>(resource: &Mutex<T>, new_value: T) -> Result<(), String> {
    match resource.lock() {
        Ok(mut lock) => {
            *lock = new_value;
            Ok(())
        },
        Err(_) => Err("Mutex was poisoned".to_string()),
    }
}
