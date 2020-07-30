extern crate cairo;
extern crate gtk;
extern crate gio;
extern crate rand;

use gio::prelude::*;
use gtk::prelude::*;
use std::env::args;
use std::f64::consts::PI;
use std::rc::Rc;
use std::cell::RefCell;

struct Coordinate {
    x: f64,
    y: f64,
}

struct EuclideanVector {
    dx: f64,
    dy: f64,
}

impl EuclideanVector {
    fn magnitude(&self) -> f64 {
        (self.dx*self.dx + self.dy*self.dy).sqrt()
    }
}

impl std::cmp::PartialEq<f64> for EuclideanVector {
    fn eq(&self, other: &f64) -> bool {
        self.magnitude() == *other
    }
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
    pub fn with(mut self, body: Body) -> Self { self.add(body); self }
    pub fn add(&mut self, body: Body) { self.bodies.push(body); }
}

// ---

trait CairoPaintable {
    fn paint_on(&self, context: &cairo::Context);
}

impl CairoPaintable for EuclideanVector {
    fn paint_on(&self, context: &cairo::Context) {
        if self.magnitude() == 0. { return; }

        context.set_source_rgb(0., 0., 1.);
        context.move_to(0., 0.);
        context.line_to(10. * self.dx, 10. * self.dy);
        context.stroke();
    }
}

impl CairoPaintable for Body {
    fn paint_on(&self, context: &cairo::Context) {
        context.save();
        context.translate(self.position.x, self.position.y);
        context.set_source_rgb(1., 1., 1.);
        context.arc(0., 0., self.mass, 0., PI*2.);
        context.stroke();

        self.velocity.paint_on(context);
        context.restore();
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

fn build_ui(application: &gtk::Application, model: Rc<RefCell<Situation>>) {
    let window = gtk::ApplicationWindow::new(application);
    let drawing_area = gtk::DrawingArea::new();

    drawing_area.connect_draw(move |drawing_area, cairo_context|{
        paint(drawing_area, cairo_context, &model.borrow())
    });

    window.set_title("rs-kepler");
    window.set_border_width(0);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1024, 768);
    window.add(&drawing_area);

    window.show_all();
}

fn main() {
    let model = Rc::new(RefCell::new(Situation::new().with(
        Body::new().with_mass(10.).at(Coordinate{x: 0., y: 0.}).moving(EuclideanVector{dx: 0., dy: 0.})
    ).with(
        Body::new().with_mass(2.).at(Coordinate{x: 100., y: 0.}).moving(EuclideanVector{dx: 0., dy: 1.})
    )));

    let application = gtk::Application::new(Some("com.rs-kepler"), Default::default())
        .expect("Failed to initialize GTK application");

    let captured_model = Rc::clone(&model);
    application.connect_activate(move |app| {
        build_ui(app, Rc::clone(&captured_model));
    });

    model.borrow_mut().add(Body::new().with_mass(20.).at(Coordinate{x: 200., y: 10.}).moving(EuclideanVector{dx: 0., dy: -1.}));
    application.run(&args().collect::<Vec<_>>());
}
