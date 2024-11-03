# rust-render-101
An expansion library for minifb in rust. Includes primitive shapes and other features in a p5js style.

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

        let blue: u32 = Self::rbg_color(50, 50, 255);
        let green: u32 = Self::rbg_color(50, 255, 50);

        self.background(blue);

        self.fill(green);
        self.rect(50, 100, 200, 100);
    }
}

fn main() {
    Sketch::from_size(640, 480).run();
}
```