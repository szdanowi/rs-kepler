extern crate cairo;
extern crate rand;

use cairo::{ ImageSurface, Format, Context };
use std::fs::File;

fn main() {
    let max_x = 1024;
    let max_y = 768;

    let surface = ImageSurface::create(Format::ARgb32, max_x, max_y).expect("Failed to create a surface");
    let context = Context::new(&surface);

    context.set_source_rgb(1.0, 1.0, 1.0);
    context.paint();

    context.set_source_rgb(0.1, 0.1, 0.1);
    for _line in 0..10 {
        let x = rand::random::<f64>() * f64::from(max_x);
        let y = rand::random::<f64>() * f64::from(max_y);
        context.line_to(x, y);
    }
    context.stroke();

    let mut file = File::create("result.png").expect("Failed to create a file");
    surface.write_to_png(&mut file).expect("Failed to write to the file");
}
