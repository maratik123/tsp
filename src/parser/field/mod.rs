use rust_decimal::Decimal;

use crate::types::field::coord::{Latitude, LatitudeHemisphere, Longitude, LongitudeHemisphere};
use crate::types::field::{
    Altitude, CycleDate, MagneticTrueIndicator, MagneticVariation, PublicMilitaryIndicator,
    RecordType, RunwaySurfaceCode, TimeZone,
};
use crate::util::{
    parse_alpha, parse_alphanum, parse_blank_arr, parse_num_u16, parse_num_u32, parse_num_u8,
    trim_right_spaces,
};

pub mod section_code;

// 5.32 Cycle Date
pub fn parse_cycle_date(cycle_date: &[u8]) -> Option<CycleDate> {
    if cycle_date.len() != 4 {
        return None;
    }
    let year = parse_num_u8(&cycle_date[..2], 2..=2, ..)?;
    let cycle = parse_num_u8(&cycle_date[2..], 2..=2, ..)?;
    Some(CycleDate { year, cycle })
}

// 5.31 File Record Number
pub fn parse_file_record_number(file_record_number: &[u8]) -> Option<u32> {
    parse_num_u32(file_record_number, 5..=5, ..)
}

// 5.71 Airport Name
pub fn parse_airport_name(airport_name: &[u8]) -> Option<&str> {
    parse_alpha(airport_name, ..=30)
}

// 5.197 Datum Code
pub fn parse_datum_code(datum_code: &[u8]) -> Option<&str> {
    parse_alpha(datum_code, 3..=3)
}

// 5.165 Magnetic/True Indicator
pub fn parse_magnetic_true_indicator(
    magnetic_true_indicator: u8,
) -> Option<Option<MagneticTrueIndicator>> {
    Some(match magnetic_true_indicator {
        b'M' => Some(MagneticTrueIndicator::Magnetic),
        b'T' => Some(MagneticTrueIndicator::True),
        b' ' => None,
        _ => None?,
    })
}

// 5.179 Daylight Indicator
pub fn parse_daylight_indicator(daylight_indicator: u8) -> Option<Option<bool>> {
    Some(match daylight_indicator {
        b'Y' => Some(true),
        b'N' => Some(false),
        b' ' => None,
        _ => None?,
    })
}

// 5.178 Time Zone
pub fn parse_time_zone(time_zone: &[u8]) -> Option<Option<TimeZone>> {
    if time_zone.len() != 3 {
        return None;
    }
    Some(match parse_blank_arr(time_zone, 3..=3) {
        None => {
            let hour = match time_zone[0] {
                b'Z' => 0,
                b'A' => -1,
                b'B' => -2,
                b'C' => -3,
                b'D' => -4,
                b'E' => -5,
                b'F' => -6,
                b'G' => -7,
                b'H' => -8,
                b'I' => -9,
                b'K' => -10,
                b'L' => -11,
                b'M' => -12,
                b'N' => 1,
                b'O' => 2,
                b'P' => 3,
                b'Q' => 4,
                b'R' => 5,
                b'S' => 6,
                b'T' => 7,
                b'U' => 8,
                b'V' => 9,
                b'W' => 10,
                b'X' => 11,
                b'Y' => 12,
                _ => None?,
            };
            let max_minute = if matches!(hour, 12 | -12) { 60 } else { 59 };
            let minute = parse_num_u8(&time_zone[1..3], 2..=2, ..max_minute)?;
            Some(TimeZone { hour, minute })
        }
        Some(_) => None,
    })
}

// 5.177 Public/Military Indicator
pub fn parse_public_military_indicator(
    public_military_indicator: u8,
) -> Option<PublicMilitaryIndicator> {
    Some(match public_military_indicator {
        b'C' => PublicMilitaryIndicator::Civil,
        b'M' => PublicMilitaryIndicator::Military,
        b'P' => PublicMilitaryIndicator::Private,
        _ => None?,
    })
}

// 5.53 Transition Altitude
pub fn parse_transition_altitude(transition_altitude: &[u8]) -> Option<Option<u32>> {
    Some(match parse_blank_arr(transition_altitude, 5..=5) {
        None => Some(parse_num_u32(transition_altitude, 5..=5, ..)?),
        Some(_) => None,
    })
}

