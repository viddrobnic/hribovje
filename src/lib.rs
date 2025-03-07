//! Parse and query Slovenian national topographic data. The data can be gotten
//! from [public geodetic data collection](https://ipi.eprostor.gov.si/jgp/data),
//! `Portal Prostor > Data collection > National topographic system > Digital elevation model`.
//! The data this library is meant to work with, is contained in the four `DEM 0050` files.
//!
//! ## Data format
//!
//! The `DEM 0050` is Slovenia split into four regions - NW, NE, SW, SE.
//! It contains multiple `.xyz` files which contain data in format:
//!
//! ```text
//! <x> <y> <height_in_meters>
//! ```
//!
//! Point `(x, y)` represents a location on the map in the
//! [D96/TM format](https://www.e-prostor.gov.si/podrocja/drzavni-koordinatni-sistem/horizontalna-sestavina/),
//! also known as [EPSG:3794](https://epsg.io/3794).

mod area;
mod point;

pub mod data;
pub mod qtree;

pub use area::*;
pub use point::*;
