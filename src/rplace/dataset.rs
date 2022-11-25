use serde::{Deserialize, Serialize};
use super::{data::RPlaceDatapoint, pixel::PixelColor};

#[derive(Serialize, Deserialize, Debug)]
pub struct RPlaceDatasetDatapoint {
    pub timestamp: u64,
    pub user_id: u32, 
    pub color: PixelColor,
    
    // indicates if a pixel was placed due to moderation
    // will not be true for all pixels placed by mods
    // only is true for swaths of pixels that mods place
    pub is_mod: bool, 
}

impl From<RPlaceDatapoint> for RPlaceDatasetDatapoint {
    fn from(item: RPlaceDatapoint) -> Self {
        RPlaceDatasetDatapoint { 
            timestamp: item.timestamp, 
            user_id: item.user_id, 
            color: item.color, 
            is_mod: item.is_mod 
        }
    }
}

impl RPlaceDatasetDatapoint {
    pub fn start() -> Self {
        RPlaceDatasetDatapoint { 
            timestamp: 0, 
            user_id: 0, 
            color: PixelColor::Black, 
            is_mod: false, 
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }

    pub fn compressed_size() -> u8 {
        let datapoint = RPlaceDatasetDatapoint::start();
        datapoint.to_bytes().len() as u8
    }
}

// Data is 2d matrix. Each element is a sorted array of edits for that pixel location
#[derive(Debug)]
pub struct RPlaceDataset {
    pub data: Vec<Vec<Vec<RPlaceDatasetDatapoint>>>,
}

impl RPlaceDataset {
    // creates a dataset where the first datapoint per pixel is sentinel
    pub fn new_with_initial_datapoint(size: usize) -> RPlaceDataset {
        let mut data = Vec::new();
        for y in 0..size {
            let mut row = Vec::new();
            for x in 0..size {
                let mut vector = Vec::new();
                let datapoint = RPlaceDatasetDatapoint::start();
                vector.push(datapoint);
                row.push(vector);
            }
            data.push(row);
        }

        RPlaceDataset { 
            data,
        }
    }

    pub fn add(&mut self, datapoint: RPlaceDatasetDatapoint, x: usize, y: usize) {
        self.data[y][x].push(datapoint);
    }

    pub fn search(&self, timestamp: u64, x: usize, y: usize, start_idx: usize, end_idx: usize) -> usize {
        //println!("Searching for timestamp {} at ({}, {}) in {}..{}", timestamp, x, y, start_idx, end_idx);
        let pixel_history = &self.data[y][x][start_idx..end_idx];
        let result = pixel_history.binary_search_by(|probe| 
            probe.timestamp.cmp(&timestamp)
        );
        match result {
            Ok(value) => return value,
            Err(value) => return value - 1,
        }
    }
}