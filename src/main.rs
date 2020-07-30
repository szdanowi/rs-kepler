extern crate cairo;
extern crate gtk;
extern crate gio;
extern crate glib;
extern crate rand;
extern crate chrono;

use gio::prelude::*;
use gtk::prelude::*;
use chrono::prelude::*;
use std::env::args;
use std::f64::consts::PI;
use std::rc::Rc;
use std::cell::RefCell;
use std::ptr;

const GRAVITATIONAL_CONSTANT: f64 = 100.;
const VECTOR_MAGNIFICATION: f64 = 25.;

#[derive(Copy, Clone)]
struct Coordinate {
    x: f64,
    y: f64,
}

#[derive(Copy, Clone)]
struct EuclideanVector {
    dx: f64,
    dy: f64,
}

impl EuclideanVector {
    fn between(from: Coordinate, to: Coordinate) -> Self {
        Self{dx: to.x - from.x, dy: to.y - from.y}
    }

    fn magnitude(&self) -> f64 {
        (self.dx*self.dx + self.dy*self.dy).sqrt()
    }

    fn versor(&self) -> Self {
        let len = self.magnitude();
        Self{dx: self.dx / len, dy: self.dy / len}
    }
}

impl std::ops::AddAssign<EuclideanVector> for Coordinate {
    fn add_assign(&mut self, delta: EuclideanVector) {
        self.x += delta.dx;
        self.y += delta.dy;
    }
}

impl std::ops::Mul<f64> for EuclideanVector {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Self{dx: self.dx * scalar, dy: self.dy * scalar}
    }
}

impl std::cmp::PartialEq<f64> for EuclideanVector {
    fn eq(&self, other: &f64) -> bool {
        self.magnitude() == *other
    }
}

struct Force {
    anchor: Coordinate,
    vector: EuclideanVector,
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

    pub fn update(&mut self) {
        self.position += self.velocity;
    }

    pub fn pull_from(&self, other: &Self) -> Force {
        let joining_vector = EuclideanVector::between(self.position, other.position);
        let distance = joining_vector.magnitude();
        Force{
            anchor: self.position,
            vector: joining_vector.versor() * ((self.mass * other.mass) / (distance * distance)) * GRAVITATIONAL_CONSTANT,
        }
    }
}

impl std::cmp::PartialEq for Body {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

struct Situation {
    bodies: Vec<Body>,
    forces: Vec<Force>,
}

impl Situation {
    pub fn new() -> Situation { Situation { bodies: Vec::<Body>::new(), forces: Vec::<Force>::new() } }
    pub fn with(mut self, body: Body) -> Self { self.add(body); self }
    pub fn add(&mut self, body: Body) { self.bodies.push(body); }

    pub fn update(&mut self) {
        for body in self.bodies.iter_mut() {
            body.update();
        }
        self.forces.clear();
        for body in self.bodies.iter() {
            for other_body in self.bodies.iter() {
                if body != other_body {
                    self.forces.push(body.pull_from(other_body));
                }
            }
        }
    }
}

// ---

trait CairoPaintable {
    fn paint_on(&self, context: &cairo::Context);
}

impl CairoPaintable for EuclideanVector {
    fn paint_on(&self, context: &cairo::Context) {
        if self.magnitude() == 0. { return; }

        context.move_to(0., 0.);
        context.line_to(VECTOR_MAGNIFICATION * self.dx, VECTOR_MAGNIFICATION * self.dy);
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

        context.set_source_rgb(0., 0., 1.);
        self.velocity.paint_on(context);

        context.restore();
    }
}

impl CairoPaintable for Force {
    fn paint_on(&self, context: &cairo::Context) {
        context.save();

        context.translate(self.anchor.x, self.anchor.y);
        context.set_source_rgb(1., 0., 0.);
        self.vector.paint_on(context);

        context.restore();
    }
}

fn print_text(context: &cairo::Context, x: f64, y:f64, text: String) {
    context.move_to(x, y);
    context.show_text(&text);
}

fn print_debug(context: &cairo::Context, situation: &Situation) {
    context.set_source_rgb(1., 1., 1.);
    print_text(context, 10., 15., format!("{}", Local::now().format("%Y-%m-%d %H:%M:%S")));
    print_text(context, 10., 25., format!("bodies: {}", situation.bodies.len()));
    print_text(context, 10., 35., format!("forces: {}", situation.forces.len()));
}

fn paint(drawing_area: &gtk::DrawingArea, context: &cairo::Context, situation: &Situation) -> gtk::Inhibit {
    let max_x = f64::from(drawing_area.get_allocated_width());
    let max_y = f64::from(drawing_area.get_allocated_height());

    context.set_source_rgb(0.05, 0.05, 0.05);
    context.paint();

    context.save();
    context.translate(max_x / 2., max_y / 2.);
    for body in situation.bodies.iter() { body.paint_on(context); }
    for force in situation.forces.iter() { force.paint_on(context); }
    context.restore();

    print_debug(context, situation);
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

    gtk::timeout_add(100, move || { drawing_area.queue_draw(); glib::Continue(true) });
}

fn main() {
    let model = Rc::new(RefCell::new(Situation::new().with(
        Body::new().with_mass(10.).at(Coordinate{x: 0., y: 0.}).moving(EuclideanVector{dx: 0., dy: 0.})
    ).with(
        Body::new().with_mass(2.).at(Coordinate{x: 100., y: 0.}).moving(EuclideanVector{dx: 0., dy: 1.})
    ).with(
        Body::new().with_mass(20.).at(Coordinate{x: 200., y: 10.}).moving(EuclideanVector{dx: 0., dy: -1.})
    )));

    let application = gtk::Application::new(Some("com.rs-kepler"), Default::default())
        .expect("Failed to initialize GTK application");

    let activation_captured_model = Rc::clone(&model);
    application.connect_activate(move |app| {
        build_ui(app, Rc::clone(&activation_captured_model));

        let timeout_captured_model = activation_captured_model.clone();
        gtk::timeout_add(100, move || {
            timeout_captured_model.borrow_mut().update();
            glib::Continue(true)
        });
    });

    application.run(&args().collect::<Vec<_>>());
}
