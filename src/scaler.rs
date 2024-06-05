use crate::types::field::coord::Coord;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Scaler {
    scale_x: f64,
    scale_y: f64,
    offset_x: f64,
    offset_y: f64,
}

impl Scaler {
    pub fn new(top_left: Coord, bottom_right: Coord, width: u32, height: u32) -> Self {
        let scale_x = (width - 1) as f64 / (bottom_right.lon - top_left.lon);
        let scale_y = (height - 1) as f64 / (bottom_right.lat - top_left.lat);
        let offset_x = top_left.lon * scale_x;
        let offset_y = top_left.lat * scale_y;
        Self {
            scale_x,
            scale_y,
            offset_x,
            offset_y,
        }
    }

    pub fn map(&self, coord: Coord) -> (i32, i32) {
        let x = coord.lon * self.scale_x - self.offset_x;
        let x = x.round() as i32;
        let y = coord.lat * self.scale_y - self.offset_y;
        let y = y.round() as i32;
        (x, y)
    }

    pub fn map_f32(&self, coord: Coord) -> (f32, f32) {
        let x = coord.lon * self.scale_x - self.offset_x;
        let y = coord.lat * self.scale_y - self.offset_y;
        (x as f32, y as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaler_new() {
        let scaler = Scaler::new(
            Coord { lat: 1.0, lon: 0.0 },
            Coord { lat: 0.0, lon: 1.0 },
            100,
            200,
        );

        assert_eq!(
            scaler,
            Scaler {
                scale_x: 99.0,
                scale_y: -199.0,
                offset_x: 0.0,
                offset_y: -199.0
            }
        );

        let scaler = Scaler::new(
            Coord {
                lat: 1.0,
                lon: -1.0,
            },
            Coord {
                lat: -1.0,
                lon: 1.0,
            },
            100,
            200,
        );

        assert_eq!(
            scaler,
            Scaler {
                scale_x: 49.5,
                scale_y: -99.5,
                offset_x: -49.5,
                offset_y: -99.5
            }
        );
    }

    #[test]
    fn test_scaler_map() {
        let scaler = Scaler::new(
            Coord { lat: 1.0, lon: 0.0 },
            Coord { lat: 0.0, lon: 1.0 },
            100,
            200,
        );

        assert_eq!(scaler.map(Coord { lat: 0.0, lon: 0.0 }), (0, 199));
        assert_eq!(scaler.map(Coord { lat: 1.0, lon: 0.0 }), (0, 0));
        assert_eq!(scaler.map(Coord { lat: 0.0, lon: 1.0 }), (99, 199));
        assert_eq!(scaler.map(Coord { lat: 1.0, lon: 1.0 }), (99, 0));
        assert_eq!(scaler.map(Coord { lat: 0.5, lon: 0.5 }), (50, 100));

        let scaler = Scaler::new(
            Coord {
                lat: 1.0,
                lon: -1.0,
            },
            Coord {
                lat: -1.0,
                lon: 1.0,
            },
            100,
            200,
        );

        assert_eq!(
            scaler.map(Coord {
                lat: -1.0,
                lon: -1.0
            }),
            (0, 199)
        );
        assert_eq!(
            scaler.map(Coord {
                lat: 1.0,
                lon: -1.0
            }),
            (0, 0)
        );
        assert_eq!(
            scaler.map(Coord {
                lat: -1.0,
                lon: 1.0
            }),
            (99, 199)
        );
        assert_eq!(scaler.map(Coord { lat: 1.0, lon: 1.0 }), (99, 0));
        assert_eq!(scaler.map(Coord { lat: 0.0, lon: 0.0 }), (50, 100));
        assert_eq!(scaler.map(Coord { lat: 0.5, lon: 0.5 }), (74, 50));
    }
}
