extern crate piston_window;

use piston_window::types::Color;
use piston_window::*;
use piston_window::{clear, rectangle, Context, G2d};

type ColorMatrix = Vec<Vec<[f32; 4]>>;
/*
pub const MAP_SIZE: usize = 350;
pub const GRID_SIZE: (usize, usize) = (MAP_SIZE, MAP_SIZE);
pub const WINDOW_SIZE: (usize, usize) = (MAP_SIZE+50, MAP_SIZE+50);
pub const RECT_SIZE: f64 = ((WINDOW_SIZE.0-50) / GRID_SIZE.1) as f64;
pub const ZOOM_AMOUNT: f64 = 0.35;
pub const SCROLL_AMOUNT: f64 = 5.0;
 */

/*
pub const MAP_SIZE: usize = 750;
pub const GRID_SIZE: (usize, usize) = (MAP_SIZE, MAP_SIZE);
pub const WINDOW_SIZE: (usize, usize) = (MAP_SIZE * 2, MAP_SIZE * 2);
//pub const RECT_SIZE: f64 = ((WINDOW_SIZE.0-100) / GRID_SIZE.1) as f64;
pub const RECT_SIZE: f64 = (WINDOW_SIZE.0 - 100) as f64 / MAP_SIZE as f64;
pub const ZOOM_AMOUNT: f64 = 0.35;
pub const SCROLL_AMOUNT: f64 = 5.0;
 */

pub const MAP_SIZE: usize = 750;
pub const GRID_SIZE: (usize, usize) = (MAP_SIZE, MAP_SIZE);
// Assuming RECT_SIZE is defined to fit the grid within the window minus padding
// And since we want the window 200 pixels greater than the grid, we adjust the calculation accordingly
pub const RECT_SIZE: f64 = (MAP_SIZE as f64 - 200.0) / MAP_SIZE as f64;
pub const WINDOW_SIZE: (usize, usize) = (
    (MAP_SIZE as f64 * RECT_SIZE + 200.0) as usize,
    (MAP_SIZE as f64 * RECT_SIZE + 200.0) as usize,
);
pub const ZOOM_AMOUNT: f64 = 0.35;
pub const SCROLL_AMOUNT: f64 = 5.0;

pub fn draw_optimized_grid(
    matrix: &ColorMatrix,
    context: Context,
    graphics: &mut G2d,
    grid_size: (usize, usize),
    rect_size: f64,
    scroll_offset: [f64; 2],
    zoom_factor: f64,
) {
    clear([0.0; 4], graphics); // Clear the screen

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
            i = end_col;
        }
    }

    let white = [1.0, 1.0, 1.0, 1.0]; // RGBA color for white

    // Draw a white rectangle to the right of the last column
    let right_rect_x = grid_size.0 as f64 * rect_size * zoom_factor - scroll_offset[0];
    let right_rect_y = -scroll_offset[1]; // Start from the top
    let right_rect_width = rect_size * zoom_factor; // Width of one rectangle
    let right_rect_height = grid_size.1 as f64 * rect_size * zoom_factor; // Height of the entire grid

    rectangle(
        white,
        [
            right_rect_x,
            right_rect_y,
            right_rect_width,
            right_rect_height,
        ],
        transform,
        graphics,
    );

    // Draw a white rectangle below the last row
    let bottom_rect_x = -scroll_offset[0]; // Start from the left
    let bottom_rect_y = grid_size.1 as f64 * rect_size * zoom_factor - scroll_offset[1];
    let bottom_rect_width = grid_size.0 as f64 * rect_size * zoom_factor; // Width of the entire grid
    let bottom_rect_height = rect_size * zoom_factor; // Height of one rectangle

    rectangle(
        white,
        [
            bottom_rect_x,
            bottom_rect_y,
            bottom_rect_width,
            bottom_rect_height,
        ],
        transform,
        graphics,
    );
}

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
