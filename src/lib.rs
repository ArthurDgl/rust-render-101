use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window};
use image::{ImageBuffer, Rgb};
use image::error::UnsupportedErrorKind::Color;
use rand;

const DEFAULT_NAME: &str = "Rust Render 101 Sketch";

pub enum StrokeMode{
    Circle,
    Square,
    Custom(fn(i8) -> Vec<(i8, i8)>),
}

pub struct RgbaColor {}

impl RgbaColor {
    pub fn random_rgb_color() -> u32 {
        let mask: u32 = !(255u32 << 24);

        mask & rand::random::<u32>()
    }

    pub fn random_rgba_color() -> u32 {
        rand::random::<u32>()
    }

    pub fn greyscale_color(g: u8) -> u32 {
        ((g as u32) << 16) | ((g as u32) << 8) | g as u32
    }

    pub fn rgb_color(r: u8, g: u8, b: u8) -> u32 {
        ((r as u32) << 16) | ((g as u32) << 8) | b as u32
    }

    pub fn rgba_color(a: u8, r: u8, g: u8, b: u8) -> u32 {
        ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32
    }

    pub fn color_alpha(color: u32) -> u8 {
        (color >> 24) as u8
    }

    pub fn color_red(color: u32) -> u8 {
        (color >> 16) as u8
    }

    pub fn color_green(color: u32) -> u8 {
        (color >> 8) as u8
    }

    pub fn color_blue(color: u32) -> u8 {
        color as u8
    }
}

pub struct Geometry {}

impl Geometry {
    pub fn random(lower: f32, upper: f32) -> f32 {
        lower + rand::random::<f32>() * (upper - lower)
    }
}

pub trait State : Default {}

pub struct Sketch<S: State> {
    window: minifb::Window,
    pixels: Vec<u32>,

    pub width: usize,
    pub height: usize,

    pub is_looping: bool,
    pub frame_count: u32,
    pub delta_time: f32,

    pub mouse_x: f32,
    pub mouse_y: f32,
    pub mouse_is_pressed: bool,
    pub mouse_button: MouseButton,

    fill_color: Option<u32>,
    stroke_color: Option<u32>,
    stroke_weight: i8,
    stroke_mode: StrokeMode,

    pub draw_method: Option<fn(&mut Self)>,
    pub setup_method: Option<fn(&mut Self)>,
    pub mouse_pressed_method: Option<fn(&mut Self)>,
    pub mouse_released_method: Option<fn(&mut Self)>,
    pub key_pressed_method: Option<fn(&mut Self, Key)>,
    pub key_released_method: Option<fn(&mut Self, Key)>,

    pub state: S,
}

