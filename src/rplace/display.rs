use min_max::{min, max};
use speedy2d::dimen::Vector2;
use speedy2d::shape::Rectangle;
use super::canvas::Canvas;
use super::data::{DAY_1_START_TIMESTAMP, DAY_2_START_TIMESTAMP, DAY_3_START_TIMESTAMP, MAX_TIMESTAMP};

#[derive(Debug)]
pub struct GraphicsHelper {
    pub canvas: Canvas,
    pub display_size: Vector2<u32>,
    pub scale_factor: f32,
}

impl GraphicsHelper {
    pub fn new(canvas: Canvas) -> GraphicsHelper {
        let timestamp = canvas.min_timestamp;
        GraphicsHelper {
            canvas,

            // default values
            display_size: Vector2::ZERO,
            scale_factor: 0.0,
        }
    }
}

impl GraphicsHelper {
    pub fn prev_nth_pixel_change(&mut self, n: u64) {
        let (x1, x2, y1, y2) = self.pixel_index_bounds_2d();
        self.canvas.prev_nth_pixel_change(n as usize, x1, x2, y1, y2);
    }

    pub fn next_nth_pixel_change(&mut self, n: u64) {
        let (x1, x2, y1, y2) = self.pixel_index_bounds_2d();

        let mut old_history_idx = vec![vec![0usize; 2000]; 2000];
        for y in y1..y2 {
            for x in x1..x2 {
                old_history_idx[y][x] = self.canvas.pixels[y][x].datapoint_history_idx;
            }
        }
        
        unsafe {
            self.canvas.next_nth_pixel_change_fast(n as usize, x1, x2, y1, y2);
        }
        self.canvas.next_nth_pixel_change(n as usize, x1, x2, y1, y2);

        let mut changes = 0;
        for y in y1..y2 {
            for x in x1..x2 {
                let pixel = self.canvas.pixels[y][x];
                let timestamp = self.canvas.dataset.datapoint_timestamp_with_xy_and_idx(x as u32, y as u32, pixel.datapoint_history_idx as u32);
                if timestamp == self.canvas.timestamp {
                    println!("Latest datapoint - (x, y)=({}, {}) | timestamp: {} | history_idx: {}",
                        x, y, self.canvas.timestamp, pixel.datapoint_history_idx);
                }

                changes += self.canvas.pixels[y][x].datapoint_history_idx - old_history_idx[y][x];
            }
        }
        println!("Detected {} changes", changes);
    }

    pub fn adjust_timestamp(&mut self, delta: i64) {
        let mut new_timestamp = self.canvas.timestamp as i64 + delta;
        if new_timestamp > self.canvas.max_timestamp as i64 {
            new_timestamp = self.canvas.max_timestamp as i64;
        }
        if new_timestamp < self.canvas.min_timestamp as i64  {
            new_timestamp = self.canvas.min_timestamp as i64;
        }
        
        let (x1, x2, y1, y2) = self.pixel_index_bounds_2d();
        self.canvas.adjust_timestamp(new_timestamp, x1, x2, y1, y2);
    }

    pub fn adjust_timestamp_to_day(&mut self, day: u32) {
        let new_timestamp: i64;
        match day {
            0 => new_timestamp = DAY_1_START_TIMESTAMP as i64 - 1,
            1 => new_timestamp = DAY_1_START_TIMESTAMP as i64,
            2 => new_timestamp = DAY_2_START_TIMESTAMP as i64,
            3 => new_timestamp = DAY_3_START_TIMESTAMP as i64,
            4 => new_timestamp = MAX_TIMESTAMP as i64,
            _ => return,
        }

        let (x1, x2, y1, y2) = self.pixel_index_bounds_2d();
        self.canvas.adjust_timestamp(new_timestamp, x1, x2, y1, y2);
    }

    pub fn display_width(&self) -> u32 {
        return self.display_size.x;
    }

    pub fn display_height(&self) -> u32 {
        return self.display_size.y;
    }

    pub fn display_center_coordinate(&self) -> Vector2<f32> {
        return Vector2::new(
            self.display_size.x as f32 / 2.0,
            self.display_size.y as f32 / 2.0
        );
    }
    
    pub fn bound_point_by_display(&self, point: &mut Vector2<f32>) {
        if point.x < 0.0 {
            point.x = 0.0;
        }

        if point.x >= self.display_width() as f32 {
            point.x = (self.display_width() - 1) as f32;
        }
        
        if point.y < 0.0 {
            point.y = 0.0;
        }

        if point.y >= self.display_height() as f32 {
            point.y = (self.display_height() - 1) as f32;
        }
    }

    pub fn is_rect_visible(&self, rect: &Rectangle<f32>) -> bool {
        if rect.bottom_right().x < 0.0 || 
            rect.bottom_right().y < 0.0 ||
            rect.top_left().x >= self.display_width() as f32 || 
            rect.top_left().y >= self.display_height() as f32 {
            return false;
        }
        return true;
    }

    pub fn pixel_index_bounds(&self, screen_location_start: f32, canvas_length: f32, display_length: f32) -> (usize, usize) {
        let x1 = f32::max(
            0.0, 
            -1.0 * screen_location_start / self.canvas.pixel_size
        );
        let x2 = min!(
            canvas_length as usize,
            (x1 + (display_length - f32::max(0.0, screen_location_start)) / self.canvas.pixel_size).ceil() as usize
        );

        let x1_final = f32::min(x1, canvas_length).floor() as usize;
        let x2_final = max(x1_final, x2);
        return (x1_final, x2_final);
    }

    pub fn pixel_index_bounds_2d(&self) -> (usize, usize, usize, usize) {
        let (x1, x2) = self.pixel_index_bounds(
            self.canvas.top_left.x, 
            self.canvas.width() as f32, 
            self.display_width() as f32
        );
        
        let (y1, y2) = self.pixel_index_bounds(
            self.canvas.top_left.y, 
            self.canvas.height() as f32, 
            self.display_height() as f32
        );

        return (x1, x2, y1, y2);
    }

    pub fn num_rectangles_to_redraw(&self) -> u32 {
        let (x1, x2, y1, y2) = self.pixel_index_bounds_2d();
        return ((x2 - x1) * (y2 - y1)) as u32;
    }

    pub fn get_rectangle_for_idx() {
        todo!()
    }
}
