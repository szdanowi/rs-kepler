extern crate cairo;
extern crate gtk;
extern crate gio;
extern crate rand;

use gio::prelude::*;
use gtk::prelude::*;
use std::env::args;

fn paint(drawing_area: &gtk::DrawingArea, context: &cairo::Context) -> gtk::Inhibit {
    let max_x = drawing_area.get_allocated_width();
    let max_y = drawing_area.get_allocated_height();

    context.set_source_rgb(0.7, 0.7, 0.7);
    context.paint();

    context.set_source_rgb(0.1, 0.1, 0.1);
    for _line in 0..10 {
        let x = rand::random::<f64>() * f64::from(max_x);
        let y = rand::random::<f64>() * f64::from(max_y);
        context.line_to(x, y);
    }
    context.stroke();

    Inhibit(false)
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);
    let drawing_area = Box::new(gtk::DrawingArea::new)();

    drawing_area.connect_draw(paint);

    window.set_title("rs-kepler");
    window.set_border_width(0);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1024, 768);
    window.add(&drawing_area);

    window.show_all();
}

fn main() {
    let application = gtk::Application::new(Some("com.rs-kepler"), Default::default())
        .expect("Failed to initialize GTK application");

    application.connect_activate(|app|{ build_ui(app); });
    application.run(&args().collect::<Vec<_>>());
}
