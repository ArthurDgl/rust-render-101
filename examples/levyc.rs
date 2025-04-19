use rust_render_101::*;


struct MyState {
    transition: Option<transition::Transition>,
    count: u32,
}

impl State for MyState {}

impl Default for MyState {
    fn default() -> Self {
        MyState{
            transition: None,
            count: 0,
        }
    }
}

fn setup(sketch: &mut Sketch<MyState>) {
    let (p1, p2) = ((sketch.width as i32 * 7 / 24, sketch.height as i32 * 3 / 5),
        (sketch.width as i32 * 17 / 24, sketch.height as i32 * 3 / 5));

    let mut points: Vec<(i32, i32)> = Vec::new();
    points.push(p1);
    points.push(p2);

    sketch.state.transition = Some(transition::Transition::initialize(
        transition::EasingType::QuadOut,
        3.0,
        transition::TransitionTarget::Points {points: points.clone()},
        transition::TransitionTarget::Points {points},
    ));

    sketch.stroke(color::RgbaColor::greyscale_color(255));
    sketch.stroke_weight(1);
}

fn next_iteration(transition: &mut transition::Transition) {
    let previous = transition.get_end_points();

    let mut next_start = previous.clone();
    let mut next_end = previous.clone();

    for k in 2..=previous.len() {
        let i = previous.len() - k;

        let p1 = previous[i];
        let p2 = previous[i + 1];

        let mid = ((p1.0 + p2.0)/2, (p1.1 + p2.1)/2);
        next_start.insert(i+1, mid);

        next_end.insert(i+1, next_point(p1, p2));
    }

    let next_start = transition::TransitionTarget::Points {points: next_start};
    let next_end = transition::TransitionTarget::Points {points: next_end};

    transition.reset_new(next_start, next_end);
}

fn next_point(p1: (i32, i32), p2: (i32, i32)) -> (i32, i32) {
    let push_vec = ((p2.1 - p1.1)/2, (p1.0 - p2.0)/2);

    ((p1.0 + p2.0)/2 + push_vec.0, (p1.1 + p2.1)/2 + push_vec.1)
}

fn draw(sketch: &mut Sketch<MyState>) {
    sketch.background(color::RgbaColor::greyscale_color(50));

    if let Some(mut transition) = sketch.state.transition.take() {
        sketch.begin_shape(geometry::ShapeType::LinearSpline {loops: false});
        let points = transition.get_current_points();
        for &(x, y) in points {
            sketch.vertex(x, y);
        }
        sketch.end_shape();

        transition.step(sketch.delta_time);

        if transition.is_finished() {
            sketch.state.count += 1;
        }
        else {
            sketch.state.count = 0;
        }

        if sketch.state.count >= 120 {
            next_iteration(&mut transition);
        }

        sketch.state.transition = Some(transition);
    }
}

fn main() {
    let state = MyState::default();
    let mut sketch = Sketch::from_size(600, 600, state);

    sketch.draw_method = Some(draw);
    sketch.setup_method = Some(setup);

    sketch.run();
}