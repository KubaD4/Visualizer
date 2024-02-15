extern crate piston_window;

use piston_window::*;
use piston_window::{Context, G2d, rectangle};
use piston_window::types::{Color};

type ColorMatrix = Vec<Vec<[f32; 4]>>;

pub const MAP_SIZE: usize = 700;
pub const GRID_SIZE: (usize, usize) = (MAP_SIZE, MAP_SIZE);
pub const RECT_SIZE: f64 = 750.0 / MAP_SIZE as f64;
pub const WINDOW_SIZE: (usize, usize) = (
    950,
    950,
);

pub const ZOOM_AMOUNT: f64 = 0.35;
pub const SCROLL_AMOUNT: f64 = 5.0;

pub const ROBOT_COLOR: [f32; 4] = [191.0 / 255.0, 139.0 / 255.0, 255.0 / 255.0, 1.0];

/// Draws a grid based on a given color matrix, with support for zoom and scroll.
///
/// This function iterates over a matrix of colors to draw a grid of rectangles. It optimizes
/// rendering by merging contiguous cells of the same color into single rectangles.
/// Then, the function displays the robot's position on the produced grid
///
/// # Arguments
/// * `matrix` - A reference to the primary color matrix for drawing rectangles.
/// * `context` - The Piston window context for drawing.
/// * `graphics` - The graphics backend for rendering shapes.
/// * `grid_size` - The dimensions of the grid (in cells).
/// * `rect_size` - The size of each cell in the grid.
/// * `scroll_offset` - The current scroll offset for the view.
/// * `zoom_factor` - The current zoom level for the view.
/// * `coord_x` - The x-coordinate of the robot's position.
/// * `coord_y` - The y-coordinate of the robot's position.
pub fn draw_optimized_grid(
    matrix: &ColorMatrix,
    context: Context,
    graphics: &mut G2d,
    grid_size: (usize, usize),
    rect_size: f64,
    scroll_offset: [f64; 2],
    zoom_factor: f64,
    coord_x: f64,
    coord_y: f64,
) {
    // Calculate visible area considering zoom and scroll
    let visible_start_col = ((scroll_offset[0] / zoom_factor) / rect_size).max(0.0) as usize;
    let visible_start_row = ((scroll_offset[1] / zoom_factor) / rect_size).max(0.0) as usize;
    let visible_end_col = (((scroll_offset[0] + WINDOW_SIZE.0 as f64) / zoom_factor) / rect_size)
        .min(grid_size.0 as f64) as usize;
    let visible_end_row = (((scroll_offset[1] + WINDOW_SIZE.1 as f64) / zoom_factor) / rect_size)
        .min(grid_size.1 as f64) as usize;

    let transform = context
        .transform
        .trans(-scroll_offset[0], -scroll_offset[1])
        .zoom(zoom_factor);

    for j in visible_start_row..visible_end_row {
        let mut i = visible_start_col;
        while i < visible_end_col {
            let color = matrix[i][j];
            let mut end_col = i + 1;
            while end_col < visible_end_col && matrix[end_col][j] == color {
                end_col += 1;
            }

            let rect_x = i as f64 * rect_size * zoom_factor - scroll_offset[0];
            let rect_y = j as f64 * rect_size * zoom_factor - scroll_offset[1];
            let rect_width = (end_col - i) as f64 * rect_size * zoom_factor;

            rectangle(
                color,
                [rect_x, rect_y, rect_width, rect_size * zoom_factor],
                transform,
                graphics,
            );

            //robot's position
            let robot_x = coord_x * rect_size * zoom_factor - scroll_offset[0];
            let robot_y = coord_y * rect_size * zoom_factor - scroll_offset[1];
            let robot_rect_width = rect_size * zoom_factor;
            rectangle(
                ROBOT_COLOR,
                [robot_x, robot_y, robot_rect_width, rect_size * zoom_factor],
                transform,
                graphics,
            );

            i = end_col;
        }
    }

    let white = [1.0, 1.0, 1.0, 1.0]; // RGBA color for white
    // Draw a white rectangle to the right of the last column
    let right_rect_x = grid_size.0 as f64 * rect_size * zoom_factor - scroll_offset[0];
    let right_rect_y = -scroll_offset[1]; // Start from the top
    let right_rect_height = grid_size.1 as f64 * rect_size * zoom_factor; // Height of the entire grid
    rectangle(
        white,
        [
            right_rect_x,
            right_rect_y,
            10.0,//right_rect_width,
            right_rect_height,
        ],
        transform,
        graphics,
    );

    // Draw a white rectangle below the last row
    let bottom_rect_x = -scroll_offset[0]; // Start from the left
    let bottom_rect_y = grid_size.1 as f64 * rect_size * zoom_factor - scroll_offset[1];
    let bottom_rect_width = grid_size.0 as f64 * rect_size * zoom_factor; // Width of the entire grid
    rectangle(
        white,
        [
            bottom_rect_x,
            bottom_rect_y,
            bottom_rect_width,
            10.0,//bottom_rect_height,
        ],
        transform,
        graphics,
    );
}

