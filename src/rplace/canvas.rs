use std::time::{Instant, Duration};

use super::data::{RPlaceDataIterator, RPlaceDataReader};
use super::pixel::PixelColor;
use super::search::{RPlaceDataset, self};
use speedy2d::dimen::Vector2;

#[derive(Debug)]
pub struct Canvas {
    pub pixels: Vec<Vec<PixelColor>>,
    pub pixels_idx: Vec<Vec<usize>>,
    pub pixel_size: f32,
    pub min_pixel_size: f32,
    pub top_left: Vector2<f32>,
    pub dataset: RPlaceDataset,
    pub timestamp: u64,
    pub max_timestamp: u64,
}

// Initialization 
impl Canvas {
    pub fn empty(size: usize, default_color: PixelColor) -> Canvas {
        Canvas {
            // TODO: Replace this with the RPlaceDataset and add another matrix of current frame's indicies 
            pixels: vec![vec![default_color; size]; size],
            pixels_idx: vec![vec![0; size]; size],
            pixel_size: 1.0,
            min_pixel_size: 0.5,
            top_left: Vector2::ZERO,
            dataset: RPlaceDataset::new_with_initial_datapoint(size),
            timestamp: 0,
            max_timestamp: 0,
        }
    }

    // inits a new Canvas aligned at display=(0,0) 
    pub fn new_with_pixels(pixels: Vec<Vec<PixelColor>>) -> Canvas {
        let size = pixels.len();
        Canvas {
            pixels,
            dataset: RPlaceDataset::new_with_initial_datapoint(size),

            // default these values to be specified at a later time by the caller
            pixel_size: 1.0,
            min_pixel_size: 0.5,
            top_left: Vector2::ZERO,
            pixels_idx: vec![vec![0; size]; size],
            timestamp: 0,
            max_timestamp: 0,
        }
    }

    pub fn new_with_file_path(file_path: &str, size: usize) -> Option<Canvas> {
        if let Some(iter) = RPlaceDataReader::new(&file_path) {
            println!("Successfully created Reddit data iterator {:?}", iter);
            let mut canvas = Canvas::empty(size, PixelColor::Black);
            canvas.load_pixels(iter);

            return Some(canvas);
        }
        return None;
    }

    fn load_pixels(&mut self, reader: RPlaceDataReader) {
        // TODO: Magic number - make limit an optional parameter
        let limit = 100000;
        for (i, record) in reader.into_iter().take(limit).enumerate() {
            if i == 0 || i == limit - 1 {
                println!("Reading datapoint {}: {:?}", i, record);
            } 

            let x = record.coordinate.x as usize;
            let y = record.coordinate.y as usize;

            self.pixels[y][x] = record.color;
            self.dataset.add(record, x, y);
            self.pixels_idx[y][x] = self.dataset.data[y][x].len() - 1;
            self.timestamp = record.timestamp;
            self.max_timestamp = record.timestamp;
        }
    }
}

impl Canvas {
    pub fn adjust_timestamp(&mut self, delta: i64) {
        let start_time = Instant::now();
        let mut search_duration = Duration::from_secs(0);
        let mut new_timestamp = self.timestamp as i64 + delta;
        if new_timestamp > self.max_timestamp as i64 {
            new_timestamp = self.max_timestamp as i64;
        }
        if new_timestamp < 0 {
            new_timestamp = 0;
        }
        println!("Setting canvas timestamp from {} to {}", self.timestamp, new_timestamp);
        for y in 0..self.height() as usize {
            for x in 0..self.width() as usize {
                // TODO: optimize indices
                let end_idx = self.dataset.data[y][x].len();
                let start_time = Instant::now();
                let search_idx = match new_timestamp - (self.timestamp as i64) {
                    1_i64..=i64::MAX => self.dataset.search(new_timestamp as u64, x, y, self.pixels_idx[y][x], end_idx),
                    i64::MIN..=-1_i64 => self.dataset.search(new_timestamp as u64, x, y, 0, self.pixels_idx[y][x] + 1),
                    0 => self.pixels_idx[y][x],
                };
                search_duration += start_time.elapsed();
                let datapoint = &self.dataset.data[y][x][search_idx];
                self.pixels[y][x] = datapoint.color;
                self.pixels_idx[y][x] = search_idx;
            }
        }
        self.timestamp = new_timestamp as u64;
        let duration = start_time.elapsed();
        println!("adjust_timestamp duration: {}ms. search duration {}ms, diff {}ms", duration.as_millis(), search_duration.as_millis(), (duration - search_duration).as_millis());
    }

    pub fn display_size(&self) -> Vector2<f32> {
        Vector2::new(
            self.width() as f32 * self.pixel_size, 
            self.height() as f32 * self.pixel_size
        )
    }

    pub fn height(&self) -> u32 {
        return self.pixels.len() as u32;
    }

    pub fn width(&self) -> u32 {
        return self.pixels[0].len() as u32;
    }

    pub fn center_coordinate(&self) -> Vector2<f32> {
        return Vector2::new(
            self.top_left.x + (self.pixel_size * self.width() as f32) / 2.0,    
            self.top_left.y + (self.pixel_size * self.height() as f32) / 2.0    
        );
    }

    pub fn get_rect_bounds(&self, x: u32, y: u32) -> (Vector2<f32>, Vector2<f32>) {
        let top_left = Vector2::new(
            self.top_left.x + (x as f32 * self.pixel_size), 
            self.top_left.y + (y as f32 * self.pixel_size)
        );

        let bottom_right = Vector2::new(
            top_left.x + self.pixel_size,
            top_left.y + self.pixel_size
        );

        return (top_left, bottom_right);
    }

    // pixel_diff is positive on zoom in and negative on zoom out
    pub fn zoom(&mut self, pixel_size_diff: f32, location: Vector2<f32>) {
        // ensures the updated pixel size is always above self.min_pixel_size
        let mut new_pixel_size = self.pixel_size + pixel_size_diff;
        if new_pixel_size < self.min_pixel_size {
            new_pixel_size = self.min_pixel_size;
        }

        let modified_pixel_size_diff = new_pixel_size - self.pixel_size;
        if modified_pixel_size_diff != 0.0 {
            // update the canvas such that the canvas zooms in/out from the specified location 
            let x_diff = (self.top_left.x - location.x) / self.pixel_size * modified_pixel_size_diff;
            let y_diff = (self.top_left.y - location.y) / self.pixel_size * modified_pixel_size_diff;
            let old_top_left = self.top_left;
            self.top_left.x += x_diff;
            self.top_left.y += y_diff;
            self.pixel_size = new_pixel_size;
            println!("Updated zoom | location=({},{}) diff=({},{}) old_top_left=({},{}) canvas_top_left=({},{})", location.x, location.y, x_diff, y_diff, old_top_left.x, old_top_left.y, self.top_left.x, self.top_left.y);
        } else {
            // unlike previous case, we need to update the pixel size first to correctly calculate
            // the center coordinate
            self.pixel_size = new_pixel_size;
            let center_coordinate = self.center_coordinate();
            
            self.top_left.x += (location.x - center_coordinate.x) / 10.0;
            self.top_left.y += (location.y - center_coordinate.y) / 10.0;
            println!("Updated zoom | location=({},{}) center=({},{}) canvas_top_left=({},{})", location.x, location.y, center_coordinate.x, center_coordinate.y, self.top_left.x, self.top_left.y);
        }

        println!("Updated pixel_size input_size_diff={} modified_size_diff={} new_pixel_size={}", pixel_size_diff, modified_pixel_size_diff, new_pixel_size); 
    }
}