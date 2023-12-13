#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![windows_subsystem = "windows"]

use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use rayon::prelude::*;

const WIDTH:  u32 = 1024;
const HEIGHT: u32 = 1024;

const HALF_WIDTH: u32 = WIDTH/2;
const HALF_HEIGHT: u32 = HEIGHT/2;

struct World {
    iteration_number: u16,
    scale: f64,
    x_offset: f64,
    y_offset: f64,
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Mandelbrot")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.frame_mut());
            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        
        // Handle input events
        if input.update(&event) {
            // Close events
            if input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            
            let scroll_diff = input.scroll_diff();
            if scroll_diff != 0.0 {
                if scroll_diff < 0.0 {
                    world.scale *= 0.9;
                }  else {
                    world.scale *= 1.1;
                }
            }
            
            if input.mouse_held(0) {
                world.x_offset -= (input.mouse_diff().0 as f64/HALF_WIDTH  as f64/world.scale) as f64;
                world.y_offset -= (input.mouse_diff().1 as f64/HALF_HEIGHT as f64/world.scale) as f64;
            }

            // Update internal state and request a redraw
            // world.update();
            window.request_redraw();
        }
    });
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

fn square_complex(x: f64, y: f64) -> [f64; 2] {
    let return_x: f64 = x*x - y*y;
    let return_y: f64 = 2.0*x*y;
    return [return_x, return_y];
}

impl World {
    fn new() -> Self {
        Self {
            iteration_number: 255,
            scale: 0.5,
            x_offset: 0.0,
            y_offset: 0.0,
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        frame.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {

            let x = i % WIDTH as usize;
            let y = i / WIDTH as usize;
            
            let relative_x = x as f64/((HALF_WIDTH  as f64) * self.scale)+self.x_offset-(1.0 / self.scale) as f64;
            let relative_y = y as f64/((HALF_HEIGHT as f64) * self.scale)+self.y_offset-(1.0 / self.scale) as f64;

            let c = self.calculate_mandelbrot([relative_x, relative_y]);
            let rgba: [u8; 4] = [c, c, c, 255];
            
            pixel.copy_from_slice(&rgba);
        });
    }

    fn calculate_mandelbrot(&self, c: [f64; 2]) -> u8 {
        let mut z: [f64; 2] = [0.0, 0.0];
        let mut x;
        let mut y;

        for i in 0..self.iteration_number {
            x = z[0] + c[0];
            y = z[1] + c[1];
            if ((x*x + y*y) as f64).sqrt() > 64.0 {
                return (i*1).min(255) as u8;
            }
            z = square_complex(x, y)
        }
        return 0;
    }
}