mod rplace;
use speedy2d::Window;
use rplace::{window::RedditPlaceWindowHandler, data::RPlaceDataReader};

fn run_visualizer(file_path: &str) {
    let window = Window::new_fullscreen_borderless("R/Place Renderer").unwrap();
    window.run_loop(RedditPlaceWindowHandler::new(file_path));
}

fn iterate_data(file_path: &str) {
    if let Some(reader) = RPlaceDataReader::new(file_path){
        let iterator = reader.into_iter();
        for data in iterator.take(100) {
            println!("{:?}", data);
        }
    }
}

fn main() {
    let file_path = "/Users/michaelqian/Projects/rplace/data/parquet/2022_place_deephaven.parquet";
    run_visualizer(file_path);
    //iterate_data(file_path);
}
