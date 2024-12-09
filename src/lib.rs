use earcutr;
use fontdue::{Font, FontSettings, Metrics};
use image::{ImageBuffer, Pixel, Rgb};
use minifb::{Key, KeyRepeat, MouseButton, MouseMode};
use rand;
use std::fs;
use std::io::Read;
use std::time::Duration;

const DEFAULT_NAME: &str = "Rust Render 101 Sketch";

pub enum StrokeMode {
    Circle,
    Square,
    Custom(fn(i8) -> Vec<(i8, i8)>),
}

pub enum FontMode {
    TimesNewRoman,
    Arial,
    Custom {file_path: String},
}

pub enum ShapeType {
    Polygon,
    LinearSpline { loops: bool },
    CubicBezierSpline { loops: bool },
}

pub struct RgbaColor {}

impl RgbaColor {
    /// Creates random opaque rgb color
    pub fn random_rgb_color() -> u32 {
        let mask: u32 = !(255u32 << 24);

        mask & rand::random::<u32>()
    }

    /// Creates random rgba color
    /// (can be transparent, see RgbaColor::random_rgb_color() for opaque colors)
    pub fn random_rgba_color() -> u32 {
        rand::random::<u32>()
    }

    /// Creates rgba value based on greyscale u8 value
    pub fn greyscale_color(g: u8) -> u32 {
        255u32 << 24 | ((g as u32) << 16) | ((g as u32) << 8) | g as u32
    }

    /// Creates rgba value based on 3 rgb u8 values
    pub fn rgb_color(r: u8, g: u8, b: u8) -> u32 {
        255u32 << 24 | ((r as u32) << 16) | ((g as u32) << 8) | b as u32
    }

    /// Creates rgba value based on 4 u8 values
    pub fn argb_color(a: u8, r: u8, g: u8, b: u8) -> u32 {
        ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32
    }

    /// Extracts alpha channel as u8 from u32 color
    pub fn color_alpha(color: u32) -> u8 {
        (color >> 24) as u8
    }

    /// Extracts red channel as u8 from u32 color
    pub fn color_red(color: u32) -> u8 {
        (color >> 16) as u8
    }

    /// Extracts green channel as u8 from u32 color
    pub fn color_green(color: u32) -> u8 {
        (color >> 8) as u8
    }

    /// Extracts blue channel as u8 from u32 color
    pub fn color_blue(color: u32) -> u8 {
        color as u8
    }

    fn color_u32_to_4xf32(color: u32) -> (f32, f32, f32, f32) {
        (
            RgbaColor::color_alpha(color) as f32 / 255f32,
            RgbaColor::color_red(color) as f32 / 255f32,
            RgbaColor::color_green(color) as f32 / 255f32,
            RgbaColor::color_blue(color) as f32 / 255f32,
        )
    }

    fn color_4xf32_to_u32(color: (f32, f32, f32, f32)) -> u32 {
        RgbaColor::argb_color(
            (color.0 * 255f32) as u8,
            (color.1 * 255f32) as u8,
            (color.2 * 255f32) as u8,
            (color.3 * 255f32) as u8,
        )
    }

    fn alpha_compose_alpha(p_a: f32, q_a: f32) -> f32 {
        p_a + q_a - p_a * q_a
    }

    fn alpha_compose_channel(p_a: f32, p_c: f32, q_a: f32, q_c: f32, r_a: f32) -> f32 {
        (p_c * p_a + q_c * q_a - p_c * p_a * q_a) / r_a
    }

