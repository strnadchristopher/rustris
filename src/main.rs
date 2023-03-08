// #![deny(clippy::all)]
// #![forbid(unsafe_code)]
#![allow(dead_code)]
use simple_logger::SimpleLogger;
use log::{debug, error, info, warn, LevelFilter};
use pixels::{Error, Pixels, SurfaceTexture};
use std::time::Instant;
// use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use std::fs;

const INTERNAL_WIDTH: u32 = 256;
const INTERNAL_HEIGHT: u32 = 240;
const SCALE: u32 = 4;
const WIN_WIDTH: u32 = INTERNAL_WIDTH * SCALE;
const WIN_HEIGHT: u32 = INTERNAL_HEIGHT * SCALE;
const TILE_WIDTH: u32 = 16;
const TILES_PER_ROW: u32 = INTERNAL_WIDTH / TILE_WIDTH; // By default the total tiles in row would be 16

const TRANSPARENT_SPRITE: Sprite = Sprite {
    data: [[0, 0, 0, 0]; 256],
    width: 16,
    height: 16,
};
const DIRT_SPRITE: Sprite = Sprite {
    data: [[128, 0, 0, 255]; 256],
    width: 16,
    height: 16,
};
const GRASS_SPRITE: Sprite = Sprite {
    data: [[0, 255, 0, 255]; 256],
    width: 16,
    height: 16,
};
const TEST_TILE_TRANSPARENT: Tile = Tile {
    sprite: TRANSPARENT_SPRITE,
    id: 0,
};
const TEST_TILE_A: Tile = Tile {
    sprite: DIRT_SPRITE,
    id: 1,
};
const TEST_TILE_B: Tile = Tile {
    sprite: GRASS_SPRITE,
    id: 2,
};

const DEFAULT_MAP_STRING: &str = 
"1100000000000000
 0000000000000000
 0000000000000000
 0000000000000000
 0000000000000000
 0000000000000000
 0000000000000000
 0000000000000000
 0000000000000000
 0000000000000000
 0000000000000000
 0000000000000000
 0000000000000000
 1111111111111111
 1111111111111111";

const MAX_FRAME_RATE: f32 = 60.0;
const MAX_FRAME_TIME: f32 = 1.0 / MAX_FRAME_RATE;

struct World {
    tiles: Vec<Tile>,
}
impl World {
    /// Create a new `World` with it's sprite map
    fn new() -> Self {
        let mut tiles = Vec::new();
        Self { tiles }
    }
    /// Update the `World` internal state
    fn update(&mut self) {
        // Move Piece down
        // Check if piece can move down
        
        
        // If it can't, then lock the piece
        // Check if any rows are full
        // If they are, then remove them

        return;
    }

    fn move_piece_right(&mut self) {
        return;
    }

    fn move_piece_left(&mut self) {
        return;
    }

    fn move_piece_down(&mut self) {
        return;
    }

    fn rotate_piece(&mut self) {
        return;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8], camera_offset: (usize, usize)) {
        // Clear frame buffer
        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0, 0, 0, 255]);
        }

        // Draw a white border around the screen
        let draw_border = false;
        if draw_border{
            for x in 0..INTERNAL_WIDTH {
                for y in 0..INTERNAL_HEIGHT {
                    if x == 0 || x == INTERNAL_WIDTH - 1 || y == 0 || y == INTERNAL_HEIGHT - 1 {
                        let pixel_index = (y * INTERNAL_WIDTH + x) as usize * 4 as usize;
                        frame[pixel_index..pixel_index + 4].copy_from_slice(&[255, 255, 255, 255]);
                    }
                }
            }
        }

        // Draw tiles
        for (i, tile) in self.tiles.iter().enumerate() {
            tile.draw(frame, i, camera_offset);
        }
    }
}

// Derive copy
#[derive(Copy, Clone)]
struct Tile {
    sprite: Sprite,
    id: u8,
}
impl Tile {
    fn draw(&self, frame: &mut [u8], tile_index: usize, camera_offset: (usize, usize)) {
        let current_column = tile_index as u32 % TILES_PER_ROW;

        let current_row = tile_index as u32 / TILES_PER_ROW;

        let offset_x: i32 = (current_column * TILE_WIDTH) as i32 - camera_offset.0 as i32;
        let offset_y: i32 = (current_row * TILE_WIDTH) as i32 - camera_offset.1 as i32;
        
        // I.e. if tile index is 36, then 36 / 32 = 1, so the tile is in the 1st row
        warn!("Current tile number: {}", tile_index);
        warn!("Tile ID: {}", self.id);
        warn!("Current row: {}", current_row);
        warn!("Current column: {}", current_column);
        self.sprite
            .draw(frame, offset_x as i32, offset_y as i32);
    }
}

