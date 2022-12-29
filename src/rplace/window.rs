use min_max::min;
use speedy2d::image::{ImageDataType, ImageSmoothingMode};
use std::time::Instant;

use super::display::GraphicsHelper;
use super::pixel::PixelColor;
use std::process::exit;
use speedy2d::color::Color;
use speedy2d::dimen::{Vector2, Vec2, UVec2};
use speedy2d::shape::Rectangle;
use speedy2d::Graphics2D;
use speedy2d::window::{WindowHandler, WindowHelper, VirtualKeyCode, MouseScrollDistance};
use super::canvas::Canvas;

#[derive(Debug)]
pub struct RedditPlaceWindowHandler {
    graphics_helper: GraphicsHelper,
    mouse_position: Vector2<f32>,
    is_mouse_pressed: bool,
    realtime_redraw_rectangle_threshold: u32,
}

impl RedditPlaceWindowHandler {
    pub fn new(file_path: &str, size: usize) -> RedditPlaceWindowHandler {
        let start_time = Instant::now();

        let canvas = Canvas::new_with_file_path(file_path);

        let duration = start_time.elapsed();
        println!("RedditPlaceWindowHandler init time {:?}", duration);
        RedditPlaceWindowHandler::new_with_canvas(canvas)
    }

    pub fn new_with_canvas(canvas: Canvas) -> RedditPlaceWindowHandler {
        let graphics_helper = GraphicsHelper::new(canvas);

        RedditPlaceWindowHandler { 
            graphics_helper,
    
            // defaulting these values until the WindowHandler sets them during on_start
            mouse_position: Vector2::ZERO,
            is_mouse_pressed: false,
            realtime_redraw_rectangle_threshold: 320000,
        }
    }
}

impl WindowHandler for RedditPlaceWindowHandler
{
    fn on_start(
            &mut self, 
            _helper: &mut WindowHelper<()>, 
            info: speedy2d::window::WindowStartupInfo
        ) {
        println!("Starting r/place renderer!");
        
        self.graphics_helper.display_size = *info.viewport_size_pixels();
        self.graphics_helper.scale_factor = info.scale_factor() as f32;

        let canvas_display_size = min!(self.graphics_helper.display_width(), self.graphics_helper.display_height()) as f32;
        self.graphics_helper.canvas.top_left = Vector2::new_x((self.graphics_helper.display_width() as f32 - canvas_display_size) / 2.0);
        self.graphics_helper.canvas.pixel_size = canvas_display_size / self.graphics_helper.canvas.pixels.len() as f32; // * 2.0; // NOTE: only multiplying by 2 because at the start we only need to display 1000x1000 

        println!("display_size={:?}, scale_factor={:?}, top_left={:?}, pixel_size={:?}", self.graphics_helper.display_size, self.graphics_helper.scale_factor, self.graphics_helper.canvas.top_left, self.graphics_helper.canvas.pixel_size);
        println!("WindowHandler size {:?}", std::mem::size_of_val(self));
    }

