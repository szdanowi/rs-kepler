use chrono::prelude::*;
use derive_more::{Add, AddAssign, Div, Mul};
use gdk::keys;
use gio::prelude::*;
use gtk::prelude::*;
use std::cell::RefCell;
use std::env::args;
use std::f64::consts::PI;
use std::rc::Rc;

const GRAVITATIONAL_CONSTANT: f64 = 10.;
const VECTOR_MAGNIFICATION: f64 = 25.;
const REFRESH_RATE: u32 = 50; // per second
const UPDATE_RATE: u32 = 50; // per second
const TRAIL_HISTORY: u32 = 2000;
const SCROLL_STEP: f64 = 25.;

#[derive(Copy, Clone)]
struct Coordinate {
    x: f64,
    y: f64,
}

#[derive(Copy, Clone, AddAssign, Div, Mul, Add)]
struct EuclideanVector {
    dx: f64,
    dy: f64,
}

impl EuclideanVector {
    fn between(from: Coordinate, to: Coordinate) -> Self {
        Self { dx: to.x - from.x, dy: to.y - from.y }
    }

    fn magnitude(&self) -> f64 {
        self.dx.hypot(self.dy)
    }

    fn versor(&self) -> Self {
        let len = self.magnitude();
        Self { dx: self.dx / len, dy: self.dy / len }
    }
}

impl std::ops::AddAssign<EuclideanVector> for Coordinate {
    fn add_assign(&mut self, delta: EuclideanVector) {
        self.x += delta.dx;
        self.y += delta.dy;
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
    radius: f64,
    velocity: EuclideanVector,
    forces: Vec<EuclideanVector>,
}

impl Body {
    const DENSITY: f64 = 3.;

    pub const fn new() -> Self {
        Self {
            position: Coordinate { x: 0., y: 0. },
            mass: 0.,
            radius: 0.,
            velocity: EuclideanVector { dx: 0., dy: 0. },
            forces: Vec::<EuclideanVector>::new(),
        }
    }
    pub const fn at(mut self, arg: Coordinate) -> Self {
        self.position = arg;
        self
    }
    pub const fn moving(mut self, arg: EuclideanVector) -> Self {
        self.velocity = arg;
        self
    }
    pub fn with_mass(mut self, arg: f64) -> Self {
        self.mass = arg;
        let volume = self.mass / Self::DENSITY;
        self.radius = ((3. / (4. * PI)) * volume).powf(0.33);
        self
    }

    pub fn update(&mut self) {
        self.position += self.velocity;

        for force in &self.forces {
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
        self == other
    }
}

struct Mark {
    position: Coordinate,
    age: u32,
}

impl Mark {
    const fn new(at: Coordinate) -> Self {
        Self { position: at, age: 0 }
    }
    fn update(&mut self) {
        self.age += 1;
    }
}

struct Situation {
    bodies: Vec<Body>,
    marks: Vec<Mark>,
    updates: u64,
    zoom: f64,
    fullscreen: bool,
    paused: bool,
    translation: EuclideanVector,
}

impl Situation {
    pub const fn new() -> Self {
        Self {
            bodies: Vec::<Body>::new(),
            marks: Vec::<Mark>::new(),
            updates: 0,
            zoom: 1.,
            fullscreen: false,
            paused: false,
            translation: EuclideanVector { dx: 0., dy: 0. },
        }
    }
    pub fn with(mut self, body: Body) -> Self {
        self.add(body);
        self
    }
    pub fn add(&mut self, body: Body) {
        self.bodies.push(body);
    }

    pub fn update(&mut self) {
        if self.paused { return; }

        for i in 0..self.bodies.len() {
            let (head, tail) = self.bodies.split_at_mut(i);
            let (body, tail) = tail.split_at_mut(1);
            let body = &mut body[0];

            body.update();
            body.forces.clear();

            for other_body in head.iter_mut().chain(tail) {
                body.add_pull_from(other_body);
            }

            if self.updates % (u64::from(REFRESH_RATE) / 10) == 0 {
                self.marks.push(Mark::new(body.position));
            }
        }

        for mark in &mut self.marks {
            mark.update();
        }
        self.marks.retain(|mark| mark.age < TRAIL_HISTORY);
        self.updates += 1;
    }

