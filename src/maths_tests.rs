#[cfg(test)]
mod tests {
    use crate::maths::EuclideanVector;
    type EV = EuclideanVector;

    const VECTOR1: EV = EV { dx: 4.4, dy: 7.7 };
    const VECTOR_EQUAL_TO_VECTOR1: EV = VECTOR1;
    const VECTOR_WITH_DIFFERENT_DX: EV = EV {
        dx: VECTOR1.dx + 0.1,
        dy: VECTOR1.dy,
    };
    const VECTOR_WITH_DIFFERENT_DY: EV = EV {
        dx: VECTOR1.dx,
        dy: VECTOR1.dy + 0.1,
    };

    #[test]
    fn euclidean_vector_is_equal_to_self() {
        assert_eq!(VECTOR1, VECTOR1);
    }

    #[test]
    fn euclidean_vector_is_equal_to_vector_with_same_values() {
        assert_eq!(VECTOR1, VECTOR_EQUAL_TO_VECTOR1);
    }

    #[test]
    fn euclidean_vector_is_not_equal_to_vector_with_different_dx() {
        assert_ne!(VECTOR1, VECTOR_WITH_DIFFERENT_DX);
    }

    #[test]
    fn euclidean_vector_is_not_equal_to_vector_with_different_dy() {
        assert_ne!(VECTOR1, VECTOR_WITH_DIFFERENT_DY);
    }

    const VECTOR_WITH_LENGTH_1: EV = EV { dx: 1., dy: 0. };
    const VECTOR_WITH_LENGTH_5: EV = EV { dx: 4., dy: 3. };

    #[test]
    fn euclidean_vector_is_comparable_to_its_length() {
        assert_eq!(VECTOR_WITH_LENGTH_1, 1.);
        assert_ne!(VECTOR_WITH_LENGTH_1, 2.);
        assert_eq!(VECTOR_WITH_LENGTH_5, 5.);
        assert_ne!(VECTOR_WITH_LENGTH_5, 1.);
    }
}