    fn color_alpha_compose_color(color_p: u32, color_q: u32) -> u32 {
        let (p_a, p_r, p_g, p_b) = RgbaColor::color_u32_to_4xf32(color_p);
        let (q_a, q_r, q_g, q_b) = RgbaColor::color_u32_to_4xf32(color_q);

        let result_a = RgbaColor::alpha_compose_alpha(p_a, q_a);
        if result_a <= 0.0001f32 {
            return RgbaColor::argb_color(0, 0, 0, 0);
        }

        let (result_r, result_g, result_b) = (
            RgbaColor::alpha_compose_channel(p_a, p_r, q_a, q_r, result_a),
            RgbaColor::alpha_compose_channel(p_a, p_g, q_a, q_g, result_a),
            RgbaColor::alpha_compose_channel(p_a, p_b, q_a, q_b, result_a),
        );

        RgbaColor::color_4xf32_to_u32((result_a, result_r, result_g, result_b))
    }
}

enum EasingType {
    Linear,
    SmoothStep,
    QuadIn,
    QuadOut,
}

impl EasingType {
    fn ease(&self, t: f32) -> f32 {
        match self {
            EasingType::Linear => t,
            EasingType::SmoothStep => {
                let t2 = t * t;
                -2f32 * t2 * t + 3f32 * t2
            }
            EasingType::QuadIn => t * t,
            EasingType::QuadOut => -t * t + 2f32 * t,
        }
    }
}

#[derive(Clone)]
enum TransitionTarget {
    Points { points: Vec<(i32, i32)> },
    Point { point: (i32, i32) },
}
impl TransitionTarget {
    fn interpolate(t: f32, start: &Self, end: &Self) -> Self {
        match (start, end) {
            (
                TransitionTarget::Points { points: start_points },
                TransitionTarget::Points { points: end_points },
            ) => {
                let new_points = start_points
                    .iter()
                    .zip(end_points.iter())
                    .map(|(&(sx, sy), &(ex, ey))| {
                        (
                            (sx as f32 * (1f32 - t) + ex as f32 * t) as i32,
                            (sy as f32 * (1f32 - t) + ey as f32 * t) as i32,
                        )
                    })
                    .collect();

                TransitionTarget::Points { points: new_points }
            }
            (
                TransitionTarget::Point { point: start_point },
                TransitionTarget::Point { point: end_point },
            ) => TransitionTarget::Point {
                point: (
                    (start_point.0 as f32 * (1f32 - t) + end_point.0 as f32 * t) as i32,
                    (start_point.1 as f32 * (1f32 - t) + end_point.1 as f32 * t) as i32,
                ),
            },
            _ => {
                panic!("Error: 'start' and 'end' parameters must be the same TransitionTarget type!")
            }
        }
    }
}

struct Transition {
    duration: f32,
    elapsed: f32,
    easing: EasingType,
    start_state: TransitionTarget,
    end_state: TransitionTarget,
    current_state: TransitionTarget,
}

impl Transition {
    fn initialize(
        easing: EasingType,
        duration: f32,
        start_state: TransitionTarget,
        end_state: TransitionTarget,
    ) -> Self {
        let current_state = start_state.clone();
        Transition {
            easing,
            duration,
            elapsed: 0.0,
            start_state,
            end_state,
            current_state,
        }
    }

    fn step(&mut self, delta_time: f32) {
        self.elapsed += delta_time;
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        let eased_t = self.easing.ease(t);
        self.current_state =
            TransitionTarget::interpolate(eased_t, &self.start_state, &self.end_state);
    }

    fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }

    fn get_current_points(&self) -> &Vec<(i32, i32)> {
        match &self.current_state {
            TransitionTarget::Points { points } => { points }
            _ => {panic!("Error: called get_points() on transition with non-points target !")}
        }
    }

    fn get_current_point(&self) -> &(i32, i32) {
        match &self.current_state {
            TransitionTarget::Point { point } => { point }
            _ => {panic!("Error: called get_point() on transition with non-point target !")}
        }
    }
}



pub struct Geometry {}

impl Geometry {
    /// Creates a random f32 value between lower and upper bounds
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

    shape_vertices: Vec<(i32, i32)>,
    shape_holes: Vec<usize>,
    shape_type: ShapeType,

