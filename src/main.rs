use chrono::prelude::*;
use derive_more::{Add, AddAssign, Div, Mul, Sub};
use gdk::{keys, ScrollDirection, WindowExt};
use gio::prelude::*;
use gtk::prelude::*;
use std::env::args;
use std::f64::consts::PI;
use cairo::{Region, RectangleInt};

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

impl Coordinate {
    const fn from(tuple: (f64, f64)) -> Self {
        Self { x: tuple.0, y: tuple.1 }
    }
}

#[derive(Copy, Clone, AddAssign, Div, Mul, Add, Sub)]
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

    fn towards(to: Coordinate) -> EuclideanVector {
        Self { dx: to.x, dy: to.y }
    }
}

impl std::ops::Neg for EuclideanVector {
    type Output = EuclideanVector;

    fn neg(self) -> Self {
        Self { dx: -self.dx, dy: -self.dy }
    }
}
impl std::fmt::Display for EuclideanVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "({:.4}, {:.4})", self.dx, self.dy)
    }
}

impl std::ops::AddAssign<EuclideanVector> for Coordinate {
    fn add_assign(&mut self, delta: EuclideanVector) {
        self.x += delta.dx;
        self.y += delta.dy;
    }
}

impl std::ops::Sub for Coordinate {
    type Output = EuclideanVector;

    fn sub(self, other: Self) -> EuclideanVector {
        EuclideanVector { dx: self.x - other.x, dy: self.y - other.y }
    }
}

impl std::cmp::PartialEq<f64> for EuclideanVector {
    fn eq(&self, other: &f64) -> bool {
        self.magnitude() == *other
    }
}

struct Body {
    name: String,
    position: Coordinate,
    mass: f64,
    radius: f64,
    velocity: EuclideanVector,
    forces: Vec<EuclideanVector>,
    highlighted: bool,
}

impl Body {
    const DENSITY: f64 = 3.;

