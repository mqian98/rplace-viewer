mod rplace;
use std::time::Instant;
use speedy2d::Window;
use rplace::{window::RedditPlaceWindowHandler, data::RPlaceDataReader, reader::custom::{write_data_to_file, read_data_from_compressed_file}};

fn run_visualizer(file_path: &str, size: usize) {
    let window = Window::new_fullscreen_borderless("R/Place Renderer").unwrap();
    window.run_loop(RedditPlaceWindowHandler::new(file_path, size));
}

fn main() {
    let file_path = "data/custom/output_white";
    run_visualizer(file_path, 2000);
}
