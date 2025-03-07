use crate::Point;

/// Square on map with `width = height = 2 * radius`.
#[derive(Debug)]
pub struct Area {
    pub center: Point,
    pub radius: f32,
}

impl Area {
    /// Creates the minimum area that contains all the points
    pub fn from_points(points: &[Point]) -> Self {
        let (min_x, min_y, max_x, max_y) = points.iter().fold(
            (f32::MAX, f32::MAX, f32::MIN, f32::MIN),
            |(min_x, min_y, max_x, max_y), p| {
                (
                    min_x.min(p.x),
                    min_y.min(p.y),
                    max_x.max(p.x),
                    max_y.max(p.y),
                )
            },
        );

        let width = max_x - min_x;
        let height = max_y - min_y;

        Area {
            center: Point {
                x: width / 2.0 + min_x,
                y: height / 2.0 + min_y,
                z: 0.0,
            },
            radius: width.max(height) / 2.0,
        }
    }

    /// Returns weather the point is inside the area.
    pub fn is_point_inside(&self, point: &Point) -> bool {
        let x_inside =
            point.x >= self.center.x - self.radius && point.x <= self.center.x + self.radius;
        let y_inside =
            point.y >= self.center.y - self.radius && point.y <= self.center.y + self.radius;

        x_inside && y_inside
    }

    /// Returns weather two areas intersect.
    pub fn intersects(&self, other: &Self) -> bool {
        let dx = (self.center.x - other.center.x).abs();
        let dy = (self.center.y - other.center.y).abs();

        let x_inter = dx <= self.radius + other.radius;
        let y_inter = dy <= self.radius + other.radius;
        x_inter && y_inter
    }
}

#[cfg(test)]
mod tests {
    use crate::Point;

    use super::Area;

    #[test]
    fn area_intersects() {
        let cases = [
            (
                Area {
                    center: Point {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    radius: 1.0,
                },
                Area {
                    center: Point {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    radius: 1.0,
                },
                true,
            ),
            (
                Area {
                    center: Point {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    radius: 1.0,
                },
                Area {
                    center: Point {
                        x: 2.0,
                        y: 2.0,
                        z: 0.0,
                    },
                    radius: 1.0,
                },
                true,
            ),
            (
                Area {
                    center: Point {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    radius: 1.0,
                },
                Area {
                    center: Point {
                        x: 2.0,
                        y: 2.0,
                        z: 0.0,
                    },
                    radius: 0.9,
                },
                false,
            ),
        ];

        for (a1, a2, expected) in &cases {
            assert_eq!(a1.intersects(a2), *expected);
            assert_eq!(a2.intersects(a1), *expected);
        }
    }
}
