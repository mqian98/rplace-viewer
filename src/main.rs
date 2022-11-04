mod rplace;
use speedy2d::Window;
use rplace::window::RedditPlaceWindowHandler;

fn main() {
    let window = Window::new_fullscreen_borderless("R/Place Renderer").unwrap();
    window.run_loop(RedditPlaceWindowHandler::new("/Users/michaelqian/Projects/rplace/data/csv/2022_place_canvas_history-000000000000.csv"));
}
