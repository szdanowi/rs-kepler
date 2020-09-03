use crate::maths::{Coordinate, EuclideanVector};
use core::f64::consts::PI;

pub const GRAVITATIONAL_CONSTANT: f64 = 10.;

pub struct Body {
    pub name: String,
    pub position: Coordinate,
    pub mass: f64,
    pub radius: f64,
    pub velocity: EuclideanVector,
    pub forces: Vec<EuclideanVector>,
    pub highlighted: bool,
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
