use image::{ImageBuffer, Rgb};
use minifb::{Key, KeyRepeat, MouseButton, MouseMode};
use rand;

const DEFAULT_NAME: &str = "Rust Render 101 Sketch";

pub enum StrokeMode{
    Circle,
    Square,
    Custom(fn(i8) -> Vec<(i8, i8)>),
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
        ((g as u32) << 16) | ((g as u32) << 8) | g as u32
    }

    /// Creates rgba value based on 3 rgb u8 values
    pub fn rgb_color(r: u8, g: u8, b: u8) -> u32 {
        ((r as u32) << 16) | ((g as u32) << 8) | b as u32
    }

    /// Creates rgba value based on 4 u8 values
    pub fn rgba_color(a: u8, r: u8, g: u8, b: u8) -> u32 {
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
}

pub struct Geometry {}

impl Geometry {
    /// Creates a random f32 value between lower and upper bounds
    pub fn random(lower: f32, upper: f32) -> f32 {
        lower + rand::random::<f32>() * (upper - lower)
    }

    fn triangulate_monotone(points: &Vec<(i32, i32)>, main_chain: &mut Vec<usize>, right_chain: &mut Vec<usize>) -> Vec<(usize, usize, usize)> {
        if main_chain.len() + right_chain.len() == 3 {
            main_chain.append(right_chain);
            return vec![(main_chain[0], main_chain[1], main_chain[2])];
        }

        let start_point = points[main_chain[0]];
        let mut result: Vec<(usize, usize, usize)> = Vec::new();
        let mut best_triangle: Option<(usize, usize, usize)> = None;
        let mut best_lowest_y: i32 = points[main_chain[main_chain.len() - 1]].1;
        let mut remove_from_main: Option<usize> = None;
        let mut remove_from_right: Option<usize> = None;

        let mut evaluate_triangle = |triangle: (usize, usize, usize), main_idx: Option<usize>, right_idx: Option<usize>| {
            let lowest_y = points[triangle.0].1.max(points[triangle.1].1).max(points[triangle.2].1);
            if lowest_y < best_lowest_y {
                best_triangle = Some(triangle);
                best_lowest_y = lowest_y;
                remove_from_main = main_idx;
                remove_from_right = right_idx;
            }
        };

        if main_chain.len() > 1 && !right_chain.is_empty() {
            evaluate_triangle((main_chain[0], main_chain[1], right_chain[0]), Some(0), None);
        }
        if main_chain.len() > 2 && Self::is_left_triangle_valid(start_point, points[main_chain[1]], points[main_chain[2]]) {
            evaluate_triangle((main_chain[0], main_chain[1], main_chain[2]), Some(1), None);
        }
        if right_chain.len() > 1 && Self::is_right_triangle_valid(start_point, points[right_chain[0]], points[right_chain[1]]) {
            evaluate_triangle((main_chain[0], right_chain[0], right_chain[1]), None, Some(0));
        }

        if let Some(idx) = remove_from_main {
            main_chain.remove(idx);
        }
        if let Some(idx) = remove_from_right {
            right_chain.remove(idx);
        }

        if let Some(triangle) = best_triangle {
            result.push(triangle);
        }
        result.extend(Self::triangulate_monotone(points, main_chain, right_chain));

        result
    }

    fn is_left_triangle_valid(p0: (i32, i32), p1: (i32, i32), p2: (i32, i32)) -> bool {
        let perp = (p1.1 - p0.1, p0.0 - p1.0);
        let p1p2 = (p2.0 - p1.0, p2.1 - p1.1);
        0 < perp.0 * p1p2.0 + perp.1 * p1p2.1
    }

