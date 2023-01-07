use std::collections::{HashMap, VecDeque};
use std::collections::hash_map::Keys;
use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver, self};
use std::thread;
use std::time::{Instant, Duration};

use super::pixel::PixelColor;
use super::reader::custom::SerializedDataset;
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
    // TODO: need to fix this logic as it can change multiple pixels at a time
    pub fn prev_pixel_change(&mut self, x1: usize, x2: usize, y1: usize, y2: usize) {
        let start_time = Instant::now();
        let mut prev_timestamp = self.min_timestamp;
        for y in y1..y2 {
            for x in x1..x2 {
                let current_timestamp = self.pixels[y][x].timestamp;
                let mut prev_datapoint_history_idx = max!(0, self.pixels[y][x].datapoint_history_idx) as u32;
                let mut timestamp = self.dataset.datapoint_timestamp_with_xy_and_idx(x as u32, y as u32, prev_datapoint_history_idx);

                if timestamp == current_timestamp {
                    prev_datapoint_history_idx = max!(0, self.pixels[y][x].datapoint_history_idx as i32 - 1) as u32;
                    timestamp = self.dataset.datapoint_timestamp_with_xy_and_idx(x as u32, y as u32, prev_datapoint_history_idx);
                }
                
                if timestamp > prev_timestamp && timestamp < current_timestamp {
                    prev_timestamp = timestamp;
                }
            }
        }
        println!("Found prev pixel: timestamp={} | duration: {:?}", prev_timestamp, start_time.elapsed());
        self.adjust_timestamp(prev_timestamp as i64, x1, x2, y1, y2);
    }

    pub fn next_pixel_change(&mut self, x1: usize, x2: usize, y1: usize, y2: usize) {
        let start_time = Instant::now();
        let mut next_timestamp = self.max_timestamp;
        for y in y1..y2 {
            for x in x1..x2 {
                let max_datapoint_history_idx = self.dataset.datapoint_history_len(x as u32, y as u32) - 1;
                let next_datapoint_history_idx = min!(self.pixels[y][x].datapoint_history_idx + 1, max_datapoint_history_idx) as u32;
                let timestamp = self.dataset.datapoint_timestamp_with_xy_and_idx(x as u32, y as u32, next_datapoint_history_idx);
                let current_timestamp = self.pixels[y][x].timestamp;
                
                if timestamp < next_timestamp && timestamp > current_timestamp {
                    next_timestamp = timestamp;
                }
            }
        }
        println!("Found next pixel: timestamp={} | duration: {:?}", next_timestamp, start_time.elapsed());
        self.adjust_timestamp(next_timestamp as i64, x1, x2, y1, y2);
    }

    pub fn next_nth_pixel_change(&mut self, n: usize,  x1: usize, x2: usize, y1: usize, y2: usize) {
        let start_time = Instant::now();
        let mut timestamps = Vec::new();

        for y in y1..y2 {
            for x in x1..x2 {
                let max_datapoint_history_idx = self.dataset.datapoint_history_len(x as u32, y as u32) - 1;
                let next_datapoint_history_idx = self.pixels[y][x].datapoint_history_idx + 1;
                let nth_datapoint_history_idx = next_datapoint_history_idx + n;
                for idx in next_datapoint_history_idx..min!(nth_datapoint_history_idx, max_datapoint_history_idx) {
                    let timestamp = self.dataset.datapoint_timestamp_with_xy_and_idx(x as u32, y as u32, idx as u32);
                    let current_timestamp = self.pixels[y][x].timestamp;
                    
                    if timestamp > current_timestamp {
                        timestamps.push(timestamp);
                    }
                }
            }
        }

        if timestamps.is_empty() {
            println!("Keeping timestamp the same");
            return;
        }

        let next_timestamp = if timestamps.len() <= n {
            *timestamps.iter().max().unwrap()
        } else {
            floydrivest::nth_element(&mut timestamps, n-1, &mut Ord::cmp);
            timestamps[n-1]
        };

        println!("Found next nth pixel: n={} timestamp={} | duration: {:?}", n, next_timestamp, start_time.elapsed());
        self.adjust_timestamp(next_timestamp as i64, x1, x2, y1, y2);
    }

    pub fn adjust_timestamp(&mut self, timestamp: i64, x1: usize, x2: usize, y1: usize, y2: usize) {
        println!("Adjust timestamp between x={}..{} y={}..{}", x1, x2, y1, y2);
        if x1 >= x2 || y1 >= y2 {
            println!("Skipping adjust timestamp");
            return;
        }

        // variables for speed metrics
        let start_time = Instant::now();
        let mut search_iterations_lesser = 0.0;
        let mut search_iterations_greater = 0.0;
        let mut unchanged_idx_count = 0;
        let mut search_idx_deltas: HashMap<i32, u32> = HashMap::new();
        let mut search_types_count: HashMap<i32, u32> = HashMap::new();
        let mut search_color_change_count = 0;

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

        type CanvasThreadOutput = (usize, f32, f32, i32, HashMap<i32, u32>, HashMap<i32, u32>, i32, Duration);
        let (tx, rx): (Sender<CanvasThreadOutput>, Receiver<CanvasThreadOutput>) = mpsc::channel();
        let dataset = Arc::new(&self.dataset);
        thread::scope(|scope| {
            for (n_th, slice) in xy_sliced_canvas.iter_mut().enumerate() {
                let thread_tx = tx.clone();
                let thread_dataset = dataset.clone();
                scope.spawn(move || {
                    let mut thread_search_iterations_lesser = 0.0;
                    let mut thread_search_iterations_greater = 0.0;
                    let mut thread_unchanged_idx_count = 0;
                    let mut thread_idx_deltas: HashMap<i32, u32> = HashMap::new();
                    let mut thread_types_count: HashMap<i32, u32> = HashMap::new();
                    let mut thread_color_change_count = 0;
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
                                    thread_search_iterations_greater += ((end_idx - start_idx) as f32).log2();
                                    (start_idx, end_idx)
                                },
                                i64::MIN..=-1_i64 => {
                                    let start_idx = 0;
                                    let end_idx = current_idx + 1;
                                    thread_search_iterations_lesser += ((end_idx - start_idx) as f32).log2();
                                    (start_idx, end_idx)
                                },
                                0 => { 
                                    thread_unchanged_idx_count += 1;
                                    *thread_idx_deltas.entry(0).or_insert(0) += 1;
                                    continue
                                },
                            };
            
                            let (search_idx, 
                                search_datapoint,
                                types_count) = thread_dataset.search(timestamp as u64, x, y, start_idx, end_idx, pixel);

                            /* 
                            *thread_idx_deltas.entry(search_idx as i32 - current_idx as i32).or_insert(0) += 1;
                            *thread_types_count.entry(types_count).or_insert(0) += 1;
                            if pixel.color != search_datapoint.color {
                                thread_color_change_count += 1;
                            }
                            */
                            
                            pixel.color = search_datapoint.color;
                            pixel.datapoint_history_idx = search_idx;
                            pixel.timestamp = timestamp as u64;
                        }
                    }

                    thread_tx.send((
                        n_th, 
                        thread_search_iterations_lesser,
                        thread_search_iterations_greater, 
                        thread_unchanged_idx_count, 
                        thread_idx_deltas, 
                        thread_types_count, 
                        thread_color_change_count, 
                        thread_start_time.elapsed()
                    )).unwrap();
                });
            }
        });

        for _ in 0..xy_sliced_canvas.len() {
            let (thread_idx, 
                thread_search_iterations_lesser, 
                thread_search_iterations_greater, 
                thread_unchanged_idx_count, 
                thread_idx_deltas, 
                thread_types_count, 
                thread_color_change_count, 
                thread_duration) = rx.recv().unwrap();
            /*
            println!("Thread number: {:?} - Finished! - Duration {:?}", thread_idx, thread_duration);

            search_iterations_lesser += thread_search_iterations_lesser;
            search_iterations_greater += thread_search_iterations_greater;
            unchanged_idx_count += thread_unchanged_idx_count;
            search_color_change_count += thread_color_change_count;
            
            for (t, count) in thread_types_count {
                *search_types_count.entry(t).or_insert(0) += count;
            }
            for (delta, count) in thread_idx_deltas {
                *search_idx_deltas.entry(delta).or_insert(0) += count;
            }
            */
        }
        
        /* 
        let duration = start_time.elapsed();
        println!("adjust_timestamp duration: {}ms. search-lesser {}, search-greater {}, unchanged-px {}, color-change-count {} | timestamp {}", 
            duration.as_millis(), search_iterations_lesser, search_iterations_greater, unchanged_idx_count, search_color_change_count, timestamp);

        let mut search_mutated: Vec<(&i32, &u32)> = search_idx_deltas.iter().collect();
        search_mutated.sort_by(|x,y| y.1.cmp(&x.1));
        println!("Search idx deltas");
        for (delta, count) in &search_mutated[0..min(5, search_mutated.len())] {
            println!("delta: {} count: {}", delta, count);
        }
        println!("Search types counts {:?}", search_types_count);
        */

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