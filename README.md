# rust-render-101

Rust Render 101 is a barebones vector graphics renderer.
It operates using a "p5js-like" loop and features.

Examples are provided in the examples folder.

Most features are contained in the Sketch struct but some are accessible through the geometry, color and transition modules.


External Crates used :

minifb v"0.27.0"
-- main loop, window and pixels

image v"0.25.4"
-- image handling

rand v"0.9.0-alpha.2"
-- random numbers

earcutr v"0.4.3"
-- triangulating polygons (O(n²) time complexity using the ear cut algorithm)

fontdue v"0.9.2"
-- font rasterization
