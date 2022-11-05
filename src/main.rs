mod rplace;
use std::{thread, time::Duration};

use speedy2d::Window;
use rplace::{window::RedditPlaceWindowHandler, data::RPlaceDataReader, search::RPlaceDataset};

fn run_visualizer(file_path: &str) {
    let window = Window::new_fullscreen_borderless("R/Place Renderer").unwrap();
    window.run_loop(RedditPlaceWindowHandler::new(file_path));
}

fn iterate_data(file_path: &str) {
    println!("Iterating data");

    let size = 2000;
    let mut history = 0;
    let mut dataset = RPlaceDataset::empty(size);
    if let Some(reader) = RPlaceDataReader::new(file_path){
        let iterator = reader.into_iter();
        for datapoint in iterator {
            if history % 1000000 == 0 {
                println!("At {} lines", history);
            }

            dataset.add(datapoint, datapoint.coordinate.x as usize, datapoint.coordinate.y as usize);
            history += 1;
        }
    }
    
    println!("Sleeping - History size: {}", history);
    thread::sleep(Duration::from_secs(30));
}

fn main() {
    let file_path = "/Users/michaelqian/Projects/rplace/data/parquet/2022_place_deephaven.parquet";
    //run_visualizer(file_path);
    iterate_data(file_path);
}
