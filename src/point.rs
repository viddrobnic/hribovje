/// A data point on the map in D96/TM format.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point {
    /// Returns squared distance between two points.
    pub fn distance_sq(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;

        dx * dx + dy * dy
    }

    /// Returns distance between two points in meters.
    pub fn distance(&self, other: &Point) -> f32 {
        self.distance_sq(other).sqrt()
    }
}
