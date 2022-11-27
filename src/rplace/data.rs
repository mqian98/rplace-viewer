use std::fmt;
use speedy2d::dimen::{Vec2, Vector2};
use super::{pixel::PixelColor, reader::parquet::{RPlaceParquetDatapoint, RPlaceParquetDataReader, RPlaceParquetDataIterator}};

pub const DAY_1_START_TIMESTAMP: u64 = 1648817050315000000;
pub const DAY_2_START_TIMESTAMP: u64 = 1648916696239000000;
pub const DAY_3_START_TIMESTAMP: u64 = 1649012633356000000;
pub const MAX_TIMESTAMP: u64 = 1649117640207000000;

pub const DAY_1_START_LINE: u64 = 0;
pub const DAY_2_START_LINE: u64 = 28_201_610;
pub const DAY_3_START_LINE: u64 = 71_784_347;
pub const TOTAL_LINES: u64 = 160_808_191;

// TODO: need to change this to custom type? maybe use u16 for size of coordinate but that can be confusing when doing math. 
// if we use u16, then we always have to make sure we dont overflow
pub type RPlaceCoordinate = Vec2;

#[derive(Copy, Clone, Debug)]
pub struct RPlaceDatapoint {
    pub timestamp: u64,
    pub user_id: u32, 
    pub color: PixelColor,
    pub coordinate: RPlaceCoordinate,

    // indicates if a pixel was placed due to moderation
    // will not be true for all pixels placed by mods
    // only is true for swaths of pixels that mods place
    pub is_mod: bool, 
}

impl Default for RPlaceDatapoint {
    fn default() -> Self {
        RPlaceDatapoint { 
            timestamp: 0, 
            user_id: 0, 
            color: PixelColor::Black, 
            coordinate: RPlaceCoordinate::new(0.0, 0.0), 
            is_mod: false, 
        }
    }
}

impl TryFrom<&RPlaceParquetDatapoint> for RPlaceDatapoint {
    type Error = ();

    // Note: This will always create a pixel at coordinate (x1, y1) 
    fn try_from(item: &RPlaceParquetDatapoint) -> Result<Self, Self::Error> {
        let is_mod = (item.x2, item.y2) != (i16::MIN, i16::MIN);

        match PixelColor::try_from(item.rgb) {
            Ok(color) => Ok(RPlaceDatapoint {
                timestamp: item.timestamp as u64,
                user_id: item.user_id as u32,
                color,
                coordinate: Vec2::new(item.x1 as f32, item.y1 as f32),
                is_mod: is_mod,
            }),
            _ => Err(())
        }
    }
}

impl TryFrom<RPlaceParquetDatapoint> for RPlaceDatapoint {
    type Error = ();

    fn try_from(item: RPlaceParquetDatapoint) -> Result<Self, Self::Error> {
        (&item).try_into()
    }
}

impl TryFrom<&RPlaceParquetDatapoint> for Vec<RPlaceDatapoint> {
    type Error = ();
    fn try_from(item: &RPlaceParquetDatapoint) -> Result<Self, Self::Error> {
        let mut vector = Vec::new();
        for y in item.y1..item.y2+1 {
            for x in item.x1..item.x2+1 {
                match PixelColor::try_from(item.rgb) {
                    Ok(color) => vector.push(RPlaceDatapoint {
                        timestamp: item.timestamp as u64,
                        user_id: item.user_id as u32,
                        color,
                        coordinate: Vec2::new(x as f32, y as f32),
                        is_mod: false,
                    }),
                    _ => return Err(()),
                }
            }
        }

        Ok(vector)
    }
}

impl TryFrom<RPlaceParquetDatapoint> for Vec<RPlaceDatapoint> {
    type Error = ();
    fn try_from(item: RPlaceParquetDatapoint) -> Result<Self, Self::Error> {
        (&item).try_into()
    }
}

pub struct RPlaceDataReader {
    file_path: String,
    reader: RPlaceParquetDataReader,
}

impl fmt::Debug for RPlaceDataReader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RPlaceDataReader")
         .field("file_path", &self.file_path)
         .finish()
    }
}

impl RPlaceDataReader {
    pub fn new(file_path: &str) -> Option<RPlaceDataReader> {
        match RPlaceParquetDataReader::new(file_path) {
            Some(reader) => Some(RPlaceDataReader{ 
                file_path: file_path.to_string(), 
                reader 
            }),
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct RPlaceDataIterator<'a> {
    iter: RPlaceParquetDataIterator<'a>,
    position: Option<Vector2<usize>>,

    // all values should be less than the x,y limit
    limit: Option<Vector2<usize>>,
    cached_datapoint: Option<RPlaceDatapoint>,
}

impl<'a> Iterator for RPlaceDataIterator<'a> {
    type Item = RPlaceDatapoint;

    fn next(&mut self) -> Option<Self::Item> {
        if let (Some(position), Some(limit), Some(mut cached_datapoint)) = (self.position, self.limit, self.cached_datapoint) {
            let x = position.x;
            let y = position.y;
            let x_end = limit.x as usize;
            let y_end = limit.y as usize;
            
            cached_datapoint.coordinate = Vec2::new(x as f32, y as f32);

            if x + 1 < x_end {
                self.position = Some(Vector2::new(x + 1, y));
            } else if y + 1 < y_end {
                self.position = Some(Vector2::new(0, y + 1));
            } else {
                self.position = None;
                self.cached_datapoint = None;
            }

            return Some(cached_datapoint);
        }

        if let Some(parquet_datapoint) = self.iter.next() {
            if let Ok(datapoint) = RPlaceDatapoint::try_from(parquet_datapoint) {
                if datapoint.is_mod {
                    self.position = Some(Vector2::new(parquet_datapoint.x1 as usize, parquet_datapoint.y1 as usize));
                    self.limit = Some(Vector2::new(parquet_datapoint.x2 as usize + 1, parquet_datapoint.y2 as usize + 1));
                    self.cached_datapoint = Some(datapoint);
                    return self.next();
                }

                return Some(datapoint);
            }
        }

        return None;
    }
}

impl IntoIterator for RPlaceDataReader {
    type Item = RPlaceDatapoint;

    type IntoIter = RPlaceDataIterator<'static>;

    fn into_iter(self) -> Self::IntoIter {
        RPlaceDataIterator {
            iter: self.reader.into_iter(),
            position: None,
            limit: None,
            cached_datapoint: None,
        }
    }
}
