#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![windows_subsystem = "windows"]
use std::time::Instant;

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const INTERNAL_WIDTH: u32 = 256;
const INTERNAL_HEIGHT: u32 = 240;
const SCALE: u32 = 4;
const WIN_WIDTH: u32 = INTERNAL_WIDTH * SCALE;
const WIN_HEIGHT: u32 = INTERNAL_HEIGHT * SCALE;
const BOX_SIZE: usize = 16 * SCALE as usize;

const MAX_FRAME_RATE: f32 = 144.0;
const MAX_FRAME_TIME : f32 = 1.0 / MAX_FRAME_RATE;
struct World {
    tiles: Vec<Tile>,
    player: Player,
    ground_height: usize,
}
struct Player {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    y_velocity: usize,
    jumping: bool,
    falling: bool,
    sprinting: bool,
    sprite: Sprite,
}

// Sprite is a struct that holds a data property which is an array of 128 arrays of 4 values which represent an rgba value
struct Sprite{
    data: [[u8; 4]; 128]
}
struct Tile {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: [u8; 4],
}

fn main() -> Result<(), Error> {
    // Start event logger
    env_logger::init();

    // Create Event Loop
    let event_loop = EventLoop::new();

    // Initialize input helper
    let mut input = WinitInputHelper::new();
    let mut right_pressed = false;
    let mut left_pressed = false;
    let mut space_pressed = false;
    let mut shift_pressed = false;

    // Initialize window
    let window = {
        let size = LogicalSize::new(WIN_WIDTH as f64, WIN_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Bit World")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_resizable(true)
            .build(&event_loop)
            .unwrap()
    };

    // Initialize pixels
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIN_WIDTH as u32, WIN_HEIGHT as u32, surface_texture)?
    };
    // Create new game world from struct
    let mut world = World::new();


    // Create a character from code, TODO: Load this from a file
    // Create new player sprite
    // Data property is equal to an array of 128 arrays of 4 values which represent an rgba value
    // Example: data[0] = [255, 255, 255, 255] = white
    let mut newSprite = Sprite{
        data: [[255, 255, 255, 255]; 128]
    };

    // Set world.player to a new player instance
    world.player = Player {
        x: 15,
        y: WIN_HEIGHT as usize - BOX_SIZE,
        width: BOX_SIZE,
        height: BOX_SIZE,
        y_velocity: 0,
        jumping: false,
        falling: false,
        sprinting: false,
        sprite: newSprite
    };

    event_loop.run(move |event, _, control_flow| {
        // Only allow loop to run at 144 fps
        *control_flow = ControlFlow::WaitUntil(Instant::now() + std::time::Duration::from_secs_f32(MAX_FRAME_TIME));

        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame_mut());
            if let Err(err) = pixels.render() {
                error!("pixels.render() failed: {err}");
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
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
                world.move_character_right();
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
                world.move_character_left();
            }

            // Check if space key is pressed
            if input.key_pressed(VirtualKeyCode::Space) {
                space_pressed = true;
            }

            // Check if space key is released
            if input.key_released(VirtualKeyCode::Space) {
                space_pressed = false;
            }

            if space_pressed{
                world.character_jump();
            }

            // Check if left shift is pressed
            if input.key_pressed(VirtualKeyCode::LShift) {
                shift_pressed = true;
            }

            // Check if left shift is released
            if input.key_released(VirtualKeyCode::LShift) {
                shift_pressed = false;
            }

            world.player.sprinting = shift_pressed;

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    error!("pixels.resize_surface() failed: {err}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            // Update internal state and request a redraw
            world.update();
            window.request_redraw();
        }
    });
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        let mut tiles = Vec::new();
        for x in 0..(WIN_WIDTH as usize) {
            if x % BOX_SIZE as usize == 0 {
                tiles.push(Tile {
                    x: x as usize,
                    y: (WIN_HEIGHT as usize) - (BOX_SIZE),
                    width: BOX_SIZE,
                    height: BOX_SIZE,
                    color: [255, 255, 255, 255],
                });
            }
        }
        Self {
            player: Player {
                x: 15,
                y: WIN_HEIGHT as usize - BOX_SIZE - 12,
                width: BOX_SIZE,
                height: BOX_SIZE,
                y_velocity: 0,
                jumping: false,
                falling: false,
                sprinting: false,
                sprite: Sprite{
                    data: [[255, 0, 255, 255]; 128]
                }
            },
            tiles: tiles,
            ground_height: WIN_HEIGHT as usize - (BOX_SIZE * 2),
        }
        // Append row of tiles at bottom of screen that fit into window width
        
    }
    fn move_character_right(&mut self) {
        // Move player right, with sprinting logic
        if self.player.sprinting {
            self.player.x += 2;
        }else{
            self.player.x += 1;
        }
    }
    fn move_character_left(&mut self) {
        // Move player left, with sprinting logic
        if self.player.sprinting {
            self.player.x -= 2;
        }else{
            self.player.x -= 1;
        }
    }
    fn character_jump(&mut self) {
        // Jump logic, jump higher if sprinting
        if self.player.jumping == false && self.player.falling == false {
            if self.player.sprinting {
                self.player.y_velocity = 12;
            }else{
                self.player.y_velocity = 10;
            }
            self.player.jumping = true;
        }
    }
    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        // Jumping logic 
        if self.player.jumping{
            self.player.y -= self.player.y_velocity;
            self.player.y_velocity -= 1;
            if self.player.y_velocity == 0 {
                self.player.falling = true;
                self.player.jumping = false;
            }
        }else if self.player.y < self.ground_height && self.player.falling {
            self.player.y_velocity += 1;
            self.player.y += self.player.y_velocity;
        }else{
            self.player.falling = false;
            self.player.y_velocity = 0;
            self.player.y = self.ground_height;
        }
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {

        // Clear frame buffer
        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0, 0, 0, 255]);
        }

        // Interate through all tiles in World
        for tile in &self.tiles {
            // Iterate through all pixels in tile
            for tile_pixel_x in 0..(tile.width) { // For every pixel in the tile
                for tile_pixel_y in 0..(tile.height) {
                    // Calculate pixel index
                    let pixel_index = ((tile.x + tile_pixel_x) + ((tile.y + tile_pixel_y) * (WIN_WIDTH as usize)) as usize) as usize * 4;
                    if pixel_index < frame.len() { //Set pixel color if within window
                        frame[pixel_index] = tile.color[0];
                        frame[pixel_index + 1] = tile.color[1];
                        frame[pixel_index + 2] = tile.color[2];
                        frame[pixel_index + 3] = tile.color[3];
                    }
                }
            }
        }


        // Draw player
        &self.player.draw(frame);
    }
}

impl Player {
    fn draw(&self, frame: &mut [u8]) {
        // Draw player sprite
        // Calculate pixel index based on player position
        // Then set pixel color to player sprite color, the player sprite is 8x16
        // The yLine variable should increase by 1 every 8 pixels
        for (i, spritePixel) in self.sprite.data.iter().enumerate(){
            let pixel_index = ((self.x + i) + ((self.y + 0) * (WIN_WIDTH as usize)) as usize) as usize * 4;
            if pixel_index < frame.len() { //Set pixel color if within window
                frame[pixel_index] = spritePixel[0];
                frame[pixel_index + 1] = spritePixel[1];
                frame[pixel_index + 2] = spritePixel[2];
                frame[pixel_index + 3] = spritePixel[3];
            }
        }
        
    }
}