// 5.23 Recommended Navaid
pub fn parse_recommended_navaid(recommended_navaid: &[u8]) -> Option<Option<&str>> {
    Some(match parse_blank_arr(recommended_navaid, 4..=4) {
        None => Some(parse_alphanum(recommended_navaid, 1..=4)?),
        Some(_) => None,
    })
}

// 5.72 Speed Limit
pub fn parse_speed_limit(speed_limit: &[u8]) -> Option<Option<u16>> {
    Some(match parse_blank_arr(speed_limit, 3..=3) {
        None => Some(parse_num_u16(speed_limit, 3..=3, ..)?),
        Some(_) => None,
    })
}

// 5.55 Airport Elevation
pub fn parse_airport_elevation(airport_elevation: &[u8]) -> Option<i32> {
    if airport_elevation.len() != 5 {
        return None;
    }
    let negative = airport_elevation[0] == b'-';
    let val = parse_num_u32(
        if negative {
            &airport_elevation[1..]
        } else {
            airport_elevation
        },
        4..=5,
        ..,
    )? as i32;
    Some(if negative { -val } else { val })
}

// 5.39 Magnetic Variation
pub fn parse_magnetic_variation(magnetic_variation: &[u8]) -> Option<MagneticVariation> {
    if magnetic_variation.len() != 5 {
        return None;
    }
    let dec = Decimal::try_new(
        parse_num_u32(&magnetic_variation[1..], 4..=4, ..)? as i64,
        1,
    )
    .ok()?;
    Some(match magnetic_variation[0] {
        b'E' => MagneticVariation::East(dec),
        b'W' => MagneticVariation::West(dec),
        b'T' if dec.is_zero() => MagneticVariation::True,
        _ => None?,
    })
}

// 5.37 Airport Reference Point Longitude
pub fn parse_airport_reference_point_longitude(
    airport_reference_point_longitude: &[u8],
) -> Option<Longitude> {
    if airport_reference_point_longitude.len() != 10 {
        None
    } else {
        let hemisphere = parse_longitude_hemisphere(airport_reference_point_longitude[0])?;
        let degrees = parse_num_u8(&airport_reference_point_longitude[1..4], 3..=3, ..=180)?;
        let minutes = parse_num_u8(&airport_reference_point_longitude[4..6], 2..=2, ..60)?;
        let seconds = parse_num_u8(&airport_reference_point_longitude[6..8], 2..=2, ..60)?;
        let fractional_seconds =
            parse_num_u8(&airport_reference_point_longitude[8..10], 2..=2, ..)?;
        if (degrees == 0
            && minutes == 0
            && seconds == 0
            && fractional_seconds == 0
            && hemisphere != LongitudeHemisphere::East)
            || (degrees == 180
                && (minutes != 0
                    || seconds != 0
                    || fractional_seconds != 0
                    || hemisphere != LongitudeHemisphere::East))
        {
            None
        } else {
            Some(Longitude {
                hemisphere,
                degrees,
                minutes,
                seconds,
                fractional_seconds,
            })
        }
    }
}

pub fn parse_longitude_hemisphere(longitude_hemisphere: u8) -> Option<LongitudeHemisphere> {
    Some(match longitude_hemisphere {
        b'E' => LongitudeHemisphere::East,
        b'W' => LongitudeHemisphere::West,
        _ => None?,
    })
}

// 5.36 Airport Reference Point Latitude
pub fn parse_airport_reference_point_latitude(
    airport_reference_point_latitude: &[u8],
) -> Option<Latitude> {
    if airport_reference_point_latitude.len() != 9 {
        None
    } else {
        let hemisphere = parse_latitude_hemisphere(airport_reference_point_latitude[0])?;
        let degrees = parse_num_u8(&airport_reference_point_latitude[1..3], 2..=2, ..=90)?;
        let minutes = parse_num_u8(&airport_reference_point_latitude[3..5], 2..=2, ..60)?;
        let seconds = parse_num_u8(&airport_reference_point_latitude[5..7], 2..=2, ..60)?;
        let fractional_seconds = parse_num_u8(&airport_reference_point_latitude[7..9], 2..=2, ..)?;
        if (degrees == 0
            && minutes == 0
            && seconds == 0
            && fractional_seconds == 0
            && hemisphere != LatitudeHemisphere::North)
            || (degrees == 90 && (minutes != 0 || seconds != 0 || fractional_seconds != 0))
        {
            None
        } else {
            Some(Latitude {
                hemisphere,
                degrees,
                minutes,
                seconds,
                fractional_seconds,
            })
        }
    }
}

