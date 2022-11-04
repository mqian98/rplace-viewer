use csv::DeserializeRecordsIntoIter;
use serde::{Deserialize, Deserializer};
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::fs::File;
use std::hash::{Hash, Hasher};
use speedy2d::dimen::Vector2;
use time::PrimitiveDateTime;
use crate::rplace::data::RPlaceDatapoint;
use super::super::pixel::PixelColor;

pub struct RPlaceCSVDataIterator {
    file_path: String,
    iter: DeserializeRecordsIntoIter<File, RPlaceCSVDatapoint>,
}

impl fmt::Debug for RPlaceCSVDataIterator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RedditPlaceDataReader")
         .field("file_path", &self.file_path)
         .finish()
    }
}

impl RPlaceCSVDataIterator {
    pub fn new(file_path: &str) -> Option<RPlaceCSVDataIterator> {
        match File::open(file_path) {
            Ok(f) => {
                let rdr = csv::Reader::from_reader(f);
                let iter = rdr.into_deserialize();
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

impl Iterator for RPlaceCSVDataIterator {
    type Item = RPlaceCSVDatapoint;

    fn next(&mut self) -> Option<Self::Item> {
        let value: Option<Result<RPlaceCSVDatapoint, csv::Error>> = self.iter.next();
        match value {
            Some(Ok(data_point)) => Some(data_point),
            _ => None
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RPlaceCSVDatapoint {
    #[serde(deserialize_with = "primitive_date_time_from_str")]
    pub timestamp: Option<PrimitiveDateTime>,
    pub user_id: String, 
    #[serde(deserialize_with = "pixel_color_from_str")]
    pub pixel_color: Option<PixelColor>,
    #[serde(deserialize_with = "vector2_from_str")]
    pub coordinate: Option<Vector2<u32>>,
}

impl TryFrom<&RPlaceCSVDatapoint> for RPlaceDatapoint {
    type Error = ();
    fn try_from(item: &RPlaceCSVDatapoint) -> Result<Self, Self::Error> {
        if let (Some(timestamp), Some(pixel_color), Some(coordinate)) = (item.timestamp, item.pixel_color, item.coordinate) {
            let mut hasher = DefaultHasher::new();
            item.user_id.hash(&mut hasher);
            return Ok(RPlaceDatapoint { 
                timestamp: timestamp.assume_utc().unix_timestamp() as u64, 
                user_id: hasher.finish() as u32, 
                color: pixel_color, 
                coordinate: (coordinate.x as u16, coordinate.y as u16), 
                is_mod: false, 
            });
        }

        Err(())
    }
}

fn primitive_date_time_from_str<'de, D: Deserializer<'de>>(d: D) -> Result<Option<PrimitiveDateTime>, D::Error> {
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
                return Ok(Some(value))
            }

            if let Ok(value) = PrimitiveDateTime::parse(&datetime, &format2) {
                return Ok(Some(value))
            }

            println!("Failed to parse string to PrimitiveDateTime: string={}", datetime);
            Ok(None)
        },
        None => {
            println!("Failed to deserialize datetime string");
            Ok(None)
        },
    }
}

fn pixel_color_from_str<'de, D: Deserializer<'de>>(d: D) -> Result<Option<PixelColor>, D::Error> {
    let s: Option<String> = Deserialize::deserialize(d)?;
    match s {
        Some(hex_string) => {
            match PixelColor::try_from(&hex_string) {
                Ok(value) => Ok(Some(value)),
                Err(_) => {
                    println!("Failed to match hex with pixel color: {:?}", hex_string);
                    Ok(None)
                },
            }
        },
        None => {
            println!("Failed to deserialize hex color string");
            Ok(None)
        },
    }
}

fn vector2_from_str<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Vector2<u32>>, D::Error> {
    let s: Option<String> = Deserialize::deserialize(d)?;
    
    match s {
        Some(some) => {
            let v: Vec<&str> = some.split(',').collect();
            if v.len() == 2 {
                let v0_option = v[0].parse::<f32>();
                let v1_option = v[1].parse::<f32>();
        
                if let (Ok(v0), Ok(v1)) = (v0_option, v1_option) {
                    return Ok(Some(Vector2::new(v0, v1).into_u32()));
                }
            }
            println!("Failed to parse vector {:?}", some);
            Ok(None)
        },
        None => {
            println!("Failed to deserialize vector2 string");
            Ok(None)
        }
    }
}