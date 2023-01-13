use rand::Rng;
use rocket::get;
use time::Instant;
use rocket::serde::{Serialize, json::Json};

use super::{canvas::Canvas, data::{DAY_1_START_TIMESTAMP, MAX_TIMESTAMP, DAY_2_START_TIMESTAMP}, pixel::PixelColor};

#[derive(Debug, Clone, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GameDisplay {
    timestamp: u64,
    top_left: (u16, u16),
    width: u16,
    height: u16,
    data: Vec<u8>,
}

impl GameDisplay {
    fn random() -> GameDisplay {
        let min = DAY_1_START_TIMESTAMP;
        let max = MAX_TIMESTAMP;
        let timestamp = rand::thread_rng().gen_range(min..max);
        
        let file_path = "data/custom/output_white";
        let mut canvas = Canvas::new_with_file_path(file_path);
        canvas.adjust_timestamp(timestamp as i64, 0, canvas.width() as usize, 0, canvas.height() as usize);

        let x_start = 0u16;
        let y_start = 0u16;
        let width = 100u16;
        let height = 100u16;
        let top_left = (x_start, y_start);

        let mut data: Vec<u8> = Vec::new();
        for row in canvas.pixels[y_start as usize .. (y_start + height) as usize].iter() {
            for pixel in row[x_start as usize .. (x_start + width) as usize].iter() {
                data.push(pixel.color as u8);
            }
        }

        GameDisplay { 
            timestamp, 
            top_left, 
            width, 
            height,
            data,
        }
    }
}

#[derive(Debug, Clone)]
struct GameInfo {
    id: String,
    last_update: Instant,
    display: GameDisplay,
}

impl GameInfo {
    fn new() -> GameInfo {
        // TODO
        GameInfo::new_with_id("test".to_string())
    }

    fn new_with_id(id: String) -> GameInfo {
        GameInfo {
            id,
            last_update: Instant::now(),
            display: GameDisplay::random(),
        }
    }
}

static mut GAMES: Vec<GameInfo> = Vec::new();

#[get("/")]
pub fn index() -> &'static str {
    "Hello, from Rocket!"
}

#[get("/<id>")]
pub fn game(id: String) -> Json<GameDisplay> {
    println!("Get game id {}", id);

    let mut current_game: Option<GameInfo> = None;
    unsafe {
        for (i, game) in GAMES.iter().enumerate() {
            match game {
                game if game.id == id => {
                    GAMES[i].last_update = Instant::now();
                    let found_game = game.clone();
                    // println!("Found match for id: {:?}", &found_game);
                    current_game = Some(found_game);
                    break;
                },
                _ => (),
            }
        }

        if current_game.is_none() {
            let new_game = GameInfo::new_with_id(id);
            // println!("New game created: {:?}", &new_game);

            GAMES.push(new_game);
            current_game = GAMES.last().cloned();
        }

        // println!("Games: {:?}", &GAMES);
        println!("Num games: {}", &GAMES.len());
    }

    let game_display = current_game.unwrap().display;
    Json(game_display)
}