#[derive(Copy, Clone)]
struct Sprite {
    data: [[u8; 4]; 256],
    width: usize,
    height: usize,
}
impl Sprite {
    fn draw(&self, frame: &mut [u8], anchor_x: i32, anchor_y: i32) {
        // Loop through each pixel in the sprite
        let mut pixel_row: i32 = 0;
        for (i, pixel) in self.data.iter().enumerate() {
            // First we set the current pixel x coordinate to the current index of the pixel in the sprite data
            let mut current_pixel_x: i32 = i as i32;
            
            // We see if the current_pixel_x is greater than or equal to the width of the sprite
            if current_pixel_x >= self.width as i32 {
                // Pixel row will be equal to the number of times the width of the sprite can go into the current_pixel_x without going over
                pixel_row = current_pixel_x / self.width as i32;

                // If it is, then we need to reset the current_pixel_x to the remainder of the current_pixel_x divided by the width of the sprite
                current_pixel_x = current_pixel_x % 16;
            }
            
            let mut current_pixel_y = pixel_row;

            let next_pixel = (current_pixel_x + anchor_x, current_pixel_y + anchor_y);

            // Now we just need to find the index of the pixel in the frame buffer that would be at the location of the next_pixel tuple
            // The frame array is a 1d array, so we need to convert the 2d coordinates of the next_pixel tuple into a 1d index
            // The formula for this is: (y * width + x) * 4
            // Where y is the y coordinate of the pixel, x is the x coordinate of the pixel, and 4 is the number of bytes per pixel
            let mut pixel_index = (next_pixel.1 * INTERNAL_WIDTH as i32 + next_pixel.0) * 4;


            // Now we can draw the pixel to the frame buffer, as long as the pixel_index is within the bounds of the frame buffer
            if pixel_index + 4 <= frame.len() as i32 && pixel_index >= 0{
                frame[pixel_index as usize..pixel_index as usize + 4].copy_from_slice(pixel);
            }
        }
    }
}

// Starts the main loop of the game
fn main() -> Result<(), Error> {
    SimpleLogger::new()
    .with_level(LevelFilter::Warn)
    .init().unwrap();
    // Create Event Loop
    let event_loop = EventLoop::new();

    // Initialize input helper
    let mut input = WinitInputHelper::new();

    // Initialize window
    let window = {
        // let size = LogicalSize::new(WIN_WIDTH as f64, WIN_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Bit World")
            // .with_inner_size(size)
            // .with_min_inner_size(size)
            .with_resizable(true)
            .build(&event_loop)
            .unwrap()
    };

    // Initialize pixels
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        // Pixels::new(WIN_WIDTH as u32, WIN_HEIGHT as u32, surface_texture)?
        Pixels::new(INTERNAL_WIDTH, INTERNAL_HEIGHT, surface_texture)?
    };

    // Create an empty Vector of Tiles
    let mut tiles: Vec<Tile> = Vec::new();
    tiles.push(TEST_TILE_TRANSPARENT);
    tiles.push(TEST_TILE_A);
    tiles.push(TEST_TILE_B);


    // Create new game world from struct
    let mut world = World::new();

    // Load the map data from levels/level_00.data, read this as a result
    let map_string = fs::read_to_string("levels/level_00.data").unwrap_or(DEFAULT_MAP_STRING.to_string()).trim().to_string().replace("\n", "");


    let map_string: String = map_string.chars().filter(|c| !c.is_whitespace()).collect();
    // Print map_string 
    warn!("Map string: {}", map_string);

    // Iterate through string by character
    for c in map_string.chars() {
        // Convert character to u8
        let tile_id = c.to_digit(10).unwrap_or(0) as u8;

        // Add tile to world
        world.tiles.push(tiles[tile_id as usize]);
    }

    let mut right_pressed = false;
    let mut left_pressed = false;
    let mut up_pressed = false;
    let mut down_pressed = false;
    let mut camera_offset: (usize, usize) = (0, 0);
    event_loop.run(move |event, _, control_flow| {
        // Only allow loop to run at 144 fps
        *control_flow = ControlFlow::WaitUntil(
            Instant::now() + std::time::Duration::from_secs_f32(MAX_FRAME_TIME),
        );
        
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame_mut(), camera_offset);
            if let Err(err) = pixels.render() {
                error!("pixels.render() failed: {err}");
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    error!("pixels.resize_surface() failed: {err}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            // Check if right arrow key is down
            if input.key_pressed_os(VirtualKeyCode::Right) {
                right_pressed = true;
            }

            // Check if right arrow key is up
            if input.key_released(VirtualKeyCode::Right) {
                right_pressed = false;
            }

            if right_pressed{
                camera_offset.0 -=1;
            }

            // Check if left arrow key is down
            if input.key_pressed_os(VirtualKeyCode::Left) {
                left_pressed = true;
            }

            // Check if left arrow key is up
            if input.key_released(VirtualKeyCode::Left) {
                left_pressed = false;
            }

            if left_pressed{
                camera_offset.0 +=1;
            }

            // Check if up arrow key is down
            if input.key_pressed_os(VirtualKeyCode::Up) {
                up_pressed = true;
            }

            // Check if up arrow key is up
            if input.key_released(VirtualKeyCode::Up) {
                up_pressed = false;
            }

            if up_pressed{
                camera_offset.1 +=1;
            }

            // Check if down arrow key is down
            if input.key_pressed_os(VirtualKeyCode::Down) {
                down_pressed = true;
            }

            // Check if down arrow key is up
            if input.key_released(VirtualKeyCode::Down) {
                down_pressed = false;
            }

            if down_pressed{
                camera_offset.1 -=1;
            }
        }
        // Update internal state and request a redraw
        world.update();
        window.request_redraw();
    });
}