    pub draw_method: Option<fn(&mut Self)>,
    pub setup_method: Option<fn(&mut Self)>,
    pub mouse_pressed_method: Option<fn(&mut Self)>,
    pub mouse_released_method: Option<fn(&mut Self)>,
    pub key_pressed_method: Option<fn(&mut Self, Key)>,
    pub key_released_method: Option<fn(&mut Self, Key)>,

    loaded_fonts: Vec<(Font, String)>,
    font_index: usize,

    pub state: S,
}

#[allow(dead_code)]
impl<S: State> Sketch<S> {
    /// Initializes a Sketch
    pub fn from_size(width: usize, height: usize, state: S) -> Sketch<S> {
        let window = minifb::Window::new(DEFAULT_NAME, width, height, minifb::WindowOptions::default())
            .unwrap_or_else(|e| {
                panic!("Unable to open window: {}", e);
            });

        let pixels: Vec<u32> = vec![0u32; width*height];

        let mut sketch = Sketch {
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
            shape_vertices: Vec::new(),
            shape_holes: Vec::new(),
            shape_type: ShapeType::Polygon,

            draw_method: None,
            setup_method: None,
            mouse_pressed_method: None,
            mouse_released_method: None,
            key_pressed_method: None,
            key_released_method: None,

            loaded_fonts: Vec::new(),
            font_index: 0,

            state,
        };

        println!("LOADING FONT");

        let tnr_file_path = "fonts/Times New Roman.ttf";
        let times_new_roman = sketch.open_ttf_file(tnr_file_path);

        let arial_file_path = "fonts/Arial.ttf";
        let arial = sketch.open_ttf_file(arial_file_path);

        sketch.loaded_fonts.push((times_new_roman, tnr_file_path.to_string()));
        sketch.loaded_fonts.push((arial, arial_file_path.to_string()));

        println!("FONT DONE");

        sketch
    }

    // Private Methods

    /// INTERNAL : interface between Sketch and minifb for mouse interactions
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

    /// INTERNAL : interface between Sketch and minifb for keyboard interactions
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

