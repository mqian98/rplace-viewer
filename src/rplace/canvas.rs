use std::collections::{HashMap, VecDeque};
use std::collections::hash_map::Keys;
use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver, self};
use std::thread;
use std::time::{Instant, Duration};

use super::pixel::PixelColor;
use super::reader::custom::SerializedDataset;
use libm::log2;
use min_max::{max, min};
use speedy2d::dimen::Vector2;

#[derive(Debug, Clone, Copy)]
pub struct CanvasPixel {
    pub color: PixelColor,
    pub datapoint_history_idx: usize,
    pub timestamp: u64,
}

impl CanvasPixel {
    pub fn new(color: PixelColor, timestamp: u64) -> CanvasPixel {
        CanvasPixel {
            color,
            datapoint_history_idx: 0,
            timestamp,
        }
    }

    pub fn new_with_color(color: PixelColor) -> CanvasPixel {
        CanvasPixel {
            color,
            datapoint_history_idx: 0,
            timestamp: 0,
        }
    }
}

#[derive(Debug)]
pub struct Canvas {
    pub pixels: Vec<Vec<CanvasPixel>>,
    pub pixel_size: f32,
    pub min_pixel_size: f32,
    pub top_left: Vector2<f32>,
    pub dataset: SerializedDataset,
    pub min_timestamp: u64,
    pub max_timestamp: u64,
    pub timestamp: u64,
}

// Initialization 
impl Canvas {
    pub fn new_with_file_path(file_path: &str) -> Canvas {
        let dataset = SerializedDataset::new(file_path);
        let min_timestamp = dataset.metadata.min_timestamp;
        let max_timestamp = dataset.metadata.max_timestamp;
        let default_pixel = CanvasPixel::new(PixelColor::White, min_timestamp);

        println!("Creating canvas | min_timestamp {} | max_timestamp {} | default_pixel {:?}", 
            min_timestamp, max_timestamp, default_pixel);

        Canvas {
            // TODO: Replace this with the RPlaceDataset and add another matrix of current frame's indicies 
            pixels: vec![vec![default_pixel; dataset.metadata.canvas_width as usize]; dataset.metadata.canvas_height as usize],
            pixel_size: 1.0,
            min_pixel_size: 0.5,
            top_left: Vector2::ZERO,
            dataset,
            min_timestamp,
            max_timestamp,
            timestamp: min_timestamp,
        }
    }
}

impl Canvas {
    pub fn prev_nth_pixel_change(&mut self, n: usize, x1: usize, x2: usize, y1: usize, y2: usize) {
        let start_time = Instant::now();
        let mut timestamps = Vec::new();

        for y in y1..y2 {
            for x in x1..x2 {
                let current_timestamp = self.pixels[y][x].timestamp;
                let mut prev_datapoint_history_idx = self.pixels[y][x].datapoint_history_idx as i32;
                let mut timestamp = self.dataset.datapoint_timestamp_with_xy_and_idx(x as u32, y as u32, prev_datapoint_history_idx as u32);

                if timestamp == current_timestamp {
                    prev_datapoint_history_idx -= 1;
                }

                if prev_datapoint_history_idx < 0 {
                    continue;
                }
                
                let last_idx = prev_datapoint_history_idx as u32 + 1;
                let first_idx = max!(0, last_idx as i32 - n as i32) as u32;
                for idx in first_idx..last_idx {
                    timestamp = self.dataset.datapoint_timestamp_with_xy_and_idx(x as u32, y as u32, idx);

                    if timestamp < current_timestamp {
                        timestamps.push(timestamp);
                    }
                }
            }
        }

        if timestamps.is_empty() {
            println!("Keeping timestamp the same");
            return;
        }

        let prev_timestamp = if timestamps.len() <= n {
            *timestamps.iter().min().unwrap()
        } else {
            let idx = timestamps.len() - n;
            //floydrivest::nth_element(&mut timestamps, idx, &mut Ord::cmp);
            timestamps.sort();
            timestamps[idx]
        };

        println!("Found prev nth pixel: n={} timestamp={} | duration: {:?}", n, prev_timestamp, start_time.elapsed());
        self.adjust_timestamp(prev_timestamp as i64, x1, x2, y1, y2);
    }

