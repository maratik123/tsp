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
