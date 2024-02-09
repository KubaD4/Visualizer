extern crate piston_window;

use piston_window::*;
use piston_window::{Context, G2d, rectangle};
use piston_window::types::{Color, ColorComponent};

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

pub const MAP_SIZE: usize = 500;
pub const GRID_SIZE: (usize, usize) = (MAP_SIZE, MAP_SIZE);
// Assuming RECT_SIZE is defined to fit the grid within the window minus padding
// And since we want the window 200 pixels greater than the grid, we adjust the calculation accordingly
pub const RECT_SIZE: f64 = (/*MAP_SIZE as f64*/750.0 /*- 200.0*/) / MAP_SIZE as f64;
pub const WINDOW_SIZE: (usize, usize) = (
    950,
    950,
);

//((MAP_SIZE as f64 * RECT_SIZE + 200.0) as usize, (MAP_SIZE as f64 * RECT_SIZE + 200.0) as usize, );
pub const ZOOM_AMOUNT: f64 = 0.35;
pub const SCROLL_AMOUNT: f64 = 5.0;

pub const ROBOT_COLOR: [f32; 4] = [191.0 / 255.0, 139.0 / 255.0, 255.0 / 255.0, 1.0];

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
        for (j, &rect_color) in row.iter().enumerate() {
            // Calculate the position for each rectangle
            let x = grid_start_x + (j as f64 * rect_size);
            let y = grid_start_y + (i as f64 * rect_size);

            // Draw the rectangle
            rectangle(
                rect_color,
                [x, y, rect_size, rect_size],
                context.transform,
                graphics,
            );


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

fn convert_colors(color:  [f64; 4]) ->  [f64; 4] {
    [color.get(0).unwrap()/255.0,color.get(1).unwrap()/255.0,color.get(2).unwrap()/255.0,1.0]
    //chiamo la funzione con valori decisi quindi so che contengono un valore -> unwrap non esplode
}

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