impl<S: State> Sketch<S> {
    pub fn from_size(width: usize, height: usize, state: S) -> Sketch<S> {
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
            fill_color: Some(0),
            stroke_color: Some(0),
            stroke_weight: 1,
            stroke_mode: StrokeMode::Circle,

            draw_method: None,
            setup_method: None,
            mouse_pressed_method: None,
            mouse_released_method: None,
            key_pressed_method: None,
            key_released_method: None,
            state,
        }
    }

    // Private Methods

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
                if let Some(mouse_released_method) = self.mouse_released_method {
                    mouse_released_method(self);
                }
            }
            self.mouse_is_pressed = false;
        }

        if !temp && self.mouse_is_pressed {
            if let Some(mouse_pressed_method) = self.mouse_pressed_method {
                mouse_pressed_method(self);
            }
        }
    }

    fn handle_keys(&mut self) {
        let keys_pressed:Vec<Key> = self.window.get_keys_pressed(KeyRepeat::No);

        for key in keys_pressed {
            if let Some(key_pressed_method) = self.key_pressed_method {
                key_pressed_method(self, key);
            }
        }

        let keys_released:Vec<Key> = self.window.get_keys_released();

        for key in keys_released {
            if let Some(key_released_method) = self.key_released_method {
                key_released_method(self, key);
            }
        }
    }

    fn run(&mut self) {
        self.setup_method.expect("Setup method was not set !")(self);

        let mut now = std::time::SystemTime::now();

        while self.window.is_open() {
            if self.is_looping {
                self.delta_time = now.elapsed().unwrap().as_secs_f32();
                now = std::time::SystemTime::now();

                self.handle_mouse();
                self.handle_keys();

                self.draw_method.expect("Draw method was not set !")(self);
            }

            self.window.update_with_buffer(&self.pixels, self.width, self.height).unwrap();

            if self.is_looping {
                self.frame_count = self.frame_count + 1;
            }
        }
    }

    fn bresenham_plot_line_pixel(&mut self, x: i32, y: i32, mask: &Vec<(i8, i8)>) {
        for (i, j) in mask {
            let (xi, yj) = (x + *i as i32, y + *j as i32);
            if xi >= 0 && yj >= 0 {
                self.stroke_pixel(xi as u32, yj as u32);
            }
        }
    }

    fn bresenham_plot_line_low(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
        let dx = x1 - x0;
        let mut dy = y1 - y0;
        let mut yi = 1;
        if dy < 0 {
            yi = -1;
            dy = -dy;
        }
        let mut delta = 2 * dy - dx;
        let mut y = y0;

        let mut result: Vec<(i32, i32)> = vec![];

        for x in x0..=x1 {
            result.push((x, y));
            if delta > 0 {
                y += yi;
                delta += 2 * (dy - dx);
            }
            else {
                delta += 2 * dy;
            }
        }
        result
    }

    fn bresenham_plot_line_high(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
        let mut dx = x1 - x0;
        let dy = y1 - y0;
        let mut xi = 1;
        if dx < 0 {
            xi = -1;
            dx = -dx;
        }
        let mut delta = 2 * dx - dy;
        let mut x = x0;

        let mut result: Vec<(i32, i32)> = vec![];

        for y in y0..=y1 {
            result.push((x, y));
            if delta > 0 {
                x += xi;
                delta += 2 * (dx - dy);
            }
            else {
                delta += 2 * dx;
            }
        }
        result
    }

    fn bresenham_plot_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
        let points_to_plot: Vec<(i32, i32)>;

        if (y1 - y0).abs() < (x1 - x0).abs() {
            if x0 > x1 {
                points_to_plot = self.bresenham_plot_line_low(x1, y1, x0, y0);
            }
            else {
                points_to_plot = self.bresenham_plot_line_low(x0, y0, x1, y1);
            }
        }
        else {
            if y0 > y1 {
                points_to_plot = self.bresenham_plot_line_high(x1, y1, x0, y0);
            }
            else {
                points_to_plot = self.bresenham_plot_line_high(x0, y0, x1, y1);
            }
        }
        points_to_plot
    }

    fn bresenham_plot_line_mask(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, mask: &Vec<(i8, i8)>) {
        let points_to_plot = self.bresenham_plot_line(x0, y0, x1, y1);

        for point in points_to_plot {
            self.bresenham_plot_line_pixel(point.0, point.1, mask);
        }
    }

    fn generate_circular_mask(&self) -> Vec<(i8, i8)> {
        let mut mask: Vec<(i8, i8)> = Vec::new();

        let stroke_weight_sq = self.stroke_weight * self.stroke_weight;

        for x in -self.stroke_weight..=self.stroke_weight {
            for y in -self.stroke_weight..=self.stroke_weight {
                if x * x + y * y <= stroke_weight_sq {
                    mask.push((x, y));
                }
            }
        }
        mask
    }

    fn generate_square_mask(&self) -> Vec<(i8, i8)> {
        let v1 = -self.stroke_weight;
        let v2 = self.stroke_weight;

        let mut mask: Vec<(i8, i8)> = Vec::new();

        for v in v1..=v2 {
            mask.push((v1, v));
            mask.push((v2, v));

            mask.push((v, v1));
            mask.push((v, v2));
        }
        mask
    }

    fn triangle_flat_top(&mut self, x0: i32, y0: i32, base: i32, x1: i32, y1: i32) {
        let left_edge = self.bresenham_plot_line(x0, y0, x1, y1);
        let right_edge = self.bresenham_plot_line(x0 + base, y0, x1, y1);

        let mut max_right_x: Vec<i32> = vec![i32::MIN; (y1 - y0 + 1) as usize];
        let mut min_left_x: Vec<i32> = vec![i32::MAX; (y1 - y0 + 1) as usize];

        for (x, y) in right_edge {
            let i = (y - y0) as usize;
            if x > max_right_x[i] {
                max_right_x[i] = x;
            }
        }
        for (x, y) in left_edge {
            let i = (y - y0) as usize;
            if x < min_left_x[i] {
                min_left_x[i] = x;
            }
        }

        for i in 0..max_right_x.len() {
            let y = i as i32 + y0;
            for x in min_left_x[i]..=max_right_x[i] {
                if x >= 0 && y >= 0 {
                    self.set_pixel(x as u32, y as u32, self.fill_color.expect(""));
                }
            }
        }
    }

    fn triangle_flat_bottom(&mut self, x0: i32, y0: i32, base: i32, x1: i32, y1: i32) {
        let left_edge = self.bresenham_plot_line(x1, y1, x0, y0);
        let right_edge = self.bresenham_plot_line(x1, y1, x0 + base, y0);

        let mut max_right_x: Vec<i32> = vec![i32::MIN; (y0 - y1 + 1) as usize];
        let mut min_left_x: Vec<i32> = vec![i32::MAX; (y0 - y1 + 1) as usize];

        for (x, y) in right_edge {
            let i = (y - y1) as usize;
            if x > max_right_x[i] {
                max_right_x[i] = x;
            }
        }
        for (x, y) in left_edge {
            let i = (y - y1) as usize;
            if x < min_left_x[i] {
                min_left_x[i] = x;
            }
        }

        for i in 0..max_right_x.len() {
            let y = i as i32 + y1;
            for x in min_left_x[i]..=max_right_x[i] {
                if x >= 0 && y >= 0 {
                    self.set_pixel(x as u32, y as u32, self.fill_color.expect(""));
                }
            }
        }
    }

    fn triangle_fill(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32) {
        let mut triangle = [(x0, y0), (x1, y1), (x2, y2)];
        triangle.sort_by(|&(_, y1), &(_, y2)| y2.cmp(&y1));

        let x_project = f32::round(
            (triangle[1].1 - triangle[2].1) as f32
                * ((triangle[2].0 - triangle[0].0) as f32 / (triangle[2].1 - triangle[0].1) as f32)
                + triangle[2].0 as f32
        ) as i32;

        let mut x_start = x_project;
        let mut base = triangle[1].0 - x_project;
        if triangle[1].0 < x_project {
            x_start = triangle[1].0;
            base = -base;
        }

        self.triangle_flat_bottom(x_start, triangle[1].1, base, triangle[2].0, triangle[2].1);
        self.triangle_flat_top(x_start, triangle[1].1, base, triangle[0].0, triangle[0].1);
    }

    fn triangle_stroke(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32) {
        self.line(x0, y0, x1, y1);
        self.line(x0, y0, x2, y2);
        self.line(x2, y2, x1, y1);
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        let index = x as usize + y as usize * self.width;
        if index < 0 || index >= self.pixels.len() {
            return;
        };
        self.pixels[index] = color;
    }

    fn rect_fill(&mut self, x: u32, y: u32, w: u32, h: u32) {
        for i in x..(x+w) {
            for j in y..(y+h) {
                self.fill_pixel(i, j);
            }
        }
    }

    fn rect_stroke(&mut self, x: u32, y: u32, w: u32, h: u32) {
        let x = x as i32;
        let y = y as i32;
        let w = w as i32;
        let h = h as i32;
        self.line(x, y, x+w, y);
        self.line(x, y, x, y+h);
        self.line(x, y+h, x+w, y+h);
        self.line(x+w, y, x+w, y+h);
    }

    // Public Methods

    pub fn name(&mut self, name: &str) {
        self.window.set_title(name);
    }

    pub fn key_is_down(&self, key: Key) -> bool{
        self.window.is_key_down(key)
    }

    pub fn no_loop(&mut self) {
        self.is_looping = false;
    }

    pub fn framerate(&mut self, fps: usize) {
        self.window.set_target_fps(fps);
    }

    pub fn fill(&mut self, color: u32) {
        self.fill_color = Some(color);
    }

    pub fn no_fill(&mut self) {
        self.fill_color = None;
    }

    pub fn stroke(&mut self, color: u32) {
        self.stroke_color = Some(color);
    }

    pub fn no_stroke(&mut self) {
        self.stroke_color = None;
    }

    pub fn stroke_weight(&mut self, weight: i8) {
        self.stroke_weight = weight;
    }

    pub fn stroke_mode(&mut self, mode: StrokeMode) {
        self.stroke_mode = mode;
    }

    pub fn fill_pixel(&mut self, x: u32, y: u32) {
         if let Some(color) = self.fill_color {
             self.set_pixel(x, y, color);
         }
    }

    pub fn stroke_pixel(&mut self, x: u32, y: u32) {
        if let Some(color) = self.stroke_color {
            self.set_pixel(x, y, color);
        }
    }

    pub fn rect(&mut self, x: u32, y: u32, w: u32, h: u32) {
        if self.fill_color.is_some() {
            self.rect_fill(x, y, w, h);
        }
        if self.stroke_color.is_some() {
            self.rect_stroke(x, y, w, h);
        }
    }

    pub fn triangle(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32) {
        if self.fill_color.is_some() {
            self.triangle_fill(x0, y0, x1, y1, x2, y2);
        }
        if self.stroke_color.is_some() {
            self.triangle_stroke(x0, y0, x1, y1, x2, y2);
        }
    }

    pub fn background(&mut self, color: u32) {
        for i in 0..self.width as u32 {
            for j in 0..self.height as u32 {
                self.set_pixel(i, j, color);
            }
        }
    }

    pub fn save(&mut self, file_path: &str) {
        let mut image = ImageBuffer::new(self.width as u32, self.height as u32);

        for x in 0..self.width as u32 {
            for y in 0..self.height as u32 {
                let pixel: u32 = self.pixels[x as usize + y as usize * self.width];
                image.put_pixel(x, y, Rgb([RgbaColor::color_red(pixel), RgbaColor::color_green(pixel), RgbaColor::color_blue(pixel)]));
            }
        }

        image.save(file_path).unwrap_or_else(|e| {
            panic!("Unable to save screenshot : {}", e);
        });
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        let mask = match self.stroke_mode {
            StrokeMode::Circle => self.generate_circular_mask(),
            StrokeMode::Square => self.generate_square_mask(),
            StrokeMode::Custom(mask_func) => mask_func(self.stroke_weight),
        };

        self.bresenham_plot_line_mask(x0, y0, x1, y1, &mask);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MyState {
        line_x1: i32,
        line_y1: i32,
        line_x2: i32,
        line_y2: i32,
        background_color: u32,
    }

    impl State for MyState {}

    fn setup(sketch: &mut Sketch<MyState>) {
        println!("SETUP WAS CALLED");
        sketch.framerate(60);
        sketch.name("Example Sketch")
    }

    fn draw(sketch: &mut Sketch<MyState>) {
        if sketch.frame_count == 0 {
            println!("FIRST DRAW CALL");
        }

        let green: u32 = RgbaColor::rgb_color(50, 255, 50);
        let gray: u32 = RgbaColor::rgb_color(50, 50, 50);

        sketch.background(sketch.state.background_color);

        sketch.fill(green);
        sketch.stroke(gray);
        sketch.stroke_weight(2);
        sketch.stroke_mode(StrokeMode::Square);
        sketch.rect(50, 100, 200, 100);

        sketch.fill(gray);
        sketch.stroke(green);
        sketch.stroke_weight(3);
        sketch.stroke_mode(StrokeMode::Circle);
        sketch.triangle(350, 50, 450, 150, 300, 300);

        sketch.stroke(RgbaColor::rgb_color(255, 50, 255));
        sketch.stroke_weight(5);
        sketch.stroke_mode(StrokeMode::Circle);
        sketch.line(sketch.state.line_x1, sketch.state.line_y1, sketch.state.line_x2, sketch.state.line_y2);
    }

    fn mouse_pressed(sketch: &mut Sketch<MyState>) {
        let (x, y) = (sketch.mouse_x as i32, sketch.mouse_y as i32);

        match sketch.mouse_button {
            MouseButton::Left => (sketch.state.line_x1, sketch.state.line_y1) = (x, y),
            MouseButton::Right => (sketch.state.line_x2, sketch.state.line_y2) = (x, y),
            _ => (),
        };
    }

    fn key_pressed(sketch: &mut Sketch<MyState>, key: Key) {
        if key == Key::Space {
            sketch.state.background_color = RgbaColor::random_rgb_color();
        } else if key == Key::S {
            sketch.save("screenshot.png");
        }
    }

    #[test]
    fn testing() {
        let mut state = MyState::default();
        state.background_color = RgbaColor::random_rgb_color();
        let mut sketch = Sketch::<MyState>::from_size(640, 480, state);

        sketch.setup_method = Some(setup);
        sketch.draw_method = Some(draw);

        sketch.mouse_pressed_method = Some(mouse_pressed);
        sketch.key_pressed_method = Some(key_pressed);

        sketch.run();

        println!("TESTING DONE")
    }
}