    pub fn count_forces(&self) -> usize {
        let mut result = 0;
        for body in &self.bodies { result += body.forces.len(); }
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
        context.arc(0., 0., self.radius, 0., PI * 2.);
        context.stroke();

        context.set_source_rgb(0., 0., 1.);
        self.velocity.paint_on(context);

        context.set_source_rgb(1., 0., 0.);
        for force in &self.forces { force.paint_on(context); }

        context.restore();
    }
}

impl CairoPaintable for Mark {
    fn paint_on(&self, context: &cairo::Context) {
        context.save();
        context.translate(self.position.x, self.position.y);

        let brightness = 0.7 * f64::max(0.05, f64::from(TRAIL_HISTORY - self.age) / f64::from(TRAIL_HISTORY));
        context.set_source_rgb(brightness, brightness, brightness);
        context.arc(0., 0., 1., 0., PI * 2.);
        context.fill();

        context.restore();
    }
}

fn print_text(context: &cairo::Context, x: f64, y: f64, text: &str) {
    context.move_to(x, y);
    context.show_text(text);
}

fn print_debug(context: &cairo::Context, situation: &Situation) {
    context.set_source_rgb(1., 1., 1.);
    print_text(context, 10., 15., &format!("{}", Local::now().format("%Y-%m-%d %H:%M:%S")));
    print_text(context, 10., 25., &format!("bodies: {}", situation.bodies.len()));
    print_text(context, 10., 35., &format!("forces: {}", situation.count_forces()));
    print_text(context, 10., 45., &format!("iteration: {}", situation.updates));
    print_text(context, 10., 55., &format!("zoom: {}", situation.zoom));
    if situation.fullscreen { print_text(context, 10., 75., "Fullscreen"); }
    if situation.paused { print_text(context, 10., 85., "Paused"); }
}

fn paint(drawing_area: &gtk::DrawingArea, context: &cairo::Context, situation: &Situation) -> gtk::Inhibit {
    let translation = EuclideanVector {
        dx: f64::from(drawing_area.get_allocated_width()) / 2.,
        dy: f64::from(drawing_area.get_allocated_height()) / 2.,
    } + situation.translation;

    context.set_source_rgb(0.05, 0.05, 0.05);
    context.paint();

    context.save();
    context.translate(translation.dx, translation.dy);
    context.scale(situation.zoom, situation.zoom);
    for body in &situation.bodies { body.paint_on(context); }
    for mark in &situation.marks { mark.paint_on(context); }
    context.restore();

    print_debug(context, situation);
    Inhibit(false)
}

fn toggle_fullscreen(window: &gtk::ApplicationWindow, model: &mut Situation) {
    if model.fullscreen {
        window.unfullscreen();
    } else {
        window.fullscreen();
    }
    model.fullscreen = !model.fullscreen;
}

fn build_ui(application: &gtk::Application, model: Rc<RefCell<Situation>>) {
    let window = gtk::ApplicationWindow::new(application);
    let drawing_area = gtk::DrawingArea::new();

    let draw_captured_model = Rc::clone(&model);
    drawing_area.connect_draw(move |drawing_area, cairo_context| {
        paint(drawing_area, cairo_context, &draw_captured_model.borrow())
    });

    window.set_title("rs-kepler");
    window.set_border_width(0);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1024, 768);
    window.add(&drawing_area);

    window.connect_key_press_event(move |window, gdk| {
        let mut mut_model = model.borrow_mut();
        match gdk.get_keyval() {
            keys::constants::F11 => toggle_fullscreen(window, &mut mut_model),
            keys::constants::plus => mut_model.zoom *= 1.1,
            keys::constants::minus => mut_model.zoom /= 1.1,
            keys::constants::_0 => mut_model.zoom = 1.,
            keys::constants::space => mut_model.paused = !mut_model.paused,
            keys::constants::Left => mut_model.translation.dx += SCROLL_STEP,
            keys::constants::Right => mut_model.translation.dx -= SCROLL_STEP,
            keys::constants::Up => mut_model.translation.dy += SCROLL_STEP,
            keys::constants::Down => mut_model.translation.dy -= SCROLL_STEP,
            _ => (),
        }
        Inhibit(false)
    });

    window.show_all();

    gtk::timeout_add(1000 / REFRESH_RATE, move || {
        drawing_area.queue_draw();
        glib::Continue(true)
    });
}

fn main() {
    let model = Rc::new(RefCell::new(Situation::new().with(
        Body::new().with_mass(70.).at(Coordinate{x: 0., y: 0.}).moving(EuclideanVector{dx: 0., dy: 0.})
    ).with(
        Body::new().with_mass(1.).at(Coordinate{x: 150., y: 0.}).moving(EuclideanVector{dx: 0., dy: 2.})
    ).with(
        Body::new().with_mass(1.).at(Coordinate{x: -400., y: 0.}).moving(EuclideanVector{dx: 0., dy: 1.})
    ).with(
        Body::new().with_mass(0.1).at(Coordinate{x: 0., y: -300.}).moving(EuclideanVector{dx: 0.9, dy: 0.})
    )));

    let application = gtk::Application::new(Some("com.rs-kepler"), gio::ApplicationFlags::default())
        .expect("Failed to initialize GTK application");

    let activation_captured_model = Rc::clone(&model);
    application.connect_activate(move |app| {
        build_ui(app, Rc::clone(&activation_captured_model));

        let timeout_captured_model = activation_captured_model.clone();
        gtk::timeout_add(1000 / UPDATE_RATE, move || {
            timeout_captured_model.borrow_mut().update();
            glib::Continue(true)
        });
    });

    application.run(&args().collect::<Vec<_>>());
}
