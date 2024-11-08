# rust-render-101
Example Code :
```rust

#[derive(Default)]
struct MyState {
    line_x1: i32,
    line_y1: i32,
    line_x2: i32,
    line_y2: i32,
}

impl State for MyState {}

fn setup(sketch: &mut Sketch<MyState>) {
    println!("SETUP WAS CALLED");
    sketch.framerate(60);
}

fn draw(sketch: &mut Sketch<MyState>) {
    if sketch.frame_count == 0 {
        println!("FIRST DRAW CALL");
    }

    let green: u32 = RgbaColor::rgb_color(50, 255, 50);
    let blue: u32 = RgbaColor::rgb_color(50, 50, 255);
    let gray: u32 = RgbaColor::rgb_color(50, 50, 50);

    sketch.background(blue);

    sketch.fill(green);
    sketch.stroke(gray);
    sketch.stroke_weight(2);
    sketch.stroke_mode(StrokeMode::Square);
    sketch.rect(50, 100, 200, 100);

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
        sketch.background(RgbaColor::rgb_color(0, 0, 0));
    } else if key == Key::S {
        sketch.save("screenshot.png");
    }
}

fn main() {
    let mut state = MyState::default();
    let mut sketch = Sketch::<MyState>::from_size(640, 480, state);

    sketch.setup_method = Some(setup);
    sketch.draw_method = Some(draw);

    sketch.mouse_pressed_method = Some(mouse_pressed);
    sketch.key_pressed_method = Some(key_pressed);

    sketch.run();
}
```
Result :
(Line position can be changed by clicking, each mouse button changes a different end of the line)


![image](https://github.com/user-attachments/assets/4f1c1b1c-8194-441c-a4ad-7d9996980f9e)
