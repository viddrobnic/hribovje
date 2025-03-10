//! Provides utilities for reading and writing points.
use std::{
    fs,
    io::{self, BufRead},
    path::Path,
};

use thiserror::Error;

use crate::Point;

/// The error type that can occur during data import.
///
/// Error can originate from underlying I/O operations or
/// from being unable to parse the data because of invalid format.
#[derive(Debug, Error)]
pub enum ImportError {
    #[error("io error")]
    Io(#[from] io::Error),

    #[error("invalid data (expected 3 components, found {0})")]
    InvalidData(u8),
}

pub type ImportResult = Result<(), ImportError>;

struct PointWriter<W: io::Write>(W);

impl<W: io::Write> PointWriter<W> {
    fn write(&mut self, point: &Point<f32>) -> Result<(), io::Error> {
        let buf = point.x.to_le_bytes();
        self.0.write_all(&buf)?;

        let buf = point.y.to_le_bytes();
        self.0.write_all(&buf)?;

        let buf = point.data.to_le_bytes();
        self.0.write_all(&buf)?;

        Ok(())
    }
}

struct PointReader<R: io::Read>(R);

impl<R: io::Read> PointReader<R> {
    fn read(&mut self) -> Result<Vec<Point<f32>>, io::Error> {
        let mut points = vec![];

        let mut comps = [0f32; 3];
        let mut comp_idx = 0;
        let mut buf = [0u8; 4];

        loop {
            match self.0.read_exact(&mut buf) {
                Ok(_) => {
                    let comp = f32::from_le_bytes(buf);
                    comps[comp_idx] = comp;
                    comp_idx += 1;

                    if comp_idx == 3 {
                        points.push(Point {
                            x: comps[0],
                            y: comps[1],
                            data: comps[2],
                        });
                        comp_idx = 0;
                    }
                }

                // We reached EOF and can break
                Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => break,
                // Handle other errors
                Err(err) => return Err(err),
            }
        }

        Ok(points)
    }
}

/// Read points from provided reader.
///
/// Returned points contain height as data.
///
/// If reading from file, you should wrap it into
/// [BufReader](https://doc.rust-lang.org/std/io/struct.BufReader.html)
/// to improve the performance.
pub fn read_points(reader: impl io::Read) -> Result<Vec<Point<f32>>, io::Error> {
    let mut reader = PointReader(reader);
    reader.read()
}

/// Write points to provided reader.
///
/// Write points that have height as additional data.
///
/// If writing to file, you should wrap it into
/// [BufWriter](https://doc.rust-lang.org/std/io/struct.BufWriter.html)
/// to improve the performance.
pub fn write_points(writer: impl io::Write, points: &[Point<f32>]) -> Result<(), io::Error> {
    let mut writer = PointWriter(writer);
    for p in points {
        writer.write(p)?;
    }

    Ok(())
}

/// Imports raw data from provided path.
///
/// Parsed points are written to provided writer.
/// If writing to file, you should wrap it into
/// [BufWriter](https://doc.rust-lang.org/std/io/struct.BufWriter.html)
/// to improve the performance.
/// See [`crate`] for more info on data format.
pub fn import_data(input_path: impl AsRef<Path>, writer: impl io::Write) -> ImportResult {
    let mut writer = PointWriter(writer);
    import_recursive(&input_path, &mut writer)?;

    Ok(())
}

fn import_recursive<W: io::Write>(
    input: impl AsRef<Path>,
    writer: &mut PointWriter<W>,
) -> ImportResult {
    let entries = fs::read_dir(input)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            import_recursive(&path, writer)?;
        } else {
            import_file(&path, writer)?;
        }
    }

    Ok(())
}

fn import_file<W: io::Write>(input: impl AsRef<Path>, writer: &mut PointWriter<W>) -> ImportResult {
    // Ignore non .xyz files
    let Some(ext) = input.as_ref().extension() else {
        return Ok(());
    };

    if ext.to_str() != Some("xyz") {
        return Ok(());
    }

    let file = fs::File::open(input)?;
    let mut reader = io::BufReader::new(file);

    let mut buf = String::new();
    loop {
        buf.clear();
        let bytes = reader.read_line(&mut buf)?;
        if bytes == 0 {
            break;
        }

        let mut iter = buf.split_whitespace().filter_map(|s| s.parse::<f32>().ok());
        let arr: [_; 3] = std::array::from_fn(|_| iter.next());

        writer.write(&Point {
            x: arr[0].ok_or(ImportError::InvalidData(0))?,
            y: arr[1].ok_or(ImportError::InvalidData(1))?,
            data: arr[2].ok_or(ImportError::InvalidData(2))?,
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::Point;

    use super::{PointReader, PointWriter};

    #[test]
    fn point_read_write() {
        let mut buf = [0u8; 4 * 3 * 2]; // 4 bytes per f32 * 3 f32 per point * 2 points
        let mut writer = PointWriter(&mut buf[..]);

        // Write points
        let points = vec![
            Point {
                x: 0.5,
                y: 1.0,
                data: -1.2,
            },
            Point {
                x: 2.0,
                y: 3.0,
                data: -4.0,
            },
        ];
        for p in &points {
            writer.write(p).unwrap();
        }

        // Read points
        let mut reader = PointReader(&buf[..]);
        let got_points = reader.read().unwrap();

        assert_eq!(points, got_points);
    }
}
