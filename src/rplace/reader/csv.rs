use csv::DeserializeRecordsIntoIter;
use serde::{Deserialize, Deserializer};
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::fs::File;
use std::hash::{Hash, Hasher};
use speedy2d::dimen::{Vector2, Vec2};
use time::PrimitiveDateTime;
use crate::rplace::data::RPlaceDatapoint;
use super::super::pixel::PixelColor;
use serde::de::Error;

pub struct RPlaceCSVDataIterator {
    file_path: String,
    iter: DeserializeRecordsIntoIter<File, RPlaceCSVDatapoint>,
}

impl fmt::Debug for RPlaceCSVDataIterator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RPlaceCSVDataIterator")
         .field("file_path", &self.file_path)
         .finish()
    }
}

impl RPlaceCSVDataIterator {
    pub fn new(file_path: &str) -> Option<RPlaceCSVDataIterator> {
        match File::open(file_path) {
            Ok(file) => {
                let reader = csv::Reader::from_reader(file);
                let iter = reader.into_deserialize();
                Some(RPlaceCSVDataIterator {
                    file_path: file_path.to_string(),
                    iter,
                })
            },
            _ => None
        }
    }

    fn print(n: u32, file_path: &str) -> Result<(), std::io::Error> {
        let f = File::open(file_path)?;
        let mut rdr = csv::Reader::from_reader(f);
        for result in rdr.deserialize().take(n as usize) {
            let record: RPlaceCSVDatapoint = result?;
            println!("{:?} {:?} {:?} {:?}", record.timestamp, record.user_id, record.pixel_color, record.coordinate);
        }

        Ok(())
    }
}


// does not iterate over bad values
impl Iterator for RPlaceCSVDataIterator {
    type Item = RPlaceDatapoint;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(value) = self.iter.next() {
            match value {
                Ok(datapoint) => {
                    return Some(RPlaceDatapoint::from(datapoint));
                },
                Err(error) => println!("Error: failed to parse datapoint. Skipping to next. Error: {:?}", error),
            }
        }
        return None;
    }
}

#[derive(Debug, Deserialize)]
pub struct RPlaceCSVDatapoint {
    #[serde(deserialize_with = "primitive_date_time_from_str")]
    pub timestamp: PrimitiveDateTime,
    pub user_id: String, 
    #[serde(deserialize_with = "pixel_color_from_str")]
    pub pixel_color: PixelColor,
    #[serde(deserialize_with = "vector2_from_str")]
    pub coordinate: Vector2<u32>,
}

impl From<&RPlaceCSVDatapoint> for RPlaceDatapoint {
    fn from(item: &RPlaceCSVDatapoint) -> Self {
        let mut hasher = DefaultHasher::new();
        item.user_id.hash(&mut hasher);
        return RPlaceDatapoint { 
            timestamp: item.timestamp.assume_utc().unix_timestamp() as u64, 
            user_id: hasher.finish() as u32, 
            color: item.pixel_color, 
            coordinate: Vec2::new(item.coordinate.x as f32, item.coordinate.y as f32), 
            is_mod: false, 
        };
    }
}

impl From<RPlaceCSVDatapoint> for RPlaceDatapoint {
    fn from(item: RPlaceCSVDatapoint) -> Self {
        (&item).into()
    }
}

fn primitive_date_time_from_str<'de, D: Deserializer<'de>>(d: D) -> Result<PrimitiveDateTime, D::Error> {
    let s: Option<String> = Deserialize::deserialize(d)?;
    match s {
        Some(datetime) => {
            // 2022-04-04 00:53:51.577 UTC
            let format1 = time::macros::format_description!(
                "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond] UTC"
            );

            // 2022-04-04 01:47:37 UTC
            let format2 = time::macros::format_description!(
                "[year]-[month]-[day] [hour]:[minute]:[second] UTC"
            );

            if let Ok(value) = PrimitiveDateTime::parse(&datetime, &format1) {
                return Ok(value)
            }

            if let Ok(value) = PrimitiveDateTime::parse(&datetime, &format2) {
                return Ok(value)
            }

            Err(D::Error::custom(format!("Failed to parse string to PrimitiveDateTime: string={datetime}")))
        },
        None => {
            Err(D::Error::custom("Failed to deserialize datetime string"))
        },
    }
}

fn pixel_color_from_str<'de, D: Deserializer<'de>>(d: D) -> Result<PixelColor, D::Error> {
    let s: Option<String> = Deserialize::deserialize(d)?;
    match s {
        Some(hex_string) => {
            match PixelColor::try_from(&hex_string) {
                Ok(value) => Ok(value),
                Err(_) => {
                    Err(D::Error::custom(format!("Failed to match hex with pixel color: {hex_string}")))
                },
            }
        },
        None => {
            Err(D::Error::custom("Failed to deserialize hex color string"))
        },
    }
}

fn vector2_from_str<'de, D: Deserializer<'de>>(d: D) -> Result<Vector2<u32>, D::Error> {
    let s: Option<String> = Deserialize::deserialize(d)?;
    
    match s {
        Some(some) => {
            let v: Vec<&str> = some.split(',').collect();
            if v.len() == 2 {
                let v0_option = v[0].parse::<f32>();
                let v1_option = v[1].parse::<f32>();
        
                if let (Ok(v0), Ok(v1)) = (v0_option, v1_option) {
                    return Ok(Vector2::new(v0, v1).into_u32());
                }
            }
            Err(D::Error::custom(format!("Failed to parse vector {some}")))
        },
        None => {
            Err(D::Error::custom("Failed to deserialize vector2 string"))
        }
    }
}