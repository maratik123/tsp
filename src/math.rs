use crate::types::field::coord::Coord;

const R2: f64 = 6371.0 * 2.0;

pub fn great_circle(coord1: &Coord, coord2: &Coord) -> f64 {
    let delta_lat2 = (coord2.lat - coord1.lat) * 0.5;
    let delta_lon2 = (coord2.lon - coord1.lon) * 0.5;

    let sin_lat2 = delta_lat2.sin();
    let sin_lon2 = delta_lon2.sin();

    let a = sin_lat2 * sin_lat2 + sin_lon2 * sin_lon2 * coord1.lat.cos() * coord2.lat.cos();
    let c = a.sqrt().atan2((1.0 - a).sqrt());

    c * R2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::field::coord::{
        Latitude, LatitudeHemisphere, Longitude, LongitudeHemisphere,
    };

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

        let distance = great_circle(&coord1.into(), &coord2.into());

        assert!(
            (968.85..=968.94).contains(&distance),
            "Distance: {distance} not in {:?}",
            968.85..=968.94
        );
    }
}
