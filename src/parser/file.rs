use crate::parser::record::parse_airport_primary_record;
use crate::types::record::AirportPrimaryRecord;
use crate::util::trim_right_0d;

pub fn parse_airport_primary_records(buf: &[u8]) -> impl Iterator<Item = AirportPrimaryRecord> {
    buf.split(|&c| c == b'\n')
        .map(trim_right_0d)
        .filter_map(parse_airport_primary_record)
}
