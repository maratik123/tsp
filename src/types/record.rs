use crate::types::field::{
    Altitude, CycleDate, Latitude, Longitude, MagneticTrueIndicator, MagneticVariation,
    PublicMilitaryIndicator, RecordType, RunwaySurfaceCode, TimeZone,
};
use crate::types::field::section_code::EnrichedSectionCode;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AirportPrimaryRecords<'a> {
    pub record_type: RecordType,
    pub customer_area_code: &'a str,
    pub icao_identifier: &'a str,
    pub icao_code: &'a str,
    pub enriched_section_code: EnrichedSectionCode,
    pub ata_designator: &'a str,
    pub continuation_record_number: usize,
    pub speed_limit_altitude: Option<Altitude>,
    pub longest_runway: u16,
    pub ifr_capability: bool,
    pub longest_runway_surface_code: RunwaySurfaceCode,
    pub airport_reference_point_latitude: Latitude,
    pub airport_reference_point_longitude: Longitude,
    pub magnetic_variation: MagneticVariation,
    pub airport_elevation: i32,
    pub speed_limit: Option<u16>,
    pub recommended_navaid: Option<&'a str>,
    pub transition_altitude: Option<u32>,
    pub transition_level: Option<u32>,
    pub public_military_indicator: PublicMilitaryIndicator,
    pub time_zone: Option<TimeZone>,
    pub daylight_indicator: Option<bool>,
    pub magnetic_true_indicator: Option<MagneticTrueIndicator>,
    pub datum_code: &'a str,
    pub airport_name: &'a str,
    pub file_record_number: u32,
    pub cycle_date: CycleDate,
}
