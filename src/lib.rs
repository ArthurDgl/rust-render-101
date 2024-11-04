use minifb;
use minifb::{Key, KeyRepeat, MouseButton, MouseMode};
use image::{ImageBuffer, Rgb};

const DEFAULT_NAME: &str = "Rust Render 101 Sketch";

pub trait Runnable {
    fn setup(&mut self);
    fn draw(&mut self);

    fn mouse_pressed(&mut self) {
        // Optional Implementation
    }

    fn mouse_released(&mut self) {
        // Optional Implementation
    }

    fn key_pressed(&mut self, key: Key) {
        // Optional Implementation
    }

    fn key_released(&mut self, key: Key) {
        // Optional Implementation
    }
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
    mouse_is_pressed: bool,
    mouse_button: MouseButton,

    fill_color: u32,
}

impl Sketch {
    fn from_size(width: usize, height: usize) -> Sketch {
        let window = minifb::Window::new(DEFAULT_NAME, width, height, minifb::WindowOptions::default())
            .unwrap_or_else(|e| {
                panic!("Unable to open window: {}", e);
            });

        let pixels: Vec<u32> = vec![0; width*height];

        Sketch {
            window,
            pixels,
            width,
            height,
            is_looping: true,
            frame_count: 0,
            delta_time: 0.0,
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_is_pressed: false,
            mouse_button: MouseButton::Left,
            fill_color: 0,
        }
    }

    fn handle_mouse(&mut self) {
        (self.mouse_x, self.mouse_y) = self.window.get_mouse_pos(MouseMode::Clamp).unwrap();

        let temp = self.mouse_is_pressed;
        if self.window.get_mouse_down(MouseButton::Left) {
            self.mouse_is_pressed = true;
            self.mouse_button = MouseButton::Left;
        }
        else if self.window.get_mouse_down(MouseButton::Right) {
            self.mouse_is_pressed = true;
            self.mouse_button = MouseButton::Right;
        }
        else if self.window.get_mouse_down(MouseButton::Middle) {
            self.mouse_is_pressed = true;
            self.mouse_button = MouseButton::Middle;
        }
        else {
            if temp {
                self.mouse_released();
            }
            self.mouse_is_pressed = false;
        }

        if !temp && self.mouse_is_pressed {
            self.mouse_pressed();
        }
    }

    fn key_is_down(&self, key: Key) -> bool{
        self.window.is_key_down(key)
    }

    fn handle_keys(&mut self) {
        let keys_pressed:Vec<Key> = self.window.get_keys_pressed(KeyRepeat::No);

        for key in keys_pressed {
            self.key_pressed(key);
        }

        let keys_released:Vec<Key> = self.window.get_keys_released();

        for key in keys_released {
            self.key_released(key);
        }
    }

    fn run(&mut self) {
        self.setup();

        let mut now = std::time::SystemTime::now();

        while self.window.is_open() {
            if self.is_looping {
                self.delta_time = now.elapsed().unwrap().as_secs_f32();
                now = std::time::SystemTime::now();

                self.handle_mouse();
                self.handle_keys();

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

    fn color_alpha(color: u32) -> u8 {
        (color >> 24) as u8
    }

    fn color_red(color: u32) -> u8 {
        (color >> 16) as u8
    }

    fn color_green(color: u32) -> u8 {
        (color >> 8) as u8
    }

    fn color_blue(color: u32) -> u8 {
        color as u8
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

    fn save(&self, file_path: &str) {
        let mut image = ImageBuffer::new(self.width as u32, self.height as u32);

        for x in 0..self.width as u32 {
            for y in 0..self.height as u32 {
                let pixel: u32 = self.pixels[x as usize + y as usize * self.width];
                image.put_pixel(x, y, Rgb([Self::color_red(pixel), Self::color_green(pixel), Self::color_blue(pixel)]));
            }
        }

        image.save(file_path).unwrap_or_else(|e| {
            panic!("Unable to save screenshot : {}", e);
        });
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    impl Runnable for Sketch {
        fn setup(&mut self) {
            println!("SETUP WAS CALLED");
            self.framerate(60);

            let blue: u32 = Self::rbg_color(50, 50, 255);
            self.background(blue);
        }

        fn draw(&mut self) {
            if self.frame_count == 0 {
                println!("FIRST DRAW CALL");
            }

            let green: u32 = Self::rbg_color(50, 255, 50);

            self.fill(green);
            self.rect(50, 100, 200, 100);
        }

        fn mouse_pressed(&mut self) {
            let red: u32 = Self::rbg_color(255, 50, 50);

            self.rect(self.mouse_x as u32, self.mouse_y as u32, 10, 10);
        }

        fn key_pressed(&mut self, key: Key) {
            if key == Key::Space {
                self.background(Self::rbg_color(0, 0, 0));
            }
            else if key == Key::S {
                self.save("screenshot.png");
            }
        }
    }

    #[test]
    fn testing() {
        Sketch::from_size(640, 480).run();

        println!("TESTING DONE")
    }
}
