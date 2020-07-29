extern crate cairo;
extern crate gtk;
extern crate gio;
extern crate rand;

use gio::prelude::*;
use gtk::prelude::*;
use std::env::args;
use std::f64::consts::PI;

struct Coordinate {
    x: f64,
    y: f64,
}

struct EuclideanVector {
    dx: f64,
    dy: f64,
}

struct Body {
    position: Coordinate,
    mass: f64,
    velocity: EuclideanVector,
}

impl Body {
    pub fn new() -> Body { Body{position: Coordinate{x: 0., y:0.}, mass: 0., velocity: EuclideanVector{dx: 0., dy: 0.}} }
    pub fn at(mut self, arg: Coordinate) -> Self { self.position = arg; self }
    pub fn moving(mut self, arg: EuclideanVector) -> Self { self.velocity = arg; self }
    pub fn with_mass(mut self, arg: f64) -> Self { self.mass = arg; self }
}

struct Situation {
    bodies: Vec<Body>,
}

impl Situation {
    pub fn new() -> Situation { Situation { bodies: Vec::<Body>::new() } }
    pub fn with(mut self, body: Body) -> Self { self.bodies.push(body); self }
}

// ---

trait CairoPaintable {
    fn paint_on(&self, context: &cairo::Context);
}

impl CairoPaintable for Body {
    fn paint_on(&self, context: &cairo::Context) {
        context.set_source_rgb(1., 1., 1.);
        context.arc(self.position.x, self.position.y, self.mass, 0., PI*2.);
        context.stroke();

        context.set_source_rgb(0., 0., 1.);
        context.move_to(self.position.x, self.position.y);
        context.line_to(self.position.x + (10. * self.velocity.dx), self.position.y + (10. * self.velocity.dy));
        context.stroke();
    }
}

fn paint(drawing_area: &gtk::DrawingArea, context: &cairo::Context, situation: &Situation) -> gtk::Inhibit {
    let max_x = f64::from(drawing_area.get_allocated_width());
    let max_y = f64::from(drawing_area.get_allocated_height());

    context.set_source_rgb(0.05, 0.05, 0.05);
    context.paint();

    context.translate(max_x / 2., max_y / 2.);
    for body in situation.bodies.iter() { body.paint_on(context); }

    Inhibit(false)
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);
    let drawing_area = gtk::DrawingArea::new();
    let model = Situation::new().with(
        Body::new().with_mass(10.).at(Coordinate{x: 0., y: 0.}).moving(EuclideanVector{dx: 0., dy: 0.})
    ).with(
        Body::new().with_mass(2.).at(Coordinate{x: 100., y: 0.}).moving(EuclideanVector{dx: 0., dy: 1.})
    );

    drawing_area.connect_draw(move |drawing_area, cairo_context|{
        paint(drawing_area, cairo_context, &model)
    });

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