    pub fn prev_nth_pixel_change_low_mem(&mut self, n: usize, x1: usize, x2: usize, y1: usize, y2: usize) {
        let start_time = Instant::now();
        let mut timestamps = Vec::with_capacity(2*n);

        for y in y1..y2 {
            for x in x1..x2 {
                let current_timestamp = self.pixels[y][x].timestamp;
                let mut prev_datapoint_history_idx = self.pixels[y][x].datapoint_history_idx as i32;
                let mut timestamp = self.dataset.datapoint_timestamp_with_xy_and_idx(x as u32, y as u32, prev_datapoint_history_idx as u32);

                if timestamp == current_timestamp {
                    prev_datapoint_history_idx -= 1;
                }

                if prev_datapoint_history_idx < 0 {
                    continue;
                }
                
                let last_idx = prev_datapoint_history_idx as u32 + 1;
                let first_idx = max!(0, last_idx as i32 - n as i32) as u32;
                for idx in first_idx..last_idx {
                    timestamp = self.dataset.datapoint_timestamp_with_xy_and_idx(x as u32, y as u32, idx);

                    if timestamp < current_timestamp {
                        timestamps.push(timestamp);
                    }
                }

                timestamps.sort_by(|a, b| b.cmp(a));
                timestamps.truncate(n);
            }
        }

        if timestamps.is_empty() {
            println!("Keeping timestamp the same");
            return;
        }

        let prev_timestamp = timestamps[n-1];
        println!("Found prev nth pixel: n={} timestamp={} | duration: {:?}", n, prev_timestamp, start_time.elapsed());
        self.adjust_timestamp(prev_timestamp as i64, x1, x2, y1, y2);
    }

    pub fn next_nth_pixel_change(&mut self, n: usize,  x1: usize, x2: usize, y1: usize, y2: usize) {
        let start_time = Instant::now();
        let mut timestamps = Vec::with_capacity(2*n);

        for y in y1..y2 {
            for x in x1..x2 {
                let max_datapoint_history_idx = self.dataset.datapoint_history_len(x as u32, y as u32);
                let next_datapoint_history_idx = self.pixels[y][x].datapoint_history_idx + 1;
                let next_nth_datapoint_history_idx = next_datapoint_history_idx + n;
                for idx in next_datapoint_history_idx..min!(next_nth_datapoint_history_idx, max_datapoint_history_idx) {
                    let timestamp = self.dataset.datapoint_timestamp_with_xy_and_idx(x as u32, y as u32, idx as u32);
                    timestamps.push(timestamp);
                }
            }
        }

        if timestamps.is_empty() {
            println!("Keeping timestamp the same: {}", self.timestamp);
            return;
        }

        let next_timestamp = if timestamps.len() <= n {
            *timestamps.iter().max().unwrap()
        } else {
            //floydrivest::nth_element(&mut timestamps, n-1, &mut Ord::cmp);
            timestamps.sort();
            timestamps[n-1]
        };

        println!("Found next nth pixel: n={} timestamp={} | duration: {:?} | fetches={}", n, next_timestamp, start_time.elapsed(), timestamps.len());
        self.adjust_timestamp(next_timestamp as i64, x1, x2, y1, y2);
    }

    // 2x slower than normal version but uses O(2n) space instaed of O(whn) space
    pub fn next_nth_pixel_change_low_mem(&mut self, n: usize,  x1: usize, x2: usize, y1: usize, y2: usize) {
        let start_time = Instant::now();
        let mut timestamps = Vec::new();

        for y in y1..y2 {
            for x in x1..x2 {
                let max_datapoint_history_idx = self.dataset.datapoint_history_len(x as u32, y as u32);
                let next_datapoint_history_idx = self.pixels[y][x].datapoint_history_idx + 1;
                let next_nth_datapoint_history_idx = next_datapoint_history_idx + n;
                for idx in next_datapoint_history_idx..min!(next_nth_datapoint_history_idx, max_datapoint_history_idx) {
                    let timestamp = self.dataset.datapoint_timestamp_with_xy_and_idx(x as u32, y as u32, idx as u32);
                    timestamps.push(timestamp);
                }

                timestamps.sort();
                timestamps.truncate(n);
            }
        }

        if timestamps.is_empty() {
            println!("Keeping timestamp the same: {}", self.timestamp);
            return;
        }

        let next_timestamp = timestamps[n-1];
        println!("Found next nth pixel: n={} timestamp={} | duration: {:?}", n, next_timestamp, start_time.elapsed());
        self.adjust_timestamp(next_timestamp as i64, x1, x2, y1, y2);
   }

