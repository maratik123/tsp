pub mod section_code;

use rust_decimal::Decimal;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CycleDate {
    pub year: u8,
    pub cycle: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MagneticTrueIndicator {
    Magnetic,
    True,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimeZone {
    pub hour: i8,
    pub minute: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PublicMilitaryIndicator {
    Civil,
    Military,
    Private,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MagneticVariation {
    East(Decimal),
    West(Decimal),
    True,
}

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RunwaySurfaceCode {
    HardSurface,
    SoftSurface,
    WaterRunway,
    Undefined,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Altitude {
    Fl(u16),
    Msl(u32),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RecordType {
    Standard,
    Tailored,
}
