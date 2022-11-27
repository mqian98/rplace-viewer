use min_max::min;
use speedy2d::dimen::Vector2;
use speedy2d::shape::Rectangle;
use super::canvas::Canvas;
use super::data::{DAY_1_START_TIMESTAMP, DAY_2_START_TIMESTAMP, DAY_3_START_TIMESTAMP, MAX_TIMESTAMP};

#[derive(Debug)]
pub struct GraphicsHelper {
    pub canvas: Canvas,
    pub display_size: Vector2<u32>,
    pub scale_factor: f32,
    pub timestamp: u64,
}

impl GraphicsHelper {
    pub fn new(canvas: Canvas) -> GraphicsHelper {
        let timestamp = canvas.min_timestamp;
        GraphicsHelper {
            canvas,
            timestamp,

            // default values
            display_size: Vector2::ZERO,
            scale_factor: 0.0,
        }
    }
}

impl GraphicsHelper {
    pub fn adjust_timestamp(&mut self, delta: i64) {
        let mut new_timestamp = self.timestamp as i64 + delta;
        if new_timestamp > self.canvas.max_timestamp as i64 {
            new_timestamp = self.canvas.max_timestamp as i64;
        }
        if new_timestamp < self.canvas.min_timestamp as i64  {
            new_timestamp = self.canvas.min_timestamp as i64;
        }
        
        let (x1, x2, y1, y2) = self.pixel_index_bounds_2d();
        self.canvas.adjust_timestamp(new_timestamp, x1, x2, y1, y2);
        self.timestamp = new_timestamp as u64;
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
        self.timestamp = new_timestamp as u64;
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

    pub fn pixel_index_bounds(&self, top_left: f32, canvas_length: f32, display_length: f32) -> (usize, usize) {
        let x1 = f32::max(
            0.0, 
            -1.0 * top_left / self.canvas.pixel_size
        );
        let x2 = min!(
            canvas_length as usize,
            (x1 + display_length as f32 / self.canvas.pixel_size).ceil() as usize, 
            (x1 + top_left / self.canvas.pixel_size + canvas_length as f32).ceil() as usize
        );

        return (x1.floor() as usize, x2);
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
