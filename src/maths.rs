use derive_more::{Add, AddAssign, Div, Mul, Sub};

#[derive(Copy, Clone)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

impl Coordinate {
    pub const fn from(tuple: (f64, f64)) -> Self {
        Self { x: tuple.0, y: tuple.1 }
    }
}

#[derive(Copy, Clone, AddAssign, Div, Mul, Add, Sub)]
pub struct EuclideanVector {
    pub dx: f64,
    pub dy: f64,
}

impl EuclideanVector {
    pub fn between(from: Coordinate, to: Coordinate) -> Self {
        Self { dx: to.x - from.x, dy: to.y - from.y }
    }

    pub fn magnitude(&self) -> f64 {
        self.dx.hypot(self.dy)
    }

    pub fn versor(&self) -> Self {
        let len = self.magnitude();
        Self { dx: self.dx / len, dy: self.dy / len }
    }

    pub fn towards(to: Coordinate) -> EuclideanVector {
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

