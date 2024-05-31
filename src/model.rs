use crate::types::field::coord::Coord;
use crate::types::record::AirportPrimaryRecord;

pub struct Airport {
    pub icao: String,
    pub name: String,
    pub coord: Coord,
}

impl<'a> From<&AirportPrimaryRecord<'a>> for Airport {
    fn from(value: &AirportPrimaryRecord<'a>) -> Self {
        Self {
            icao: value.icao_code.to_string(),
            name: value.airport_name.to_string(),
            coord: (
                &value.airport_reference_point_latitude,
                &value.airport_reference_point_longitude,
            )
                .into(),
        }
    }
}
