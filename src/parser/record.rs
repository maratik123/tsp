use crate::consts::ENTRY_LEN;
use crate::parser::field::section_code::{parse_section_code, parse_subsection_code};
use crate::parser::field::{
    parse_airport_elevation, parse_airport_name, parse_airport_reference_point_latitude,
    parse_airport_reference_point_longitude, parse_ata_designator,
    parse_continuation_record_number, parse_customer_area_code, parse_cycle_date, parse_datum_code,
    parse_daylight_indicator, parse_file_record_number, parse_icao_code, parse_icao_identifier,
    parse_ifr_capability, parse_longest_runway, parse_longest_runway_surface_code,
    parse_magnetic_true_indicator, parse_magnetic_variation, parse_public_military_indicator,
    parse_recommended_navaid, parse_record_type, parse_speed_limit, parse_speed_limit_altitude,
    parse_time_zone, parse_transition_altitude,
};
use crate::types::field::section_code::{AirportSubsectionCode, EnrichedSectionCode, SectionCode};
use crate::types::record::AirportPrimaryRecords;
use crate::util::{parse_blank, parse_blank_arr};

pub fn parse_airport_primary_records(rec: &[u8]) -> Option<AirportPrimaryRecords> {
    if rec.len() != ENTRY_LEN {
        return None;
    }
    let record_type = parse_record_type(rec[0])?; // 5.2
    let customer_area_code = parse_customer_area_code(&rec[1..4])?; // 5.3
    let section_code = parse_section_code(rec[4])?; // 5.4
    if section_code != SectionCode::Airport {
        return None;
    }
    parse_blank(rec[5])?;
    let icao_identifier = parse_icao_identifier(&rec[6..10])?; // 5.6
    let mut icao_code = parse_icao_code(&rec[10..12])?; // 5.14
    let enriched_section_code = parse_subsection_code(section_code, rec[12])?; // 5.5
    if enriched_section_code != EnrichedSectionCode::Airport(AirportSubsectionCode::ReferencePoints)
    {
        return None;
    }
    let ata_designator = parse_ata_designator(&rec[13..16])?; // 5.107
    let _reserved = &rec[16..18];
    parse_blank_arr(&rec[18..21], 3..=3)?;
    let continuation_record_number = parse_continuation_record_number(rec[21], true)?; // 5.16
    if !(..=1).contains(&continuation_record_number) {
        return None;
    }
    let speed_limit_altitude = parse_speed_limit_altitude(&rec[22..27])?; // 5.73
    let longest_runway = parse_longest_runway(&rec[27..30])?; // 5.54
    let ifr_capability = parse_ifr_capability(rec[30])?; // 5.108
    let longest_runway_surface_code = parse_longest_runway_surface_code(rec[31])?; // 5.249
    let airport_reference_point_latitude = parse_airport_reference_point_latitude(&rec[32..41])?; // 5.36
    let airport_reference_point_longitude = parse_airport_reference_point_longitude(&rec[41..51])?; // 5.37
    let magnetic_variation = parse_magnetic_variation(&rec[51..56])?; // 5.39
    let airport_elevation = parse_airport_elevation(&rec[56..61])?; // 5.55
    let speed_limit = parse_speed_limit(&rec[61..64])?; // 5.72
    let recommended_navaid = parse_recommended_navaid(&rec[64..68])?; // 5.23
    let icao_code2 = parse_icao_code(&rec[68..70])?; // 5.14
    if !(icao_code.is_empty() || icao_code2.is_empty()) && icao_code != icao_code2 {
        return None;
    } else if icao_code.is_empty() {
        icao_code = icao_code2;
    }
    let transition_altitude = parse_transition_altitude(&rec[70..75])?; // 5.53
    let transition_level = parse_transition_altitude(&rec[75..80])?; // 5.53
    let public_military_indicator = parse_public_military_indicator(rec[80])?; // 5.177
    let time_zone = parse_time_zone(&rec[81..84])?; // 5.178
    let daylight_indicator = parse_daylight_indicator(rec[84])?; // 5.179
    let magnetic_true_indicator = parse_magnetic_true_indicator(rec[85])?; // 5.165
    let datum_code = parse_datum_code(&rec[86..89])?; //5.197
    let _reserved = &rec[89..93];
    let airport_name = parse_airport_name(&rec[93..123])?; // 5.71
    let file_record_number = parse_file_record_number(&rec[123..128])?; // 5.31
    let cycle_date = parse_cycle_date(&rec[128..132])?; // 5.32
    Some(AirportPrimaryRecords {
        record_type,
        customer_area_code,
        icao_identifier,
        icao_code,
        enriched_section_code,
        ata_designator,
        continuation_record_number,
        speed_limit_altitude,
        longest_runway,
        ifr_capability,
        longest_runway_surface_code,
        airport_reference_point_latitude,
        airport_reference_point_longitude,
        magnetic_variation,
        airport_elevation,
        speed_limit,
        recommended_navaid,
        transition_altitude,
        transition_level,
        public_military_indicator,
        time_zone,
        daylight_indicator,
        magnetic_true_indicator,
        datum_code,
        airport_name,
        file_record_number,
        cycle_date,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::field::{
        CycleDate, Latitude, LatitudeHemisphere, Longitude, LongitudeHemisphere,
        MagneticTrueIndicator, MagneticVariation, PublicMilitaryIndicator, RecordType,
        RunwaySurfaceCode,
    };
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn parse_klax() {
        let record = b"SUSAP KLAXK2ALAX     0     \
        129YHN33563299W118242898E012000128         1800018000C    \
        MNAR    LOS ANGELES INTL              310231906";
        let parsed = parse_airport_primary_records(&record[..]).unwrap();
        assert_eq!(
            parsed,
            AirportPrimaryRecords {
                record_type: RecordType::Standard,
                customer_area_code: "USA",
                icao_identifier: "KLAX",
                icao_code: "K2",
                enriched_section_code: EnrichedSectionCode::Airport(
                    AirportSubsectionCode::ReferencePoints
                ),
                ata_designator: "LAX",
                continuation_record_number: 0,
                speed_limit_altitude: None,
                longest_runway: 129,
                ifr_capability: true,
                longest_runway_surface_code: RunwaySurfaceCode::HardSurface,
                airport_reference_point_latitude: Latitude {
                    hemisphere: LatitudeHemisphere::North,
                    degrees: 33,
                    minutes: 56,
                    seconds: 32,
                    fractional_seconds: 99
                },
                airport_reference_point_longitude: Longitude {
                    hemisphere: LongitudeHemisphere::West,
                    degrees: 118,
                    minutes: 24,
                    seconds: 28,
                    fractional_seconds: 98
                },
                magnetic_variation: MagneticVariation::East(Decimal::from_str("12").unwrap()),
                airport_elevation: 128,
                speed_limit: None,
                recommended_navaid: None,
                transition_altitude: Some(18000),
                transition_level: Some(18000),
                public_military_indicator: PublicMilitaryIndicator::Civil,
                time_zone: None,
                daylight_indicator: None,
                magnetic_true_indicator: Some(MagneticTrueIndicator::Magnetic),
                datum_code: "NAR",
                airport_name: "LOS ANGELES INTL",
                file_record_number: 31023,
                cycle_date: CycleDate { year: 19, cycle: 6 },
            }
        );
    }

    #[test]
    fn parse_ksea() {
        let record = b"SUSAP KSEAK1ASEA     0     \
        119YHN47265960W122184240E016000432         1800018000C    \
        MNAR    SEATTLE-TACOMA INTL           065001807";
        let parsed = parse_airport_primary_records(&record[..]).unwrap();
        assert_eq!(
            parsed,
            AirportPrimaryRecords {
                record_type: RecordType::Standard,
                customer_area_code: "USA",
                icao_identifier: "KSEA",
                icao_code: "K1",
                enriched_section_code: EnrichedSectionCode::Airport(
                    AirportSubsectionCode::ReferencePoints
                ),
                ata_designator: "SEA",
                continuation_record_number: 0,
                speed_limit_altitude: None,
                longest_runway: 119,
                ifr_capability: true,
                longest_runway_surface_code: RunwaySurfaceCode::HardSurface,
                airport_reference_point_latitude: Latitude {
                    hemisphere: LatitudeHemisphere::North,
                    degrees: 47,
                    minutes: 26,
                    seconds: 59,
                    fractional_seconds: 60
                },
                airport_reference_point_longitude: Longitude {
                    hemisphere: LongitudeHemisphere::West,
                    degrees: 122,
                    minutes: 18,
                    seconds: 42,
                    fractional_seconds: 40
                },
                magnetic_variation: MagneticVariation::East(Decimal::from_str("16").unwrap()),
                airport_elevation: 432,
                speed_limit: None,
                recommended_navaid: None,
                transition_altitude: Some(18000),
                transition_level: Some(18000),
                public_military_indicator: PublicMilitaryIndicator::Civil,
                time_zone: None,
                daylight_indicator: None,
                magnetic_true_indicator: Some(MagneticTrueIndicator::Magnetic),
                datum_code: "NAR",
                airport_name: "SEATTLE-TACOMA INTL",
                file_record_number: 6500,
                cycle_date: CycleDate { year: 18, cycle: 7 },
            }
        );
    }

    #[test]
    fn parse_kden() {
        let record = b"SUSAP KDENK2ADEN     0     \
        160YHN39514200W104402340E008005434         1800018000C    \
        MNAR    DENVER INTL                   630481208";
        let parsed = parse_airport_primary_records(&record[..]).unwrap();
        assert_eq!(
            parsed,
            AirportPrimaryRecords {
                record_type: RecordType::Standard,
                customer_area_code: "USA",
                icao_identifier: "KDEN",
                icao_code: "K2",
                enriched_section_code: EnrichedSectionCode::Airport(
                    AirportSubsectionCode::ReferencePoints
                ),
                ata_designator: "DEN",
                continuation_record_number: 0,
                speed_limit_altitude: None,
                longest_runway: 160,
                ifr_capability: true,
                longest_runway_surface_code: RunwaySurfaceCode::HardSurface,
                airport_reference_point_latitude: Latitude {
                    hemisphere: LatitudeHemisphere::North,
                    degrees: 39,
                    minutes: 51,
                    seconds: 42,
                    fractional_seconds: 0
                },
                airport_reference_point_longitude: Longitude {
                    hemisphere: LongitudeHemisphere::West,
                    degrees: 104,
                    minutes: 40,
                    seconds: 23,
                    fractional_seconds: 40
                },
                magnetic_variation: MagneticVariation::East(Decimal::from_str("8").unwrap()),
                airport_elevation: 5434,
                speed_limit: None,
                recommended_navaid: None,
                transition_altitude: Some(18000),
                transition_level: Some(18000),
                public_military_indicator: PublicMilitaryIndicator::Civil,
                time_zone: None,
                daylight_indicator: None,
                magnetic_true_indicator: Some(MagneticTrueIndicator::Magnetic),
                datum_code: "NAR",
                airport_name: "DENVER INTL",
                file_record_number: 63048,
                cycle_date: CycleDate { year: 12, cycle: 8 },
            }
        );
    }

    #[test]
    fn parse_kjfk() {
        let record = b"SUSAP KJFKK6AJFK     0     \
        145YHN40382374W073464329W013000013         1800018000C    \
        MNAR    JOHN F KENNEDY INTL           257211912";
        let parsed = parse_airport_primary_records(&record[..]).unwrap();
        assert_eq!(
            parsed,
            AirportPrimaryRecords {
                record_type: RecordType::Standard,
                customer_area_code: "USA",
                icao_identifier: "KJFK",
                icao_code: "K6",
                enriched_section_code: EnrichedSectionCode::Airport(
                    AirportSubsectionCode::ReferencePoints
                ),
                ata_designator: "JFK",
                continuation_record_number: 0,
                speed_limit_altitude: None,
                longest_runway: 145,
                ifr_capability: true,
                longest_runway_surface_code: RunwaySurfaceCode::HardSurface,
                airport_reference_point_latitude: Latitude {
                    hemisphere: LatitudeHemisphere::North,
                    degrees: 40,
                    minutes: 38,
                    seconds: 23,
                    fractional_seconds: 74
                },
                airport_reference_point_longitude: Longitude {
                    hemisphere: LongitudeHemisphere::West,
                    degrees: 73,
                    minutes: 46,
                    seconds: 43,
                    fractional_seconds: 29
                },
                magnetic_variation: MagneticVariation::West(Decimal::from_str("13").unwrap()),
                airport_elevation: 13,
                speed_limit: None,
                recommended_navaid: None,
                transition_altitude: Some(18000),
                transition_level: Some(18000),
                public_military_indicator: PublicMilitaryIndicator::Civil,
                time_zone: None,
                daylight_indicator: None,
                magnetic_true_indicator: Some(MagneticTrueIndicator::Magnetic),
                datum_code: "NAR",
                airport_name: "JOHN F KENNEDY INTL",
                file_record_number: 25721,
                cycle_date: CycleDate {
                    year: 19,
                    cycle: 12
                },
            }
        );
    }
}
