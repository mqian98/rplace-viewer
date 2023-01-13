pub mod pixel;
pub mod reader;
pub mod window;
pub mod canvas;
pub mod search;
pub mod dataset;
pub mod display;
pub mod data;
pub mod api;

use speedy2d::Window;
use window::RedditPlaceWindowHandler;

pub fn run_visualizer(file_path: &str, size: usize) {
    let window = Window::new_fullscreen_borderless("R/Place Renderer").unwrap();
    window.run_loop(RedditPlaceWindowHandler::new(file_path, size));
}

pub fn main() {
    let file_path = "data/custom/output_white";
    run_visualizer(file_path, 2000);
}
