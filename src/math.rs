use crate::types::field::coord::Coord;

const R2: f64 = 6371.0 * 2.0;

pub fn great_circle(coord1: Coord, coord2: Coord) -> f64 {
    let delta_lat2 = (coord2.lat - coord1.lat) * 0.5;
    let delta_lon2 = (coord2.lon - coord1.lon) * 0.5;

    let a =
        delta_lat2.sin().powi(2) + delta_lon2.sin().powi(2) * coord1.lat.cos() * coord2.lat.cos();
    let c = a.sqrt().atan2((1.0 - a).sqrt());

    c * R2
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

    use crate::types::field::coord::{
        Latitude, LatitudeHemisphere, Longitude, LongitudeHemisphere,
    };

    use super::*;

    fn assert_symmetry_eq(coord1: Coord, coord2: Coord, distance: f64) {
        assert_eq!(great_circle(coord1, coord2), distance);
        assert_eq!(great_circle(coord2, coord1), distance);
    }

    #[test]
    fn great_circle_test_quarter() {
        assert_symmetry_eq(
            Coord { lat: 0.0, lon: 0.0 },
            Coord {
                lat: 0.0,
                lon: FRAC_PI_2,
            },
            FRAC_PI_4 * R2,
        );
        assert_symmetry_eq(
            Coord {
                lat: FRAC_PI_2,
                lon: 0.0,
            },
            Coord { lat: 0.0, lon: 0.0 },
            FRAC_PI_4 * R2,
        );
    }

    #[test]
    fn great_circle_test() {
        let coord1 = (
            &Latitude {
                degrees: 50,
                minutes: 3,
                seconds: 59,
                fractional_seconds: 0,
                hemisphere: LatitudeHemisphere::North,
            },
            &Longitude {
                degrees: 5,
                minutes: 42,
                seconds: 53,
                fractional_seconds: 0,
                hemisphere: LongitudeHemisphere::West,
            },
        );
        let coord2 = (
            &Latitude {
                degrees: 58,
                minutes: 38,
                seconds: 38,
                fractional_seconds: 0,
                hemisphere: LatitudeHemisphere::North,
            },
            &Longitude {
                degrees: 3,
                minutes: 4,
                seconds: 12,
                fractional_seconds: 0,
                hemisphere: LongitudeHemisphere::West,
            },
        );

        let distance = great_circle(coord1.into(), coord2.into());

        assert!(
            (968.85..=968.94).contains(&distance),
            "Distance: {distance} not in {:?}",
            968.85..=968.94
        );
    }
}