fn parse_latitude_hemisphere(latitude_hemisphere: u8) -> Option<LatitudeHemisphere> {
    Some(match latitude_hemisphere {
        b'N' => LatitudeHemisphere::North,
        b'S' => LatitudeHemisphere::South,
        _ => None?,
    })
}

// 5.249 Longest Runway Surface Code
pub fn parse_longest_runway_surface_code(
    longest_runway_surface_code: u8,
) -> Option<RunwaySurfaceCode> {
    Some(match longest_runway_surface_code {
        b'H' => RunwaySurfaceCode::HardSurface,
        b'S' => RunwaySurfaceCode::SoftSurface,
        b'W' => RunwaySurfaceCode::WaterRunway,
        b'U' => RunwaySurfaceCode::Undefined,
        _ => None?,
    })
}

// 5.108 IFR Capability
pub fn parse_ifr_capability(ifr_capability: u8) -> Option<bool> {
    Some(match ifr_capability {
        b'Y' => true,
        b'N' => false,
        _ => None?,
    })
}

// 5.54 Longest Runway
pub fn parse_longest_runway(longest_runway: &[u8]) -> Option<u16> {
    parse_num_u16(longest_runway, 3..=3, ..)
}

// 5.73 Speed Limit Altitude
pub fn parse_speed_limit_altitude(speed_limit_altitude: &[u8]) -> Option<Option<Altitude>> {
    let speed_limit_altitude = trim_right_spaces(speed_limit_altitude);
    Some(if speed_limit_altitude.is_empty() {
        None
    } else if speed_limit_altitude[0] == b'F' {
        let mut remaining_len = 4;
        let mut bytes = &speed_limit_altitude[1..];
        if !bytes.is_empty() && bytes[0] == b'L' {
            remaining_len = 3;
            bytes = &bytes[1..];
        }
        Some(parse_num_u16(bytes, 1..=remaining_len, ..).map(Altitude::Fl)?)
    } else {
        Some(parse_num_u32(speed_limit_altitude, 1..=5, ..).map(Altitude::Msl)?)
    })
}

// 5.16 Continuation Record Number
pub fn parse_continuation_record_number(continuation_record: u8, is_primary: bool) -> Option<u8> {
    Some(if is_primary {
        match continuation_record {
            b'0'..=b'1' => continuation_record - b'0',
            _ => None?,
        }
    } else {
        match continuation_record {
            b'2'..=b'9' => continuation_record - b'0',
            b'A'..=b'Z' => continuation_record - b'A' + 10,
            _ => None?,
        }
    })
}

// 5.107 ATA Designator
pub fn parse_ata_designator(ata_designator: &[u8]) -> Option<&str> {
    parse_alpha(ata_designator, 3..=3)
}

// 5.14 ICAO Code
pub fn parse_icao_code(icao_code: &[u8]) -> Option<&str> {
    parse_alphanum(icao_code, ..=2)
}

// 5.6 ICAO Identifier
pub fn parse_icao_identifier(icao_identifier: &[u8]) -> Option<&str> {
    parse_alphanum(icao_identifier, ..=4)
}

// 5.3 Customer Area Code
pub fn parse_customer_area_code(customer_area_code: &[u8]) -> Option<&str> {
    parse_alpha(customer_area_code, ..=3)
}

// 5.2 Record Type
pub fn parse_record_type(record_type: u8) -> Option<RecordType> {
    Some(match record_type {
        b'S' => RecordType::Standard,
        b'T' => RecordType::Tailored,
        _ => None?,
    })
}