    pub fn adjust_timestamp(&mut self, timestamp: i64, x1: usize, x2: usize, y1: usize, y2: usize) {
        println!("Adjust timestamp between x={}..{} y={}..{} | t={}", x1, x2, y1, y2, timestamp);
        if x1 >= x2 || y1 >= y2 {
            println!("Skipping adjust timestamp");
            return;
        }

        let start_time = Instant::now();
        let n_threads = 8;
        let chunk = f32::ceil((y2 - y1) as f32 / n_threads as f32) as usize;
        let y_chunked_canvas: Vec<&mut [Vec<CanvasPixel>]> = self.pixels[y1..y2].chunks_mut(chunk).collect();
        let mut xy_sliced_canvas: Vec<Vec<&mut [CanvasPixel]>> = Vec::new();
        for rows in y_chunked_canvas {
            let mut slices: Vec<&mut [CanvasPixel]> = Vec::new();
            for row in rows {
                slices.push(&mut row[x1..x2]);
            }
            xy_sliced_canvas.push(slices);
        }

        type CanvasThreadOutput = (usize, Duration);
        let (tx, rx): (Sender<CanvasThreadOutput>, Receiver<CanvasThreadOutput>) = mpsc::channel();
        let dataset = Arc::new(&self.dataset);
        thread::scope(|scope| {
            for (n_th, slice) in xy_sliced_canvas.iter_mut().enumerate() {
                let thread_tx = tx.clone();
                let thread_dataset = dataset.clone();
                scope.spawn(move || {
                    let mut thread_start_time = Instant::now();

                    for (row_idx, row) in slice.into_iter().enumerate() {
                        let y = n_th * chunk + row_idx + y1;
                        for (col_idx, pixel) in row.into_iter().enumerate() {
                            let x = col_idx + x1; 
                            let current_idx = pixel.datapoint_history_idx;
                            let (start_idx, end_idx) = match timestamp - (pixel.timestamp as i64) {
                                1_i64..=i64::MAX => {
                                    let start_idx = current_idx;
                                    let end_idx = thread_dataset.datapoint_history_len(x as u32, y as u32);
                                    (start_idx, end_idx)
                                },
                                i64::MIN..=-1_i64 => {
                                    let start_idx = 0;
                                    let end_idx = current_idx + 1;
                                    (start_idx, end_idx)
                                },
                                0 => { 
                                    continue
                                },
                            };
            
                            let search_idx = thread_dataset.search(timestamp as u64, x, y, start_idx, end_idx, pixel);
                            let history_offset = thread_dataset.datapoint_history_xy_offset(x as u32, y as u32);
                            let search_datapoint = thread_dataset.datapoint_with_history_offset(history_offset, search_idx as u32);
                            
                            pixel.color = search_datapoint.color;
                            pixel.datapoint_history_idx = search_idx;
                            pixel.timestamp = timestamp as u64;
                        }
                    }

                    thread_tx.send((
                        n_th, 
                        thread_start_time.elapsed()
                    )).unwrap();
                });
            }
        });

        for _ in 0..xy_sliced_canvas.len() {
            let (thread_idx, thread_duration) = rx.recv().unwrap();
            //println!("Thread number: {:?} - Finished! - Duration {:?}", thread_idx, thread_duration);
        }

        self.timestamp = timestamp as u64;
        println!("adjust_timestamp duration: {:?}", start_time.elapsed());
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

    pub fn get_canvas_coordinates(&self, x: f32, y: f32) -> Vector2<u32> {
        let canvas_x = max!(
            0,
            ((x - self.top_left.x) / self.pixel_size).floor() as u32
        );

        let canvas_y = max!(
            0,
            ((y - self.top_left.y) / self.pixel_size).floor() as u32
        );
        
        return Vector2::new(
            canvas_x.min(self.width() - 1), 
            canvas_y.min(self.height() - 1)
        );
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
            println!("Updated zoom | location=({},{}) diff=({},{}) old_top_left=({},{}) canvas_top_left=({},{})", 
                location.x, location.y, x_diff, y_diff, old_top_left.x, old_top_left.y, self.top_left.x, self.top_left.y);
        } else {
            // unlike previous case, we need to update the pixel size first to correctly calculate
            // the center coordinate
            self.pixel_size = new_pixel_size;
            let center_coordinate = self.center_coordinate();
            
            self.top_left.x += (location.x - center_coordinate.x) / 10.0;
            self.top_left.y += (location.y - center_coordinate.y) / 10.0;
            println!("Updated zoom | location=({},{}) center=({},{}) canvas_top_left=({},{})", 
                location.x, location.y, center_coordinate.x, center_coordinate.y, self.top_left.x, self.top_left.y);
        }

        println!("Updated pixel_size input_size_diff={} modified_size_diff={} new_pixel_size={}", 
            pixel_size_diff, modified_pixel_size_diff, new_pixel_size); 
    }
}