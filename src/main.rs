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

const GRAVITATIONAL_CONSTANT: f64 = 10.;
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

impl std::ops::AddAssign for EuclideanVector {
    fn add_assign(&mut self, other: EuclideanVector) {
        self.dx += other.dx;
        self.dy += other.dy;
    }
}

impl std::ops::Mul<f64> for EuclideanVector {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Self{dx: self.dx * scalar, dy: self.dy * scalar}
    }
}

impl std::ops::Div<f64> for EuclideanVector {
    type Output = Self;

    fn div(self, scalar: f64) -> Self {
        Self{dx: self.dx / scalar, dy: self.dy / scalar}
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
    forces: Vec<EuclideanVector>,
}

impl Body {
    pub fn new() -> Body { Body{position: Coordinate{x: 0., y:0.}, mass: 0., velocity: EuclideanVector{dx: 0., dy: 0.}, forces: Vec::<EuclideanVector>::new()} }
    pub fn at(mut self, arg: Coordinate) -> Self { self.position = arg; self }
    pub fn moving(mut self, arg: EuclideanVector) -> Self { self.velocity = arg; self }
    pub fn with_mass(mut self, arg: f64) -> Self { self.mass = arg; self }

    pub fn update(&mut self) {
        self.position += self.velocity;

        for force in self.forces.iter() {
            let acceleration = *force / self.mass;
            self.velocity += acceleration; // * 1 unit of time
        }
    }

    pub fn pull_from(&self, other: &Self) -> EuclideanVector {
        let joining_vector = EuclideanVector::between(self.position, other.position);
        let distance = joining_vector.magnitude();

        joining_vector.versor() * ((self.mass * other.mass) / (distance * distance)) * GRAVITATIONAL_CONSTANT
    }

    pub fn add_pull_from(&mut self, other: &Self) {
        self.forces.push(self.pull_from(other));
    }
}

impl std::cmp::PartialEq for Body {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

struct Situation {
    bodies: Vec<Body>,
}

impl Situation {
    pub fn new() -> Situation { Situation { bodies: Vec::<Body>::new() } }
    pub fn with(mut self, body: Body) -> Self { self.add(body); self }
    pub fn add(&mut self, body: Body) { self.bodies.push(body); }

    pub fn update(&mut self) {
        for i in 0..self.bodies.len() {
            let (head, tail) = self.bodies.split_at_mut(i);
            let (body, tail) = tail.split_at_mut(1);
            let body = &mut body[0];

            body.update();
            body.forces.clear();

            for other_body in head.iter_mut().chain(tail) {
                body.add_pull_from(other_body);
            }
        }
    }

    pub fn count_forces(&self) -> usize {
        let mut result = 0;
        for body in self.bodies.iter() { result += body.forces.len(); }
        result
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

        context.set_source_rgb(1., 0., 0.);
        for force in self.forces.iter() { force.paint_on(context); }

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
    print_text(context, 10., 35., format!("forces: {}", situation.count_forces()));
}

fn paint(drawing_area: &gtk::DrawingArea, context: &cairo::Context, situation: &Situation) -> gtk::Inhibit {
    let max_x = f64::from(drawing_area.get_allocated_width());
    let max_y = f64::from(drawing_area.get_allocated_height());

    context.set_source_rgb(0.05, 0.05, 0.05);
    context.paint();

    context.save();
    context.translate(max_x / 2., max_y / 2.);
    for body in situation.bodies.iter() { body.paint_on(context); }
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
        Body::new().with_mass(70.).at(Coordinate{x: 0., y: 0.}).moving(EuclideanVector{dx: 0., dy: 0.})
    ).with(
        Body::new().with_mass(1.).at(Coordinate{x: 150., y: 0.}).moving(EuclideanVector{dx: 0., dy: 2.})
    ).with(
        Body::new().with_mass(1.).at(Coordinate{x: -400., y: 0.}).moving(EuclideanVector{dx: 0., dy: 1.})
    )));

    let application = gtk::Application::new(Some("com.rs-kepler"), Default::default())
        .expect("Failed to initialize GTK application");

    let activation_captured_model = Rc::clone(&model);
    application.connect_activate(move |app| {
        build_ui(app, Rc::clone(&activation_captured_model));

        let timeout_captured_model = activation_captured_model.clone();
        gtk::timeout_add(20, move || {
            timeout_captured_model.borrow_mut().update();
            glib::Continue(true)
        });
    });

    application.run(&args().collect::<Vec<_>>());
}
