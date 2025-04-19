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

    /// Converts a u32 color to a tuple of 4 f32's between 0 and 1
    fn color_u32_to_4xf32(color: u32) -> (f32, f32, f32, f32) {
        (
            RgbaColor::color_alpha(color) as f32 / 255f32,
            RgbaColor::color_red(color) as f32 / 255f32,
            RgbaColor::color_green(color) as f32 / 255f32,
            RgbaColor::color_blue(color) as f32 / 255f32,
        )
    }

    /// Converts a tuple of 4 f32's to a u32 color
    fn color_4xf32_to_u32(color: (f32, f32, f32, f32)) -> u32 {
        RgbaColor::argb_color(
            (color.0 * 255f32) as u8,
            (color.1 * 255f32) as u8,
            (color.2 * 255f32) as u8,
            (color.3 * 255f32) as u8,
        )
    }

    /// Performs the alpha compose operation in the alpha channels
    fn alpha_compose_alpha(p_a: f32, q_a: f32) -> f32 {
        p_a + q_a - p_a * q_a
    }

    /// Performs the alpha compose operation on the color channels
    fn alpha_compose_channel(p_a: f32, p_c: f32, q_a: f32, q_c: f32, r_a: f32) -> f32 {
        (p_c * p_a + q_c * q_a - p_c * p_a * q_a) / r_a
    }

    /// Entire pipeline of composing 2 u32 colors using alpha compose operation
    pub fn color_alpha_compose_color(color_p: u32, color_q: u32) -> u32 {
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
