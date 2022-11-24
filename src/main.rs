mod rplace;
use std::{time::Instant, fs::File, io::Write};

use memmap::Mmap;
use speedy2d::Window;
use rplace::{window::RedditPlaceWindowHandler, data::RPlaceDataReader};
use strum::IntoEnumIterator;

use crate::rplace::{search::RPlaceDatasetDatapoint, pixel::PixelColor};

pub struct RPlaceDataCounter {
    pub counter: Vec<Vec<u32>>,
}

impl RPlaceDataCounter {
    pub fn empty(size: usize) -> RPlaceDataCounter {
        let mut counter = Vec::new();
        for y in 0..size {
            let mut row = Vec::new();
            for x in 0..size {
                row.push(0);
            }
            counter.push(row);
        }

        RPlaceDataCounter { 
            counter,
        }
    }

    pub fn increment(&mut self, x: usize, y: usize) {
        self.counter[y][x] += 1;
    }
}

fn run_visualizer(file_path: &str, size: usize) {
    let window = Window::new_fullscreen_borderless("R/Place Renderer").unwrap();
    window.run_loop(RedditPlaceWindowHandler::new(file_path, size));
}

fn iterate_data(file_path: &str) {
    println!("Iterating data");

    let size = 2000;
    let mut history = 0;
    let limit = 170_000_000;
    let mut start_time = Instant::now();
    let mut found_day2 = false;
    let mut found_day3 = false;
    //let mut dataset = RPlaceDataset::empty(size);
    let mut counter = RPlaceDataCounter::empty(size);
    if let Some(reader) = RPlaceDataReader::new(file_path){
        let iterator = reader.into_iter();
        for datapoint in iterator.take(limit) {
            if history % 1000000 == 0 {
                let duration = start_time.elapsed();
                println!("Start | At {} lines | Duration - {:?}", history, duration);
                start_time = Instant::now();
            }

            if !found_day2 && datapoint.coordinate.x >= 1000.0 {
                found_day2 = true;
                println!("Day 2 | At {} lines | {:?}", history, datapoint);
            }

            if !found_day3 && datapoint.coordinate.y >= 1000.0 {
                found_day3 = true;
                println!("Day 3 | At {} lines | {:?}", history, datapoint);
            }

            //dataset.add(datapoint, datapoint.coordinate.x as usize, datapoint.coordinate.y as usize);
            counter.increment(datapoint.coordinate.x as usize, datapoint.coordinate.y as usize);
            history += 1;
        }
    }

    let mut all_counts: Vec<&u32> = counter.counter.iter().flatten().collect::<Vec<&u32>>();
    all_counts.sort_by(|&&a, &&b| b.cmp(&a));

    for (i, count) in all_counts.iter().enumerate().take(100) {
        println!("Most counts: {} {}", i, count);
    }
    //println!("Sleeping - History size: {}", history);
    //thread::sleep(Duration::from_secs(30));
}

fn test_mmap() {
    let mut datapoints = Vec::new();
    for (i, color) in PixelColor::iter().enumerate() {
        let datapoint = RPlaceDatasetDatapoint {
            timestamp: i as u64,
            user_id: i as u32,
            color, 
            is_mod: (i % 2) != 0,
        };
        datapoints.push(datapoint);
    }

    let length = datapoints.len();
    let bytes: Vec<u8> = datapoints.iter().map(|x: &RPlaceDatasetDatapoint| x.to_bytes()).flatten().collect::<Vec<u8>>();
    let size = bytes.len() / datapoints.len();
    println!("Hello, world! | datapoints {} | size {} | bytes {} | {:?}", length, size, bytes.len(), bytes);

    {
        let mut file = File::create("temp").unwrap();
        file.write_all(bytes.as_slice()).unwrap();
        file.flush().unwrap();
    }

    let file = File::open("temp").unwrap();
    let mmap1 = unsafe { Mmap::map(&file).unwrap() };
    let mmap2 = unsafe { Mmap::map(&file).unwrap() };

    for i in 0..length/2 {
        let data = mmap1.get(i*size..(i+1)*size).unwrap();
        println!("Data {} | {:?} | {:?}", i, data, RPlaceDatasetDatapoint::from_bytes(data));
    }
    
    for i in length/2..length {
        let data = mmap2.get(i*size..(i+1)*size).unwrap();
        println!("Data {} | {:?} | {:?}", i, data, RPlaceDatasetDatapoint::from_bytes(data));
    }

    for i in 0..length/2 {
        let data = mmap2.get(i*size..(i+1)*size).unwrap();
        println!("Data {} | {:?} | {:?}", i, data, RPlaceDatasetDatapoint::from_bytes(data));
    }

    for i in length/2..length {
        let data = mmap1.get(i*size..(i+1)*size).unwrap();
        println!("Data {} | {:?} | {:?}", i, data, RPlaceDatasetDatapoint::from_bytes(data));
    }

    println!("Done");
}

fn main() {
    let file_path = "/Users/michaelqian/Projects/rplace/data/parquet/2022_place_deephaven.parquet";
    //run_visualizer(file_path, 2000);
    //iterate_data(file_path);
    test_mmap();
}