/// Draws a 3x3 grid representing the robot's immediate surroundings.
///
/// This function visualizes the robot's local view by drawing a 3x3 grid of
/// rectangles(representing the TileType) and circles(representing the Content),
/// where each cell's color is determined by the corresponding entry in the provided color matrices.
///
/// # Arguments
/// * `rect_matrix` - Color matrix for the rectangles of the robot's view.
/// * `circle_matrix` - Color matrix for the circles within the robot's view.
/// * `context` - The Piston window context.
/// * `graphics` - The graphics backend.
/// * `rect_size` - The size of each rectangle and circle in the grid.
pub fn draw_robot_view(
    rect_matrix: &Vec<Vec<[f32; 4]>>,
    circle_matrix: &Vec<Vec<[f32; 4]>>,
    context: Context,
    graphics: &mut G2d,
    rect_size: f64,
) {
    let grid_start_x = 500.0;
    let grid_start_y = 770.0;

    // Iterate over the 3x3 matrix for rectangles
    for (i, row) in rect_matrix.iter().enumerate() {
        for (j, _) in row.iter().enumerate() {
            // Calculate the position for each rectangle
            let x = grid_start_x + (j as f64 * rect_size);
            let y = grid_start_y + (i as f64 * rect_size);

            // Draw the rectangle
            if let Some(&rect_color) = rect_matrix.get(j).and_then(|r| r.get(i)) {
                rectangle(
                    rect_color,
                    [x, y, rect_size, rect_size],
                    context.transform,
                    graphics,
                );
            }


            let circle_radius = rect_size / 4.0;
            let circle_x = x + rect_size / 2.0 - circle_radius;
            let circle_y = y + rect_size / 2.0 - circle_radius;
            if let Some(&circle_color) = circle_matrix.get(j).and_then(|r| r.get(i)) {
                ellipse(
                    circle_color,
                    [circle_x, circle_y, circle_radius * 2.0, circle_radius * 2.0],
                    context.transform,
                    graphics,
                );
            }
        }
    }
}

/// Draws a rectangle representing the robot's current energy level.
///
/// The color and length of the rectangle vary based on the robot's current energy, providing
/// a visual indicator of its status.
///
/// # Arguments
/// * `energy_level` - The current energy level of the robot.
/// * `context` - The Piston window context.
/// * `graphics` - The graphics backend.
/// * `start_x` - The starting x-coordinate for the energy level rectangle.
/// * `start_y` - The starting y-coordinate for the energy level rectangle.
pub fn draw_energy_level(
    energy_level: usize,
    context: &Context,
    graphics: &mut G2d,
    start_x: f64, // Starting X position for the rectangle
    start_y: f64, // Starting Y position for the rectangle
) {
    let length = (energy_level as f64 / 1000.0) * 100.0; //pixels -> if the energy is at the maximum value(1000) the rect will be 100pixel long

    let color:[f32;4] = match energy_level {
        801..=1000 => { // 80% to 100%
            // Full green to lighter green
            let factor = (energy_level as f64 - 800.0) / 200.0;
            [0.0, ((1.0 - factor) * 139.0/255.0 + factor) as f32, 0.0, 1.0]
        },
        601..=800 => { // 60% to 80%
            // Lighter green to orange
            let factor = (energy_level as f64 - 600.0) / 200.0;
            [((factor) * 255.0/255.0) as f32, ((1.0 - factor) * 255.0/255.0) as f32, 0.0, 1.0]
        },
        401..=600 => { // 40% to 60%
            // Orange to red
            [255.0/255.0, ((1.0 - ((energy_level as f64 - 400.0) / 200.0)) as f32) * 165.0/255.0, 0.0, 1.0]
        },
        201..=400 => { // 20% to 40%
            // Red to light red
            let factor = (energy_level as f64 - 200.0) / 200.0;
            [1.0, (factor * 69.0/255.0) as f32, (factor * 69.0/255.0) as f32, 1.0]
        },
        _ => { // Below 20%
            // Light red
            [1.0, 0.69, 0.69, 1.0]
        },
    };

    // Draw the energy level rectangle
    rectangle(
         color, // Color based on energy level
        [start_x, start_y, length, 10.0], // x, y, width, height
        context.transform,
        graphics,
    );
}

/// Draws textual information at a specified position on the screen.
///
/// # Arguments
/// * `ctx` - The Piston window context.
/// * `graphics` - The graphics backend.
/// * `glyphs` - The font glyphs for
/// * `color` - The color of the text
/// * `pos` - The position of the text
/// * `text` - The actual text to draw
pub fn draw_text(
    ctx: &Context,
    graphics: &mut G2d,
    glyphs: &mut Glyphs,
    color: Color,
    pos: [u32; 2],
    text: &str,
) {
    Text::new_color(color, 20)
        .draw(
            text,
            glyphs,
            &ctx.draw_state,
            ctx.transform.trans(pos[0] as f64, pos[1] as f64),
            graphics,
        )
        .unwrap();
}
