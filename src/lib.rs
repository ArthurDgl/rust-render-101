use minifb;
use minifb::MouseMode;

const DEFAULT_NAME: &str = "Rust Render 101 Sketch";

pub trait Runnable {
    fn setup(&mut self);
    fn draw(&mut self);
}

pub struct Sketch {
    window: minifb::Window,
    pixels: Vec<u32>,

    width: usize,
    height: usize,

    is_looping: bool,
    frame_count: u32,
    delta_time: f32,

    mouse_x: f32,
    mouse_y: f32,
}

impl Sketch {
    fn from_size(width: usize, height: usize) -> Sketch {
        let mut window = minifb::Window::new(DEFAULT_NAME, width, height, minifb::WindowOptions::default())
            .unwrap_or_else(|e| {
                panic!("Unable to open window: {}", e);
            });

        let mut pixels: Vec<u32> = vec![0; width*height];

        Sketch {window, pixels, width, height, is_looping: true, frame_count: 0, delta_time: 0.0, mouse_x: 0.0, mouse_y: 0.0}
    }

    fn run(&mut self) {
        self.setup();

        let mut now = std::time::SystemTime::now();

        while self.window.is_open() {
            if self.is_looping {
                self.delta_time = now.elapsed().unwrap().as_secs_f32();
                now = std::time::SystemTime::now();

                (self.mouse_x, self.mouse_y) = self.window.get_mouse_pos(MouseMode::Clamp).unwrap();

                self.draw();
            }

            self.window.update_with_buffer(&self.pixels, self.width, self.height).unwrap();

            if self.is_looping {
                self.frame_count += 1;
            }
        }
    }

    fn no_loop(&mut self) {
        self.is_looping = false;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    impl Runnable for Sketch {
        fn setup(&mut self) {
            println!("SETUP WAS CALLED");
        }

        fn draw(&mut self) {
            println!("DRAW WAS CALLED");

            if self.frame_count >= 5 {
                self.no_loop();
            }
        }
    }

    #[test]
    fn testing() {
        Sketch::from_size(640, 480).run();

        println!("TESTING DONE")
    }
}
