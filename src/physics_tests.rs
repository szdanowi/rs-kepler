#[cfg(test)]
mod tests {
    use crate::maths::{Coordinate, EuclideanVector};
    use crate::physics::Body;

    #[test]
    fn when_body_with_no_forces_is_updated_its_velocity_does_not_change() {
        let initial_velocity = EuclideanVector { dx: 4.4, dy: 7.7 };
        let mut body = Body::new().with_mass(1.).moving(initial_velocity);
        body.update();
        assert_eq!(body.velocity, initial_velocity);
    }

    #[test]
    fn a_body_may_be_gravitationally_pulled_by_other_body() {
        let mut body = Body::new()
            .at(Coordinate { x: 0.0, y: 0.0 })
            .with_mass(1.);
        let other_body = Body::new()
            .at(Coordinate { x: 10.0, y: 10.0 })
            .with_mass(1.);

        body.add_pull_from(&other_body);
        body.update();

        assert!(body.velocity.dx > 0.);
        assert!(body.velocity.dy > 0.);
    }

    #[test]
    fn a_moving_body_velocity_is_also_affected_by_gravitational_pull() {
        let initial_velocity = EuclideanVector { dx: 1.0, dy: 1.0 };
        let mut body = Body::new()
            .at(Coordinate { x: 0.0, y: 0.0 })
            .with_mass(1.)
            .moving(initial_velocity);
        let other_body = Body::new()
            .at(Coordinate { x: -10.0, y: 10.0 })
            .with_mass(1.);

        body.add_pull_from(&other_body);
        body.update();

        assert!(body.velocity.dx < initial_velocity.dx);
        assert!(body.velocity.dy > initial_velocity.dy);
    }
}