    fn on_key_up(
            &mut self,
            helper: &mut WindowHelper<()>,
            virtual_key_code: Option<speedy2d::window::VirtualKeyCode>,
            scancode: speedy2d::window::KeyScancode
        ) {
        println!("Detected keyup event {:?} {:?}", virtual_key_code, scancode);

        match virtual_key_code {
            Some(VirtualKeyCode::Q) | Some(VirtualKeyCode::Escape) => exit(0),
            Some(VirtualKeyCode::Up) => {
                self.zoom_into_center_of_display(0.5);
                self.graphics_helper.adjust_timestamp(0);
                helper.request_redraw();
            },
            Some(VirtualKeyCode::Down) => {
                self.zoom_into_center_of_display(-0.5);
                self.graphics_helper.adjust_timestamp(0);
                helper.request_redraw();
            },
            Some(VirtualKeyCode::W) => {
                self.graphics_helper.canvas.top_left.y += self.keypress_move_distance();
                self.graphics_helper.adjust_timestamp(0);
                helper.request_redraw();
            },
            Some(VirtualKeyCode::A) => {
                self.graphics_helper.canvas.top_left.x += self.keypress_move_distance();
                self.graphics_helper.adjust_timestamp(0);
                helper.request_redraw();
            },
            Some(VirtualKeyCode::S) => {
                self.graphics_helper.canvas.top_left.y -= self.keypress_move_distance();
                self.graphics_helper.adjust_timestamp(0);
                helper.request_redraw();
            },
            Some(VirtualKeyCode::D) => {
                self.graphics_helper.canvas.top_left.x -= self.keypress_move_distance();
                self.graphics_helper.adjust_timestamp(0);
                helper.request_redraw();
            },
            Some(VirtualKeyCode::C) => {
                println!("Center coordinate = {:?}", self.graphics_helper.canvas.center_coordinate());
            },
            Some(VirtualKeyCode::M) => {
                println!("Mouse position = {:?}", self.mouse_position);
            },
            Some(VirtualKeyCode::H) => {
                println!("{:?}", self);
            },
            Some(VirtualKeyCode::J) => {
                let delta = -1_000_000_000_000;
                self.graphics_helper.adjust_timestamp(delta);
                helper.request_redraw();
            },
            Some(VirtualKeyCode::L) => {
                let delta = 1_000_000_000_000;
                self.graphics_helper.adjust_timestamp(delta);
                helper.request_redraw();
            },
            Some(VirtualKeyCode::Comma) => {
                self.graphics_helper.prev_pixel_change();
                helper.request_redraw();
            },
            Some(VirtualKeyCode::Period) => {
                self.graphics_helper.next_pixel_change();
                helper.request_redraw();
            },
            Some(VirtualKeyCode::Key0) => {
                self.graphics_helper.adjust_timestamp_to_day(0);
                helper.request_redraw();
            },
            Some(VirtualKeyCode::Key1) => {
                self.graphics_helper.adjust_timestamp_to_day(1);
                helper.request_redraw();
            },
            Some(VirtualKeyCode::Key2) => {
                self.graphics_helper.adjust_timestamp_to_day(2);
                helper.request_redraw();
            },
            Some(VirtualKeyCode::Key3) => {
                self.graphics_helper.adjust_timestamp_to_day(3);
                helper.request_redraw();
            },
            Some(VirtualKeyCode::Key4) => {
                self.graphics_helper.adjust_timestamp_to_day(4);
                helper.request_redraw();
            },
            _ => (),
        }
    }

    fn on_mouse_wheel_scroll(
            &mut self,
            helper: &mut WindowHelper<()>,
            distance: speedy2d::window::MouseScrollDistance
        ) {
        match distance {
            MouseScrollDistance::Lines { y, .. } => {
                let zoom = -1.0 * y as f32;
                println!("on_mouse_wheel_scroll {:?}", zoom);
                self.zoom_into_mouse_location(zoom); 
                self.graphics_helper.adjust_timestamp(0);
                helper.request_redraw();
            },
            _ => (),
        }
    }

    fn on_mouse_move(&mut self, helper: &mut WindowHelper<()>, position: speedy2d::dimen::Vec2) {
        //println!("on_mouse_move {:?}", position);
        if self.is_mouse_pressed {
            self.graphics_helper.canvas.top_left.x += position.x - self.mouse_position.x;
            self.graphics_helper.canvas.top_left.y += position.y - self.mouse_position.y;
            
            // only redraw on mouse drag if amount of pixels to redraw is low
            if self.graphics_helper.num_rectangles_to_redraw() < self.realtime_redraw_rectangle_threshold {
                self.graphics_helper.adjust_timestamp(0);
                helper.request_redraw();
            }
        }
        self.mouse_position = position;
    }

