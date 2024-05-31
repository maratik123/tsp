#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Longitude {
    pub hemisphere: LongitudeHemisphere,
    pub degrees: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub fractional_seconds: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LongitudeHemisphere {
    East,
    West,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Latitude {
    pub hemisphere: LatitudeHemisphere,
    pub degrees: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub fractional_seconds: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LatitudeHemisphere {
    North,
    South,
}

impl From<&Longitude> for f64 {
    fn from(value: &Longitude) -> Self {
        coord_to_radians(
            match value.hemisphere {
                LongitudeHemisphere::East => false,
                LongitudeHemisphere::West => true,
            },
            value.degrees,
            value.minutes,
            value.seconds,
            value.fractional_seconds,
        )
    }
}

impl From<&Latitude> for f64 {
    fn from(value: &Latitude) -> Self {
        coord_to_radians(
            match value.hemisphere {
                LatitudeHemisphere::North => false,
                LatitudeHemisphere::South => true,
            },
            value.degrees,
            value.minutes,
            value.seconds,
            value.fractional_seconds,
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Coord {
    pub lat: f64,
    pub lon: f64,
}

impl From<(&Latitude, &Longitude)> for Coord {
    fn from((lat, lon): (&Latitude, &Longitude)) -> Self {
        Coord {
            lat: lat.into(),
            lon: lon.into(),
        }
    }
}

const RADIANS_PER_DEGREE: f64 = std::f64::consts::PI / 180.0;
const FRAC_100: f64 = 1.0 / 100.0;
const FRAC_60: f64 = 1.0 / 60.0;

fn coord_to_radians(
    neg: bool,
    degrees: u8,
    minutes: u8,
    seconds: u8,
    fractional_seconds: u8,
) -> f64 {
    let (degrees, minutes, seconds, fractional_seconds) = (
        degrees as f64,
        minutes as f64,
        seconds as f64,
        fractional_seconds as f64,
    );
    let result = fractional_seconds * FRAC_100 + seconds;
    let result = result * FRAC_60 + minutes;
    let result = (result * FRAC_60 + degrees) * RADIANS_PER_DEGREE;
    if neg {
        -result
    } else {
        result
    }
}
