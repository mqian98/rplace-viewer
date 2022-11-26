mod rplace;
use std::time::Instant;
use speedy2d::Window;
use rplace::{window::RedditPlaceWindowHandler, data::RPlaceDataReader, reader::custom::{write_data_to_file, read_data_from_compressed_file}};

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
    if let Some(reader) = RPlaceDataReader::new(file_path) {
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

fn main() {
    let file_path = "/Users/michaelqian/Projects/rplace/data/parquet/2022_place_deephaven.parquet";
    //run_visualizer(file_path, 2000);
    //iterate_data(file_path);
    //test_mmap();
    let compressed_data_file_path = "output";
    write_data_to_file(file_path, compressed_data_file_path);
    read_data_from_compressed_file(compressed_data_file_path);
}