    /// main loop of the Sketch
    pub fn run(&mut self) {
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

    fn open_ttf_file(&self, file_path: &str) -> Font {
        let mut file_content = Vec::new();
        fs::File::open(file_path).unwrap().read_to_end(&mut file_content).unwrap();

        Font::from_bytes(file_content, FontSettings::default()).unwrap()
    }

    /// Generates a mask based on current stroke mode
    fn generate_mask(&self) -> Vec<(i8, i8)> {
        match self.stroke_mode {
            StrokeMode::Circle => self.generate_circular_mask(),
            StrokeMode::Square => self.generate_square_mask(),
            StrokeMode::Custom(mask_func) => mask_func(self.stroke_weight),
        }
    }

    /// Pastes mask on Sketch at certain x,y coordinates
    fn apply_mask_as_stroke(&mut self, x: i32, y: i32, mask: &Vec<(i8, i8)>) {
        for (i, j) in mask {
            let (xi, yj) = (x + *i as i32, y + *j as i32);
            self.stroke_pixel(xi, yj);
        }
    }

    /// Traces a line using the low variant of the Bresenham algorithm
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

    /// Traces a line using the high variant of the Bresenham algorithm
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

    /// Traces a line integrating both low and high variants of the Bresenham algorithm
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

    /// Applies mask along traced line
    fn bresenham_plot_line_mask(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, mask: Vec<(i8, i8)>) {
        let points_to_plot = self.bresenham_plot_line(x0, y0, x1, y1);

        for point in points_to_plot {
            self.apply_mask_as_stroke(point.0, point.1, &mask);
        }
    }

    /// Plots a point on all octants of a space around xc,yc origin
    fn plot_on_all_octants(points: &mut Vec<(i32, i32)>, xc: i32, yc: i32, x: i32, y: i32) {
        points.push((xc + x, yc + y));
        points.push((xc + y, yc + x));

        points.push((xc - x, yc + y));
        points.push((xc - y, yc + x));

        points.push((xc + x, yc - y));
        points.push((xc + y, yc - x));

        points.push((xc - x, yc - y));
        points.push((xc - y, yc - x));
    }

    /// Traces a circle using the Bresenham algorithm (Midpoint algorithm)
    fn bresenham_plot_circle(&mut self, xc: i32, yc: i32, r: i32) -> Vec<(i32, i32)> {
        let mut points: Vec<(i32, i32)> = vec![];

        let (mut x, mut y) = (0, r);
        let mut d = 3 - 2 * r;

        while y >= x {
            Self::plot_on_all_octants(&mut points, xc, yc, x, y);
            if d > 0 {
                y -= 1;
                d += 4 * (x - y) + 10;
            }
            else {
                d += 4 * x + 6;
            }
            x += 1;
        }
        points
    }

    /// Applies stroke mask along traced circle
    fn circle_stroke(&mut self, xc: i32, yc: i32, r: i32) {
        let mask = self.generate_mask();
        let circle = self.bresenham_plot_circle(xc, yc, r);
        for (x, y) in circle {
            self.apply_mask_as_stroke(x, y, &mask)
        }
    }

    /// Fills a circular region of the sketch using a brute-force algorithm
    fn circle_fill(&mut self, xc: i32, yc: i32, r: i32) {
        for xi in -r..=r {
            for yi in -r..=r {
                if xi*xi + yi*yi > r*r {continue;}
                let (x, y) = (xc + xi, yc + yi);

                self.fill_pixel(x, y);
            }
        }
    }

    /// Generates a circular mask with radius = stroke_weight
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

    /// Generates a square mask with side_length = 2 * stroke_weight
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

    /// Fills a flat triangle situated above its base
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
                self.change_pixel(x, y, self.fill_color.unwrap());
            }
        }
    }

    /// Fills a flat triangle situated below its base
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
                self.change_pixel(x, y, self.fill_color.unwrap());
            }
        }
    }

    /// Fills a triangle by separating into top and bottom parts
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

    /// Strokes the 3 sides of a triangle
    fn triangle_stroke(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32) {
        self.line(x0, y0, x1, y1);
        self.line(x0, y0, x2, y2);
        self.line(x2, y2, x1, y1);
    }

    fn change_pixel(&mut self, x: i32, y: i32, color: u32) {
        if x < 0 ||y < 0 ||x >= self.width as i32 || y >= self.height as i32 {return;}

        let (x, y) = (x as u32, y as u32);

        if RgbaColor::color_alpha(color) == 255 {
            self.set_pixel(x, y, color);
        }
        else {
            self.mix_pixel(x, y, color);
        }
    }

    /// Changes the color of the pixel at x,y
    fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        let index = x as usize + y as usize * self.width;
        self.pixels[index] = color;
    }

    fn mix_pixel(&mut self, x: u32, y: u32, color: u32) {
        let index = x as usize + y as usize * self.width;
        let new_color = RgbaColor::color_alpha_compose_color(self.pixels[index], color);
        self.pixels[index] = new_color;
    }

    /// Fills the inside of a rectangle at x,y with side lengths w,h
    fn rect_fill(&mut self, x: i32, y: i32, w: i32, h: i32) {
        for i in x..(x+w) {
            for j in y..(y+h) {
                self.fill_pixel(i, j);
            }
        }
    }

    /// Strokes the 4 sides of a rectangle at x,y with side lengths w,h
    fn rect_stroke(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.line(x, y, x+w, y);
        self.line(x, y, x, y+h);
        self.line(x, y+h, x+w, y+h);
        self.line(x+w, y, x+w, y+h);
    }

    /// Triangulates and fills current constructed polygon
    fn polygon_fill(&mut self) {
        let mut coords: Vec<f64> = Vec::new();

        for point in &self.shape_vertices {
            coords.push(point.0 as f64);
            coords.push(point.1 as f64);
        }

        let triangles = earcutr::earcut(&coords, &self.shape_holes, 2).unwrap_or_else(|e| panic!("Triangulation error: {e}"));

        println!("triangles :");

        for i in 0..(triangles.len() / 3) {
            let (a, b, c) = (triangles[3*i], triangles[3*i+1], triangles[3*i+2]);
            self.triangle_fill(
                self.shape_vertices[a].0, self.shape_vertices[a].1,
                self.shape_vertices[b].0, self.shape_vertices[b].1,
                self.shape_vertices[c].0, self.shape_vertices[c].1
            )
        }
    }

    /// Strokes all edges of the current constructed polygon
    fn polygon_stroke(&mut self) {
        self.linear_spline(true);
    }

    /// Draws a polygon based on current shape construction
    fn polygon(&mut self) {
        if self.fill_color.is_some() {
            self.polygon_fill();
        }
        if self.stroke_color.is_some() {
            self.polygon_stroke();
        }
    }

    /// Draws a linear spline based on the current shape construction, holes separate different chains
    fn linear_spline(&mut self, loops: bool) {
        let mut start = 0usize;
        let mut hole = 0usize;
        for i in 0..self.shape_vertices.len() {
            if !loops && (i == self.shape_vertices.len() - 1 || (hole < self.shape_holes.len() && i == self.shape_holes[hole] - 1)) {
                continue;
            }

            let (x0, y0) = self.shape_vertices[i];
            let (x1, y1) = self.shape_vertices[
                if i == self.shape_vertices.len() - 1 || (hole < self.shape_holes.len() && i == self.shape_holes[hole] - 1) {start}
                else {i + 1}
            ];

            self.line(x0, y0, x1, y1);

            if hole < self.shape_holes.len() && i == self.shape_holes[hole] {
                hole += 1;
                start = i;
            }
        }
    }

    // Public Methods

    /// Changes the name of the window
    pub fn name(&mut self, name: &str) {
        self.window.set_title(name);
    }

    /// Checks if the key: Key is currently pressed
    pub fn key_is_down(&self, key: Key) -> bool{
        self.window.is_key_down(key)
    }

    /// Stops the animation loop
    pub fn no_loop(&mut self) {
        self.is_looping = false;
    }

    /// Sets the framerate limit of the window
    pub fn framerate(&mut self, fps: usize) {
        self.window.set_target_fps(fps);
    }

    /// Sets the current fill color
    pub fn fill(&mut self, color: u32) {
        self.fill_color = Some(color);
    }

    /// Removes current fill color, drawn shapes will be hollow
    pub fn no_fill(&mut self) {
        self.fill_color = None;
    }

    /// Sets the current stroke color
    pub fn stroke(&mut self, color: u32) {
        self.stroke_color = Some(color);
    }

    /// Removes the current stroke color, drawn shapes will not have an outline
    pub fn no_stroke(&mut self) {
        self.stroke_color = None;
    }

    /// Sets the thickness of the outline
    pub fn stroke_weight(&mut self, weight: i8) {
        self.stroke_weight = weight;
    }

    /// Changes the current stroke mode, see StrokeMode
    pub fn stroke_mode(&mut self, mode: StrokeMode) {
        self.stroke_mode = mode;
    }

    /// Applies current fill color to pixel at x,y
    pub fn fill_pixel(&mut self, x: i32, y: i32) {
         if let Some(color) = self.fill_color {
             self.change_pixel(x, y, color);
         }
    }

    /// Applies current stroke color to pixel at x,y
    pub fn stroke_pixel(&mut self, x: i32, y: i32) {
        if let Some(color) = self.stroke_color {
            self.change_pixel(x, y, color);
        }
    }

    /// Draws a rectangle at x,y with side lengths w,h
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        if self.fill_color.is_some() {
            self.rect_fill(x, y, w, h);
        }
        if self.stroke_color.is_some() {
            self.rect_stroke(x, y, w, h);
        }
    }

    /// Draws a triangle between points x0,y0 x1,y1 and x2,y2
    pub fn triangle(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32) {
        if self.fill_color.is_some() {
            self.triangle_fill(x0, y0, x1, y1, x2, y2);
        }
        if self.stroke_color.is_some() {
            self.triangle_stroke(x0, y0, x1, y1, x2, y2);
        }
    }

    /// Draws a circle at x,y with radius r
    pub fn circle(&mut self, x: i32, y: i32, r: i32) {
        if self.fill_color.is_some() {
            self.circle_fill(x, y, r);
        }
        if self.stroke_color.is_some() {
            self.circle_stroke(x, y, r);
        }
    }

    /// Fills the window with given color
    pub fn background(&mut self, color: u32) {
        let new_frame: Vec<u32> = vec![color;self.width*self.height];
        self.pixels = new_frame;
    }

    /// Saves a png screenshot of the window
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

    pub fn image(&mut self, image_buffer: ImageBuffer<Rgb<u8>, Vec<u8>>, x: i32, y: i32) {
        for i in 0..image_buffer.width() {
            for j in 0..image_buffer.height() {
                let (px, py) = (x + i as i32, y + j as i32);
                if px < 0 || py < 0 || px as usize >= self.width || py as usize >= self.height {continue;}

                let (r, g, b, a) = image_buffer.get_pixel(i, j).channels4();
                self.change_pixel(px, py, RgbaColor::argb_color(a, r, g, b));
            }
        }
    }

    /// Draws a line between points x0,y0 and x1,y1
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        let mask = self.generate_mask();
        self.bresenham_plot_line_mask(x0, y0, x1, y1, mask);
    }

    /// Indicates the start of a shape construction
    pub fn begin_shape(&mut self, shape_type: ShapeType) {
        self.shape_type = shape_type;
        self.shape_vertices.clear();
        self.shape_holes.clear();
    }

    /// Add a vertex to current shape construction
    pub fn vertex(&mut self, x: i32, y: i32) {
        self.shape_vertices.push((x, y));
    }

    /// Indicate start of a hole within the current shape construction
    pub fn begin_hole(&mut self) {
        self.shape_holes.push(self.shape_vertices.len());
    }

    /// Indicate the end of the current shape construction and render constructed shape
    pub fn end_shape(&mut self) {
        match self.shape_type {
            ShapeType::Polygon => {
                self.polygon();
            }
            ShapeType::LinearSpline {loops} => {
                self.linear_spline(loops);
            }
            ShapeType::CubicBezierSpline {loops} => {
                todo!()
            }
        }
    }

    pub fn font(&mut self, font: FontMode) {
        match font {
            FontMode::TimesNewRoman => {
                self.font_index = 0;
            }
            FontMode::Arial => {
                self.font_index = 1;
            }
            FontMode::Custom { file_path } => {
                let mut i = 0usize;
                let fonts = &self.loaded_fonts;
                for (_, fp) in fonts {
                    if *fp == file_path {
                        self.font_index = i;
                        break;
                    }
                    i += 1;
                }

                let new_font = self.open_ttf_file(file_path.as_str());
                self.loaded_fonts.push((new_font, file_path));
                self.font_index = self.loaded_fonts.len() - 1;
            }
        }
    }

    fn render_char(&mut self, metrics: Metrics, pixels: Vec<u8>, x_start: i32, y_start : i32) {
        for i in 0..metrics.width {
            for j in 0..metrics.height {
                let index = j*metrics.width + i;
                let (px, py) = (x_start + metrics.xmin + i as i32, y_start + j as i32 - metrics.height as i32 - metrics.ymin);
                if px < 0 || py < 0 || px as usize >= self.width || py as usize >= self.height {continue;}

                let color_to_mix = RgbaColor::argb_color(
                    pixels[index],
                    RgbaColor::color_red(self.fill_color.unwrap()),
                    RgbaColor::color_green(self.fill_color.unwrap()),
                    RgbaColor::color_blue(self.fill_color.unwrap()),
                );

                self.mix_pixel(px as u32, py as u32, color_to_mix);
            }
        }
    }

    pub fn text(&mut self, string: &str, x: i32, y: i32) {
        let scale = 32.0;
        let mut x_start = x;
        let mut y_start = y;

        for char in string.chars() {
            let (metrics, pixels) = {
                let mut font = &mut self.loaded_fonts[self.font_index].0;
                font.rasterize(char, scale)
            };

            self.render_char(metrics, pixels, x_start, y_start);

            x_start += metrics.advance_width as i32;
            y_start += metrics.advance_height as i32;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MyState {
        transition: Option<Transition>,
    }

    impl State for MyState {}

    fn setup(sketch: &mut Sketch<MyState>) {
        println!("SETUP WAS CALLED");
        sketch.framerate(60);
        sketch.name("Example Sketch");

        let mut start: Vec<(i32, i32)> = Vec::new();
        start.push((50, 50));
        start.push((100, 50));
        start.push((100, 100));
        start.push((150, 100));

        let mut end: Vec<(i32, i32)> = Vec::new();
        end.push((450, 200));
        end.push((500, 200));
        end.push((500, 250));
        end.push((450, 250));

        let transition = Transition::initialize(
            EasingType::SmoothStep,
            3f32,
            TransitionTarget::Points {points: start},
            TransitionTarget::Points {points: end},
        );

        sketch.state.transition = Some(transition);

        sketch.stroke(RgbaColor::greyscale_color(255));
        sketch.stroke_weight(2);
        sketch.stroke_mode(StrokeMode::Circle);
    }

    fn draw(sketch: &mut Sketch<MyState>) {
        if sketch.frame_count == 0 {
            println!("FIRST DRAW CALL");
        }

        sketch.background(RgbaColor::greyscale_color(50));


        if let Some(mut transition) = sketch.state.transition.take() {
            sketch.begin_shape(ShapeType::LinearSpline {loops: false});
            let points = transition.get_current_points();
            for &(x, y) in points {
                sketch.vertex(x, y);
            }
            sketch.end_shape();

            if sketch.frame_count > 60 && !transition.is_finished() {
                transition.step(sketch.delta_time);
            }
            sketch.state.transition = Some(transition);
        }


        // sketch.fill(RgbaColor::argb_color(255, 255, 20, 20));
        // sketch.rect(100, 100, 200, 50);
        //
        // sketch.fill(RgbaColor::greyscale_color(255));
        //
        // sketch.font(FontMode::TimesNewRoman);
        // let fps =  ((1f32 / sketch.delta_time * 100f32) as u32) as f32 / 100f32;
        // sketch.text(format!("FPS : {}", fps).as_str(), 50, 50);



        // sketch.stroke(RgbaColor::greyscale_color(255));
        // sketch.stroke_weight(3);
        // sketch.fill(0);
        //
        // sketch.begin_shape(ShapeType::Polygon);
        // {
        //     for i in 0..10 {
        //         let angle: f32 = std::f32::consts::PI / 5f32  * i as f32;
        //         let (x, y) = (320f32 + 200f32 * angle.cos(), 240f32 + 200f32 * angle.sin());
        //         sketch.vertex(x as i32, y as i32);
        //     }
        //     sketch.begin_hole();
        //     for i in 0..4 {
        //         let angle: f32 = std::f32::consts::PI / 2f32  * i as f32;
        //         let (x, y) = (320f32 + 100f32 * angle.cos(), 240f32 + 100f32 * angle.sin());
        //         sketch.vertex(x as i32, y as i32);
        //     }
        // }
        // sketch.end_shape();
    }

    #[test]
    fn testing() {
        println!("START");
        let mut state = MyState::default();
        let mut sketch = Sketch::<MyState>::from_size(640, 480, state);

        sketch.setup_method = Some(setup);
        sketch.draw_method = Some(draw);

        sketch.run();

        println!("TESTING DONE")
    }
}
