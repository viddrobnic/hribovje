/// A point on the map in D96/TM format.
///
/// Point can hold additional data. By default that data is just unit.
#[derive(Debug, Clone, PartialEq)]
pub struct Point<T = ()> {
    pub x: f32,
    pub y: f32,
    pub data: T,
}

impl<T> Point<T> {
    /// Returns squared distance between two points.
    pub fn distance_sq<U>(&self, other: &Point<U>) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;

        dx * dx + dy * dy
    }

    /// Returns distance between two points in meters.
    pub fn distance<U>(&self, other: &Point<U>) -> f32 {
        self.distance_sq(other).sqrt()
    }
}
