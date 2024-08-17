use crate::graph::GraphIdx;
use crate::model::AirportIdx;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct DistancesIdx<'a> {
    pub graph: GraphIdx<'a, Option<f64>>,
}

impl<'a> DistancesIdx<'a> {
    pub fn between(&self, apt1: u32, apt2: u32) -> Option<f64> {
        self.graph.between(None, apt1, apt2).flatten()
    }

    pub fn from(
        apt_idx: &'a AirportIdx<'a>,
        min_dist: Option<f64>,
        excepts: &HashMap<&str, HashSet<&str>>,
    ) -> Self {
        Self {
            graph: GraphIdx::new(apt_idx, |apt1, apt2| {
                Some(apt1.distance_to(apt2)).filter(|&dist| {
                    min_dist.map(|min_dist| dist >= min_dist).unwrap_or(true)
                        || excepts
                            .get(apt1.icao.as_str())
                            .filter(|s| s.contains(apt2.icao.as_str()))
                            .is_some()
                        || excepts
                            .get(apt2.icao.as_str())
                            .filter(|s| s.contains(&apt1.icao.as_str()))
                            .is_some()
                })
            }),
        }
    }

    pub fn transform(&self, f: impl Fn(f64) -> f64) -> Self {
        Self {
            graph: self.graph.transform(|d| d.map(|v| f(v))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::FRAC_PI_2;
    use std::marker::PhantomData;

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
        let distances_idx = DistancesIdx::from(&apt_idx, None, &HashMap::new());
        let quarter = quarter();
        for apt1 in 0..airports.len() as u32 {
            for apt2 in 0..airports.len() as u32 {
                assert_eq!(
                    distances_idx.between(apt1, apt2),
                    if apt1 == apt2 { None } else { Some(quarter) }
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
        let distances_idx = DistancesIdx::from(&apt_idx, None, &HashMap::new());
        let quarter = quarter();
        assert_eq!(
            distances_idx,
            DistancesIdx {
                graph: GraphIdx {
                    size: 3,
                    edges: vec![Some(quarter); 3],
                    _pd: PhantomData
                }
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
