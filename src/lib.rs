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

    fill_color: u32,
}

impl Sketch {
    fn from_size(width: usize, height: usize) -> Sketch {
        let window = minifb::Window::new(DEFAULT_NAME, width, height, minifb::WindowOptions::default())
            .unwrap_or_else(|e| {
                panic!("Unable to open window: {}", e);
            });

        let pixels: Vec<u32> = vec![0; width*height];

        Sketch {window, pixels, width, height, is_looping: true, frame_count: 0, delta_time: 0.0, mouse_x: 0.0, mouse_y: 0.0, fill_color: 0}
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

    fn framerate(&mut self, fps: usize) {
        self.window.set_target_fps(fps);
    }

    fn rbg_color(r: u8, g: u8, b: u8) -> u32 {
        ((r as u32) << 16) | ((g as u32) << 8) | b as u32
    }

    fn rgba_color(a: u8, r: u8, g: u8, b: u8) -> u32 {
        ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        self.pixels[x as usize + y as usize * self.width] = color;
    }

    fn fill(&mut self, color: u32) {
        self.fill_color = color;
    }

    fn fill_pixel(&mut self, x: u32, y: u32) {
         self.set_pixel(x, y, self.fill_color);
    }

    fn rect(&mut self, x: u32, y: u32, w: u32, h: u32) {
        for i in x..(x+w) {
            for j in y..(y+h) {
                self.fill_pixel(i, j);
            }
        }
    }

    fn background(&mut self, color: u32) {
        for i in 0..self.width as u32 {
            for j in 0..self.height as u32 {
                self.set_pixel(i, j, color);
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    impl Runnable for Sketch {
        fn setup(&mut self) {
            println!("SETUP WAS CALLED");
            self.framerate(60);
        }

        fn draw(&mut self) {
            if self.frame_count == 0 {
                println!("FIRST DRAW CALL");
            }

            let blue: u32 = Self::rbg_color(50, 50, 255);
            let green: u32 = Self::rbg_color(50, 255, 50);

            self.background(blue);

            self.fill(green);
            self.rect(50, 100, 200, 100);
        }
    }

    #[test]
    fn testing() {
        Sketch::from_size(640, 480).run();

        println!("TESTING DONE")
    }
}
