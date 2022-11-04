use super::pixel::PixelColor;
use super::reader::csv::RPlaceCSVDataIterator;
use speedy2d::dimen::Vector2;

#[derive(Debug)]
pub struct Canvas {
    pub pixels: Vec<Vec<PixelColor>>,
    pub pixel_size: f32,
    pub min_pixel_size: f32,
    pub top_left: Vector2<f32>,
}

// Initialization 
impl Canvas {
    // inits a new Canvas aligned at display=(0,0) 
    pub fn new_with_pixels(pixels: Vec<Vec<PixelColor>>) -> Canvas {
        Canvas {
            pixels,

            // default these values to be specified at a later time by the caller
            pixel_size: 1.0,
            min_pixel_size: 0.5,
            top_left: Vector2::ZERO,
        }
    }

    pub fn new_with_file_path(file_path: &str) -> Option<Canvas> {
        if let Some(iter) = RPlaceCSVDataIterator::new(&file_path) {
            println!("Successfully created Reddit data iterator {:?}", iter);
            let pixels = Canvas::load_pixels(iter);

            return Some(Canvas::new_with_pixels(pixels));
        }
        return None;
    }

    fn load_pixels(iter: RPlaceCSVDataIterator) -> Vec<Vec<PixelColor>> {
        let mut pixels = vec![vec![PixelColor::Black; 2000]; 2000];
        println!("Size of pixels {:?} {:?} {:?}", std::mem::size_of_val(&pixels[0][0]), std::mem::size_of_val(&pixels[0]),  std::mem::size_of_val(&pixels));

        for record in iter {
            if let (Some(pixel_color), Some(coordinate)) = (record.pixel_color, record.coordinate) {
                pixels[coordinate.y as usize][coordinate.x as usize] = pixel_color;            
            }
        }
        return pixels;
    }
}

impl Canvas {
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