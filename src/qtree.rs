//! Implementation of quad tree data structure.
//!
//! This is a very specific implementation that allows
//! for zero allocation query and removal of points in
//! a single operation.

use thiserror::Error;

use crate::{Area, Point};

// Max points in leaf node
const MAX_POINTS: usize = 1000;

#[derive(Debug)]
enum NodeInner {
    Leaf {
        points: Vec<Point>,
    },
    Intermediate {
        nw: Box<Node>,
        ne: Box<Node>,
        sw: Box<Node>,
        se: Box<Node>,
    },
}

#[derive(Debug)]
struct Node {
    area: Area,
    inner: NodeInner,
}

#[derive(Debug, Error)]
pub enum InsertError {
    /// Inserted point is outside of the tree area.
    #[error("point is outside of the tree area")]
    OutsideArea,
}

#[derive(Debug, Error)]
pub enum QueryError {
    /// Queried area does not intersect with tree area.
    #[error("area is outside of the tree area")]
    OutsideArea,
}

pub struct QuadTree(Node);

impl QuadTree {
    /// Construct a new quad tree to insert points from specific bounds.
    ///
    /// This tree will only be able to insert and query points inside this area.
    /// Inserting points outside of the provided area will result in an error!
    pub fn new(area: Area) -> Self {
        Self(Node {
            area,
            inner: NodeInner::Leaf { points: vec![] },
        })
    }

    /// Returns the number of points in the tree.
    pub fn size(&self) -> usize {
        self.0.size()
    }

    /// Insert a new point into the tree.
    pub fn insert(&mut self, point: Point) -> Result<(), InsertError> {
        self.0.insert(point)
    }

    /// Queries points inside the given area.
    ///
    /// Points are cloned from the tree and put into `results`.
    /// The method returns number of points that have been written to results.
    /// In other words, you are interested in `results[..return_value]`
    ///
    /// Warning: If there are more points in the area than length of the results,
    /// this method will panic.
    pub fn query(&mut self, area: &Area, results: &mut [Point]) -> Result<usize, QueryError> {
        let mut idx = 0;
        self.0
            .query(area, |points, idx| points[idx].clone(), results, &mut idx)?;

        Ok(idx)
    }

    /// Queries points inside the given area and removes them.
    ///
    /// Points are removed from the tree and put into `results`.
    /// The method returns number of points that have been written to results.
    /// In other words, you are interested in `results[..return_value]`
    ///
    /// Warning: If there are more points in the area than length of the results,
    /// this method will panic.
    pub fn query_remove(
        &mut self,
        area: &Area,
        results: &mut [Point],
    ) -> Result<usize, QueryError> {
        let mut idx = 0;

        self.0.query(
            area,
            |points, idx| points.swap_remove(idx),
            results,
            &mut idx,
        )?;

        Ok(idx)
    }

    /// Finds the point nearest to the given point.
    ///
    /// Point by which you query, has to be in the area of the tree.
    /// If the tree is empty, None is returned.
    pub fn nearest<'a>(&'a self, point: &Point) -> Result<Option<&'a Point>, QueryError> {
        self.0
            .nearest(point)
            .map(|opt_point| opt_point.map(|(_, p)| p))
    }
}

impl Node {
    fn insert(&mut self, point: Point) -> Result<(), InsertError> {
        if !self.area.is_point_inside(&point) {
            return Err(InsertError::OutsideArea);
        }

        match &mut self.inner {
            NodeInner::Intermediate { nw, ne, sw, se } => {
                if nw.area.is_point_inside(&point) {
                    nw.insert(point)
                } else if ne.area.is_point_inside(&point) {
                    ne.insert(point)
                } else if sw.area.is_point_inside(&point) {
                    sw.insert(point)
                } else if se.area.is_point_inside(&point) {
                    se.insert(point)
                } else {
                    unreachable!(
                        "Invalid tree! Point is in tree area, but in any of the subdivisions."
                    );
                }
            }
            NodeInner::Leaf { points } => {
                points.push(point);
                if points.len() > MAX_POINTS {
                    self.subdivide();
                }

                Ok(())
            }
        }
    }

