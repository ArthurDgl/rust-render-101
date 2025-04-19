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

pub struct Geometry {}

impl Geometry {
    /// Creates a random f32 value between lower and upper bounds
    pub fn random(lower: f32, upper: f32) -> f32 {
        lower + rand::random::<f32>() * (upper - lower)
    }
}