    fn is_right_triangle_valid(p0: (i32, i32), p1: (i32, i32), p2: (i32, i32)) -> bool {
        let perp = (p0.1 - p1.1, p1.0 - p0.0);
        let p1p2 = (p2.0 - p1.0, p2.1 - p1.1);
        0 < perp.0 * p1p2.0 + perp.1 * p1p2.1
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

#[allow(dead_code)]
impl<S: State> Sketch<S> {
    /// Initializes a Sketch
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

    /// INTERNAL : main loop of the Sketch
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
            if xi >= 0 && yj >= 0 {
                self.stroke_pixel(xi as u32, yj as u32);
            }
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
                if x >= 0 && y >= 0 {
                    self.fill_pixel(x as u32, y as u32);
                }
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
                if x >= 0 && y >= 0 {
                    self.set_pixel(x as u32, y as u32, self.fill_color.expect(""));
                }
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
                if x >= 0 && y >= 0 {
                    self.set_pixel(x as u32, y as u32, self.fill_color.expect(""));
                }
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

    /// Changes the color of the pixel at x,y
    fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x >= self.width as u32 || y >= self.height as u32 {
            return;
        }
        let index = x as usize + y as usize * self.width;
        self.pixels[index] = color;
    }

    /// Fills the inside of a rectangle at x,y with side lengths w,h
    fn rect_fill(&mut self, x: u32, y: u32, w: u32, h: u32) {
        for i in x..(x+w) {
            for j in y..(y+h) {
                self.fill_pixel(i, j);
            }
        }
    }

    /// Strokes the 4 sides of a rectangle at x,y with side lengths w,h
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
    pub fn fill_pixel(&mut self, x: u32, y: u32) {
         if let Some(color) = self.fill_color {
             self.set_pixel(x, y, color);
         }
    }

    /// Applies current stroke color to pixel at x,y
    pub fn stroke_pixel(&mut self, x: u32, y: u32) {
        if let Some(color) = self.stroke_color {
            self.set_pixel(x, y, color);
        }
    }

    /// Draws a rectangle at x,y with side lengths w,h
    pub fn rect(&mut self, x: u32, y: u32, w: u32, h: u32) {
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
        for i in 0..self.width as u32 {
            for j in 0..self.height as u32 {
                self.set_pixel(i, j, color);
            }
        }
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

    /// Draws a line between points x0,y0 and x1,y1
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        let mask = self.generate_mask();
        self.bresenham_plot_line_mask(x0, y0, x1, y1, mask);
    }

    fn monotone_polygon(&mut self, points: &Vec<(i32, i32)>, main_chain: &mut Vec<usize>, right_chain: &mut Vec<usize>) {
        let triangles = Geometry::triangulate_monotone(points, main_chain, right_chain);
        for (i, j, k) in triangles {
            let ((x0, y0), (x1, y1), (x2, y2)) = (points[i], points[j], points[k]);
            self.triangle(x0, y0, x1, y1, x2, y2);
        }
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

        let green: u32 = RgbaColor::rgb_color(50, 200, 50);
        let gray: u32 = RgbaColor::rgb_color(50, 50, 50);

        let points: Vec<(i32, i32)> = vec![(180, 30), (60, 120), (240, 150), (30, 270), (210, 360), (360, 210), (480, 240), (180, 300)];
        let main_chain: Vec<usize> = vec![0, 1, 2, 3, 4];
        let right_chain: Vec<usize> = vec![5, 6, 7];

        sketch.background(gray);
        sketch.no_stroke();
        sketch.fill(green);
        //sketch.stroke_weight(2);
        sketch.monotone_polygon(&points, &mut main_chain.clone(), &mut right_chain.clone());

        sketch.no_loop();

        // sketch.background(sketch.state.background_color);
        //
        // sketch.fill(green);
        // sketch.stroke(gray);
        // sketch.stroke_weight(2);
        // sketch.stroke_mode(StrokeMode::Square);
        // sketch.rect(50, 100, 200, 100);
        //
        // sketch.fill(gray);
        // sketch.stroke(green);
        // sketch.stroke_weight(3);
        // sketch.stroke_mode(StrokeMode::Circle);
        // sketch.triangle(350, 50, 450, 150, 300, 300);
        //
        // sketch.stroke(RgbaColor::rgb_color(255, 50, 255));
        // sketch.stroke_weight(5);
        // sketch.stroke_mode(StrokeMode::Circle);
        // sketch.line(sketch.state.line_x1, sketch.state.line_y1, sketch.state.line_x2, sketch.state.line_y2);
        //
        // sketch.stroke(RgbaColor::rgb_color(200, 50, 50));
        // sketch.stroke_weight(3);
        // sketch.circle(sketch.mouse_x as i32, sketch.mouse_y as i32, 15);
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