    pub const fn new() -> Self {
        Self {
            name: String::new(),
            position: Coordinate { x: 0., y: 0. },
            mass: 0.,
            radius: 0.,
            velocity: EuclideanVector { dx: 0., dy: 0. },
            forces: Vec::<EuclideanVector>::new(),
            highlighted: false,
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
    pub fn named(mut self, arg: &str) -> Self {
        self.name = arg.to_string();
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
    zoom_exponent: f64,
    fullscreen: bool,
    paused: bool,
    translation: EuclideanVector,
    drag_start: Coordinate,
    tracked_body: Option<usize>,
}

impl Situation {
    pub const fn new() -> Self {
        Self {
            bodies: Vec::<Body>::new(),
            marks: Vec::<Mark>::new(),
            updates: 0,
            zoom_exponent: 0.,
            fullscreen: false,
            paused: false,
            translation: EuclideanVector { dx: 0., dy: 0. },
            drag_start: Coordinate { x: 0., y: 0. },
            tracked_body: None,
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

            body.highlighted = self.tracked_body == Some(i);
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
    pub fn zoom_in(&mut self) {
        self.zoom_exponent += 0.25;
    }
    pub fn zoom_out(&mut self) {
        self.zoom_exponent -= 0.25;
    }
    pub fn zoom_reset(&mut self) {
        self.zoom_exponent = 0.;
    }
    pub fn zoom(&self) -> f64 {
        2.0_f64.powf(self.zoom_exponent)
    }
    pub fn track_next(&mut self) {
        match self.tracked_body {
            Some(tracked) => if self.bodies.len() > tracked + 1 { self.tracked_body = Some(tracked + 1); } else { self.tracked_body = None; },
            None => if !self.bodies.is_empty() { self.tracked_body = Some(0); },
        }
    }
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused
    }
    pub fn drag_started(&mut self, window_position: Coordinate) {
        self.drag_start = window_position;
    }
    pub fn dragging_to(&mut self, window_position: Coordinate) {
        let delta = (window_position - self.drag_start) / self.zoom();
        self.translation += delta;
        self.drag_start = window_position;
    }
    pub fn center_translation(&self) -> EuclideanVector {
        match self.tracked_body {
            Some(tracked) => -EuclideanVector::towards(self.bodies[tracked].position),
            None => self.translation,
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
        context.arc(0., 0., self.radius, 0., PI * 2.);
        context.stroke();

        context.move_to(7., 10.);
        if self.highlighted { context.set_source_rgb(1., 1., 0.); }
        context.show_text(&self.name);
        context.move_to(0., 0.);

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
    print_text(context, 10., 55., &format!("zoom: {}", situation.zoom_exponent));
    print_text(context, 10., 65., &format!("center: {}", -situation.center_translation()));
    if situation.fullscreen { print_text(context, 10., 85., "Fullscreen"); }
    if situation.paused { print_text(context, 10., 95., "Paused"); }
}

fn viewport_translation(viewport: &gtk::DrawingArea) -> EuclideanVector {
    EuclideanVector {
        dx: f64::from(viewport.get_allocated_width()) / 2.,
        dy: f64::from(viewport.get_allocated_height()) / 2.,
    }
}

fn paint(drawing_area: &gtk::DrawingArea, context: &cairo::Context, situation: &Situation) {
    context.set_source_rgb(0.05, 0.05, 0.05);
    context.paint();
    context.save();

    let viewport_translation = viewport_translation(drawing_area);
    context.translate(viewport_translation.dx, viewport_translation.dy);

    let scale = situation.zoom();
    context.scale(scale, scale);

    let translation = situation.center_translation();
    context.translate(translation.dx, translation.dy);

    for body in &situation.bodies { body.paint_on(context); }
    for mark in &situation.marks { mark.paint_on(context); }
    context.restore();

    print_debug(context, situation);
}

fn toggle_fullscreen(window: &gtk::ApplicationWindow, model: &mut Situation) {
    if model.fullscreen {
        window.unfullscreen();
    } else {
        window.fullscreen();
    }
    model.fullscreen = !model.fullscreen;
}

const DEFAULT_CONTEXT: Option<&glib::MainContext> = None;

enum Event {
    UpdateModel,
    Draw,
    KeyPressed(gdk::keys::Key),
    Scrolling(gdk::ScrollDirection),
    MousePressed(Coordinate),
    MouseDragged(Coordinate),
}

macro_rules! with_clone_of {
    ($object: ident, $expression: expr) => {{
        let $object = $object.clone();
        $expression
    }};
}

fn build_ui(application: &gtk::Application, mut model: Situation) {
    let drawing_area = gtk::DrawingArea::new();
    drawing_area.add_events(
        gdk::EventMask::BUTTON_PRESS_MASK |
        gdk::EventMask::SCROLL_MASK |
        gdk::EventMask::POINTER_MOTION_MASK);

    let window = gtk::ApplicationWindow::new(application);
    window.set_title("rs-kepler");
    window.set_border_width(0);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1024, 768);
    window.add(&drawing_area);
    window.show_all();

    let (event_sender, event_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    with_clone_of!(event_sender, window.connect_key_press_event(move |_, gdk| {
        event_sender.send(Event::KeyPressed(gdk.get_keyval())).expect("Failed to raise KeyPressed event");
        Inhibit(false)
    }));

    with_clone_of!(event_sender, drawing_area.connect_button_press_event(move |_, gdk| {
        event_sender.send(Event::MousePressed(Coordinate::from(gdk.get_position()))).expect("Failed to raise MousePressed event");
        Inhibit(false)
    }));

    with_clone_of!(event_sender, drawing_area.connect_motion_notify_event(move |_, gdk| {
        if gdk.get_state().contains(gdk::ModifierType::BUTTON1_MASK) {
            event_sender.send(Event::MouseDragged(Coordinate::from(gdk.get_position()))).expect("Failed to raise MouseDragged event");
        }
        Inhibit(false)
    }));

    with_clone_of!(event_sender, drawing_area.connect_scroll_event(move |_, gdk| {
        event_sender.send(Event::Scrolling(gdk.get_direction())).expect("Failed to raise Scrolling event");
        Inhibit(false)
    }));

    with_clone_of!(event_sender, gtk::timeout_add(1000 / UPDATE_RATE, move || {
        event_sender.send(Event::UpdateModel).expect("Failed to raise UpdateModel event");
        glib::Continue(true)
    }));

    with_clone_of!(event_sender, gtk::timeout_add(1000 / REFRESH_RATE, move || {
        event_sender.send(Event::Draw).expect("Failed to raise Draw event");
        glib::Continue(true)
    }));

    event_receiver.attach(DEFAULT_CONTEXT, move |event| {
        match event {
            Event::UpdateModel => model.update(),
            Event::Draw => {
                let area = RectangleInt { x: 0, y: 0, width: window.get_allocated_width(), height: window.get_allocated_height() };
                let region = Region::create_rectangle(&area);
                let window = drawing_area.get_window().unwrap();
                let drawing_context = window.begin_draw_frame(&region).unwrap();
                let context = drawing_context.get_cairo_context().unwrap();
                paint(&drawing_area, &context, &model);
                window.end_draw_frame(&drawing_context);
            },
            Event::KeyPressed(keys::constants::Escape) => window.close(),
            Event::KeyPressed(keys::constants::F12)    => window.close(),
            Event::KeyPressed(keys::constants::F11)    => toggle_fullscreen(&window, &mut model),
            Event::KeyPressed(keys::constants::plus)   => model.zoom_in(),
            Event::KeyPressed(keys::constants::minus)  => model.zoom_out(),
            Event::KeyPressed(keys::constants::_0)     => model.zoom_reset(),
            Event::KeyPressed(keys::constants::space)  => model.toggle_pause(),
            Event::KeyPressed(keys::constants::Left)   => model.translation.dx += SCROLL_STEP,
            Event::KeyPressed(keys::constants::Right)  => model.translation.dx -= SCROLL_STEP,
            Event::KeyPressed(keys::constants::Up)     => model.translation.dy += SCROLL_STEP,
            Event::KeyPressed(keys::constants::Down)   => model.translation.dy -= SCROLL_STEP,
            Event::KeyPressed(keys::constants::Tab)    => model.track_next(),
            Event::Scrolling(ScrollDirection::Down)    => model.zoom_out(),
            Event::Scrolling(ScrollDirection::Up)      => model.zoom_in(),
            Event::MousePressed(coordinate)            => model.drag_started(coordinate),
            Event::MouseDragged(coordinate)            => model.dragging_to(coordinate),
            _ => (),
        };
        glib::Continue(true)
    });
}

fn build_situation() -> Situation {
    Situation::new().with(
        Body::new().with_mass(70.).at(Coordinate{x: 0., y: 0.}).moving(EuclideanVector{dx: 0., dy: 0.}).named("Imagirus*")
    ).with(
        Body::new().with_mass(1.).at(Coordinate{x: 150., y: 0.}).moving(EuclideanVector{dx: 0., dy: 2.}).named("Imagirus I")
    ).with(
        Body::new().with_mass(1.).at(Coordinate{x: -400., y: 0.}).moving(EuclideanVector{dx: 0., dy: 1.}).named("Imagirus II")
    ).with(
        Body::new().with_mass(0.1).at(Coordinate{x: 0., y: -300.}).moving(EuclideanVector{dx: 0.9, dy: 0.}).named("Feather")
    )
}

fn main() {
    let application = gtk::Application::new(Some("com.rs-kepler"), gio::ApplicationFlags::default())
        .expect("Failed to initialize GTK application");

    application.connect_activate(move |app| { build_ui(app, build_situation()); });
    application.run(&args().collect::<Vec<_>>());
}