    fn query<F>(
        &mut self,
        area: &Area,
        get_point: F,
        results: &mut [Point],
        idx: &mut usize,
    ) -> Result<(), QueryError>
    where
        F: Fn(&mut Vec<Point>, usize) -> Point + Copy,
    {
        if !self.area.intersects(area) {
            return Err(QueryError::OutsideArea);
        }

        match &mut self.inner {
            NodeInner::Intermediate { nw, ne, sw, se } => {
                if nw.area.intersects(area) {
                    nw.query(area, get_point, results, idx)?;
                }
                if ne.area.intersects(area) {
                    ne.query(area, get_point, results, idx)?;
                }
                if sw.area.intersects(area) {
                    sw.query(area, get_point, results, idx)?;
                }
                if se.area.intersects(area) {
                    se.query(area, get_point, results, idx)?;
                }
            }
            NodeInner::Leaf { points } => {
                let mut i = 0;
                while i < points.len() {
                    if !area.is_point_inside(&points[i]) {
                        i += 1;
                        continue;
                    }

                    let point = get_point(points, i);
                    results[*idx] = point;
                    *idx += 1;
                }
            }
        }

        Ok(())
    }

    fn nearest(&self, point: &Point) -> Result<Option<(f32, &Point)>, QueryError> {
        if !self.area.is_point_inside(point) {
            return Err(QueryError::OutsideArea);
        }

        let res = match &self.inner {
            NodeInner::Intermediate { nw, ne, sw, se } => {
                let mut res = None;

                if nw.area.is_point_inside(point) {
                    res = min_point(res, nw.nearest(point)?);
                }
                if ne.area.is_point_inside(point) {
                    res = min_point(res, nw.nearest(point)?);
                }
                if sw.area.is_point_inside(point) {
                    res = min_point(res, nw.nearest(point)?);
                }
                if se.area.is_point_inside(point) {
                    res = min_point(res, nw.nearest(point)?);
                }

                res
            }
            NodeInner::Leaf { points } => points.iter().fold(None, |acc, p| {
                let distance = p.distance_sq(point);
                min_point(acc, Some((distance, p)))
            }),
        };

        Ok(res)
    }

    fn new_leaf(area: Area) -> Self {
        Self {
            area,
            inner: NodeInner::Leaf { points: vec![] },
        }
    }

    fn subdivide(&mut self) {
        let area = &self.area;

        // Radius is created with a small epsilon to handle numerical error.
        // This means that areas overlap a bit, but that's fine. We are using
        // if/else for insertion, which means point is inserted only in one subsection.
        let r = area.radius / 2.0 + 0.01;

        let nw_area = Area {
            center: Point {
                x: area.center.x - r,
                y: area.center.y - r,
                z: 0.0,
            },
            radius: r,
        };
        let ne_area = Area {
            center: Point {
                x: area.center.x + r,
                y: area.center.y - r,
                z: 0.0,
            },
            radius: r,
        };
        let sw_area = Area {
            center: Point {
                x: area.center.x - r,
                y: area.center.y + r,
                z: 0.0,
            },
            radius: r,
        };
        let se_area = Area {
            center: Point {
                x: area.center.x + r,
                y: area.center.y + r,
                z: 0.0,
            },
            radius: r,
        };

        let mut curr_leaf = NodeInner::Intermediate {
            nw: Box::new(Node::new_leaf(nw_area)),
            ne: Box::new(Node::new_leaf(ne_area)),
            sw: Box::new(Node::new_leaf(sw_area)),
            se: Box::new(Node::new_leaf(se_area)),
        };
        std::mem::swap(&mut curr_leaf, &mut self.inner);

        let NodeInner::Leaf { points } = curr_leaf else {
            panic!("subdivide called on non-leaf node");
        };
        for p in points {
            self.insert(p).expect("subdivide became invalid");
        }
    }

    fn size(&self) -> usize {
        match &self.inner {
            NodeInner::Leaf { points } => points.len(),
            NodeInner::Intermediate { nw, ne, sw, se } => {
                nw.size() + ne.size() + sw.size() + se.size()
            }
        }
    }
}

fn min_point<'a>(
    a: Option<(f32, &'a Point)>,
    b: Option<(f32, &'a Point)>,
) -> Option<(f32, &'a Point)> {
    match (a, b) {
        (None, None) => None,
        (None, Some(x)) => Some(x),
        (Some(x), None) => Some(x),
        (Some((dist_a, point_a)), Some((dist_b, point_b))) => {
            if dist_a < dist_b {
                Some((dist_a, point_a))
            } else {
                Some((dist_b, point_b))
            }
        }
    }
}
