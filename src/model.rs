use crate::math::great_circle;
use crate::types::field::coord::Coord;
use crate::types::record::AirportPrimaryRecord;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Airport {
    pub icao: String,
    pub name: String,
    pub coord: Coord,
}

impl Airport {
    pub fn distance_to_coord(&self, coord: Coord) -> f64 {
        great_circle(self.coord, coord)
    }

    pub fn distance_to(&self, other: &Airport) -> f64 {
        self.distance_to_coord(other.coord)
    }
}

impl<'a: 'b, 'b> From<&'b AirportPrimaryRecord<'a>> for Airport {
    fn from(value: &AirportPrimaryRecord<'a>) -> Self {
        Self {
            icao: value.icao_identifier.to_string(),
            name: value.airport_name.to_string(),
            coord: (
                &value.airport_reference_point_latitude,
                &value.airport_reference_point_longitude,
            )
                .into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AirportIdx<'a> {
    pub aps: &'a [Airport],
    pub idx_by_icao: HashMap<&'a str, u32>,
}

impl<'a> AirportIdx<'a> {
    pub fn new(aps: &'a [Airport]) -> Option<Self> {
        let idx_by_icao: HashMap<_, _> = aps
            .iter()
            .enumerate()
            .map(|(i, apt)| (&apt.icao[..], i as u32))
            .collect();
        if aps.len() != idx_by_icao.len() {
            None
        } else {
            Some(Self { aps, idx_by_icao })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::record::parse_airport_primary_record;

    #[test]
    fn test_apt_from_apr() {
        let record = b"SUSAP KLAXK2ALAX     0     \
        129YHN33563299W118242898E012000128         1800018000C    \
        MNAR    LOS ANGELES INTL              310231906";
        let apr = parse_airport_primary_record(&record[..]).unwrap();
        let apt = Airport::from(&apr);
        let coord = (
            &apr.airport_reference_point_latitude,
            &apr.airport_reference_point_longitude,
        )
            .into();
        assert_eq!(
            apt,
            Airport {
                name: "LOS ANGELES INTL".to_string(),
                icao: "KLAX".to_string(),
                coord
            }
        );
    }

    #[test]
    fn test_apt_idx_from_apr() {
        let record = b"SUSAP KLAXK2ALAX     0     \
        129YHN33563299W118242898E012000128         1800018000C    \
        MNAR    LOS ANGELES INTL              310231906";
        let apr = parse_airport_primary_record(&record[..]).unwrap();
        let apt = [Airport::from(&apr)];
        let apt_idx = AirportIdx::new(&apt);
        assert_eq!(
            apt_idx,
            Some(AirportIdx {
                aps: &apt,
                idx_by_icao: HashMap::from([("KLAX", 0)])
            })
        );
    }
}
