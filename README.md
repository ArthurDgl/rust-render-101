# rust-render-101
Example Code :
```rust
// IMPORTS

impl Runnable for Sketch {
    fn setup(&mut self) {
        println!("SETUP WAS CALLED");
        self.framerate(60);
    }

    fn draw(&mut self) {
        if self.frame_count == 0 {
            println!("FIRST DRAW CALL");
        }

        let green: u32 = Self::rgb_color(50, 255, 50);
        let blue: u32 = Self::rgb_color(50, 50, 255);
        let gray: u32 = Self::rgb_color(50, 50, 50);

        self.background(blue);

        self.fill(green);
        self.stroke(gray);
        self.stroke_weight(2);
        self.stroke_mode(StrokeMode::Square);
        self.rect(50, 100, 200, 100);

        self.stroke(Self::rgb_color(255, 50, 255));
        self.stroke_weight(5);
        self.stroke_mode(StrokeMode::Circle);
        self.line(300, 400, self.mouse_x as i32, self.mouse_y as i32);
    }

    fn mouse_pressed(&mut self) {
        let red: u32 = Self::rgb_color(255, 50, 50);

        self.rect(self.mouse_x as u32, self.mouse_y as u32, 10, 10);
    }

    fn key_pressed(&mut self, key: Key) {
        if key == Key::Space {
            self.background(Self::rgb_color(0, 0, 0));
        }
        else if key == Key::S {
            self.save("screenshot.png");
        }
    }
}

fn main() {
    Sketch::from_size(640, 480).run();
}
```
Result :


![image](https://github.com/user-attachments/assets/4f1c1b1c-8194-441c-a4ad-7d9996980f9e)