    fn on_mouse_button_down(
            &mut self,
            _helper: &mut WindowHelper<()>,
            button: speedy2d::window::MouseButton
        ) {
        println!("on_mouse_button_down {:?}", button);
        self.is_mouse_pressed = true;
    }

    fn on_mouse_button_up(
            &mut self,
            helper: &mut WindowHelper<()>,
            button: speedy2d::window::MouseButton
        ) {
        println!("on_mouse_button_down {:?}", button);
        self.is_mouse_pressed = false;
        self.graphics_helper.adjust_timestamp(0);
        helper.request_redraw();
    }

    fn on_draw(&mut self, _helper: &mut WindowHelper, graphics: &mut Graphics2D)
    {
        let start_time = Instant::now();
        graphics.clear_screen(Color::from_rgb(0.0, 0.0, 0.0));
        self.draw_pixels(graphics, None); //, Some(PixelColor::Black));
        let duration = start_time.elapsed();
        println!("on_draw duration: {}ms", duration.as_millis());
    }
}

impl RedditPlaceWindowHandler {
    fn keypress_move_distance(&self) -> f32 {
        return min!(self.graphics_helper.display_width(), self.graphics_helper.display_height()) as f32 / 20.0;
    }

    fn zoom_into_center_of_display(&mut self, pixel_size_diff: f32) {
        self.graphics_helper.canvas.zoom(pixel_size_diff, self.graphics_helper.display_center_coordinate());
    }

    fn zoom_into_mouse_location(&mut self, pixel_size_diff: f32) {
        self.graphics_helper.canvas.zoom(pixel_size_diff, self.mouse_position);
    }

    fn draw_pixels(&self, graphics: &mut Graphics2D, ignore_color: Option<PixelColor>) {
        let (x1, x2, y1, y2) = self.graphics_helper.pixel_index_bounds_2d();
        let x_width = x2 - x1;
        let y_height = y2 - y1;

        let total_canvas_pixels = self.graphics_helper.num_rectangles_to_redraw();
        let total_display_pixels = total_canvas_pixels as f32 * self.graphics_helper.canvas.pixel_size;
        println!("Drawing pixels between x={}..{}, y={}..{} | # canvas px: {} | # display px {} | px size {}", 
            x1, x2, y1, y2, total_canvas_pixels, total_display_pixels, self.graphics_helper.canvas.pixel_size);

        let mut image_bytes: Vec<u8> = vec![0; x_width * y_height * 3];
        println!("Image bytes size: {}, {} {}", image_bytes.len(), x_width, y_height);

        for (display_y, canvas_y) in (y1..y2).into_iter().enumerate() {
            for (display_x, canvas_x) in (x1..x2).into_iter().enumerate() {
                let color: u32 = self.graphics_helper.canvas.pixels[canvas_y][canvas_x].color.into();
                let r = (color >> 16 & 0xff) as u8;
                let g = (color >> 8 & 0xff) as u8;
                let b = (color & 0xff) as u8;

                let idx = ((display_y * x_width) + display_x) * 3;
                image_bytes[idx] = r;
                image_bytes[idx + 1] = g;
                image_bytes[idx + 2] = b;
            }
        }

        let (mut top_left, _) = self.graphics_helper.canvas.get_rect_bounds(x1 as u32, y1 as u32);
        let (_, mut bottom_right) = self.graphics_helper.canvas.get_rect_bounds(x2 as u32, y2 as u32);

        let rect = Rectangle::new(top_left, bottom_right);
        let image = graphics.create_image_from_raw_pixels(
            ImageDataType::RGB, 
            ImageSmoothingMode::NearestNeighbor, 
            UVec2::new(x_width as u32, y_height as u32), 
            image_bytes.as_slice()
        );
        
        match image {
            Ok(image) => graphics.draw_rectangle_image(rect, &image),
            Err(e) => println!("Error {:?}", e),
        }
    }
}