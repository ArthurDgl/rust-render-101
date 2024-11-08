# rust-render-101
Example Code :
```rust

fn setup(sketch: &mut Sketch) {
    println!("SETUP WAS CALLED");
    sketch.framerate(60);
}

fn draw(sketch: &mut Sketch) {
    if sketch.frame_count == 0 {
        println!("FIRST DRAW CALL");
    }

    let green: u32 = Sketch::rgb_color(50, 255, 50);
    let blue: u32 = Sketch::rgb_color(50, 50, 255);
    let gray: u32 = Sketch::rgb_color(50, 50, 50);

    sketch.background(blue);

    sketch.fill(green);
    sketch.stroke(gray);
    sketch.stroke_weight(2);
    sketch.stroke_mode(StrokeMode::Square);
    sketch.rect(50, 100, 200, 100);

    sketch.stroke(Sketch::rgb_color(255, 50, 255));
    sketch.stroke_weight(5);
    sketch.stroke_mode(StrokeMode::Circle);
    sketch.line(300, 400, sketch.mouse_x as i32, sketch.mouse_y as i32);
}

fn mouse_pressed(sketch: &mut Sketch) {
    let red: u32 = Sketch::rgb_color(255, 50, 50);

    sketch.rect(sketch.mouse_x as u32, sketch.mouse_y as u32, 10, 10);
}

fn key_pressed(sketch: &mut Sketch, key: Key) {
    if key == Key::Space {
        sketch.background(Sketch::rgb_color(0, 0, 0));
    }
    else if key == Key::S {
        sketch.save("screenshot.png");
    }
}

fn main() {
    let mut sketch = Sketch::from_size(640, 480);

    sketch.setup_method = setup;
    sketch.draw_method = draw;

    sketch.mouse_pressed_method = mouse_pressed;
    sketch.key_pressed_method = key_pressed;

    sketch.run();
}
```
Result :


![image](https://github.com/user-attachments/assets/4f1c1b1c-8194-441c-a4ad-7d9996980f9e)
