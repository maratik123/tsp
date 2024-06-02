use std::error::Error;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

use crate::model::AirportIdx;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct DistancesIdx<'a> {
    size: usize,
    dists: Vec<f64>,
    _pd: PhantomData<AirportIdx<'a>>,
}

impl<'a> DistancesIdx<'a> {
    pub fn between(&self, apt1: usize, apt2: usize) -> Option<f64> {
        if apt1 >= self.size || apt2 >= self.size {
            return None;
        }
        if apt1 == apt2 {
            return Some(0.0);
        }
        let (apt1, apt2) = if apt1 > apt2 {
            (apt1, apt2)
        } else {
            (apt2, apt1)
        };
        Some(self.dists[apt1 * (apt1 - 1) / 2 + apt2])
    }
}

impl<'a> From<&'a AirportIdx<'a>> for DistancesIdx<'a> {
    fn from(AirportIdx { aps, .. }: &'a AirportIdx) -> Self {
        let size = aps.len();
        let dists = aps
            .iter()
            .enumerate()
            .flat_map(|(apt1_i, apt1)| aps[..apt1_i].iter().map(|apt2| apt1.distance_to(apt2)))
            .collect();
        DistancesIdx {
            size,
            dists,
            _pd: PhantomData,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct KeyNotFound<'a>(&'a str);

impl<'a> Display for KeyNotFound<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Key not found: {}", self.0)
    }
}

impl<'a> Error for KeyNotFound<'a> {}

#[cfg(test)]
mod tests {
    use std::f64::consts::FRAC_PI_2;

    use crate::math::great_circle;
    use crate::model::Airport;
    use crate::types::field::coord::{
        Coord, Latitude, LatitudeHemisphere, Longitude, LongitudeHemisphere,
    };

    use super::*;

    fn airports_template() -> [Airport; 3] {
        [
            Airport {
                icao: "A".to_string(),
                name: "Airport A".to_string(),
                coord: (
                    &Latitude {
                        degrees: 0,
                        minutes: 0,
                        seconds: 0,
                        fractional_seconds: 0,
                        hemisphere: LatitudeHemisphere::North,
                    },
                    &Longitude {
                        degrees: 0,
                        minutes: 0,
                        seconds: 0,
                        fractional_seconds: 0,
                        hemisphere: LongitudeHemisphere::East,
                    },
                )
                    .into(),
            },
            Airport {
                icao: "B".to_string(),
                name: "Airport B".to_string(),
                coord: (
                    &Latitude {
                        degrees: 90,
                        minutes: 0,
                        seconds: 0,
                        fractional_seconds: 0,
                        hemisphere: LatitudeHemisphere::North,
                    },
                    &Longitude {
                        degrees: 0,
                        minutes: 0,
                        seconds: 0,
                        fractional_seconds: 0,
                        hemisphere: LongitudeHemisphere::East,
                    },
                )
                    .into(),
            },
            Airport {
                icao: "C".to_string(),
                name: "Airport C".to_string(),
                coord: (
                    &Latitude {
                        degrees: 0,
                        minutes: 0,
                        seconds: 0,
                        fractional_seconds: 0,
                        hemisphere: LatitudeHemisphere::North,
                    },
                    &Longitude {
                        degrees: 90,
                        minutes: 0,
                        seconds: 0,
                        fractional_seconds: 0,
                        hemisphere: LongitudeHemisphere::East,
                    },
                )
                    .into(),
            },
        ]
    }

    #[test]
    fn idx_between_test() {
        let airports = airports_template();
        let apt_idx = AirportIdx::new(&airports).unwrap();
        let distances_idx = DistancesIdx::from(&apt_idx);
        let quarter = quarter();
        for apt1 in 0..airports.len() {
            for apt2 in 0..airports.len() {
                assert_eq!(
                    distances_idx.between(apt1, apt2),
                    Some(if apt1 == apt2 { 0.0 } else { quarter })
                )
            }
            assert_eq!(distances_idx.between(3, apt1), None);
            assert_eq!(distances_idx.between(apt1, 3), None);
        }
    }

    #[test]
    fn test_distances_idx() {
        let airports = airports_template();
        let apt_idx = AirportIdx::new(&airports).unwrap();
        let distances_idx = DistancesIdx::from(&apt_idx);
        let quarter = quarter();
        assert_eq!(
            distances_idx,
            DistancesIdx {
                size: 3,
                dists: vec![quarter; 3],
                _pd: PhantomData
            }
        );
    }

    fn quarter() -> f64 {
        great_circle(
            Coord {
                lat: 0.0,
                lon: FRAC_PI_2,
            },
            Coord { lat: 0.0, lon: 0.0 },
        )
    }
}
