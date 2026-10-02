extern crate sdl2;

// use self::sdl2::Sdl;
// use self::sdl2::render::WindowCanvas;
use self::sdl2::pixels::Color;
use self::sdl2::rect::Point;
use self::sdl2::render::WindowCanvas;
use self::sdl2::VideoSubsystem;

use super::{HEIGHT, WIDTH};

#[allow(unused)]
pub struct Screen {
    canvas: WindowCanvas,
    frame: Vec<Color>,
}

#[allow(unused)]
impl Screen {
    pub fn new(video_context: VideoSubsystem) -> Screen {
        let canvas = video_context
            .window("Rustboy", WIDTH as u32, HEIGHT as u32)
            .position_centered()
            .opengl()
            .build()
            .unwrap()
            .into_canvas()
            .build()
            .unwrap();

        Screen {
            canvas,
            frame: vec![Color::RGB(255, 255, 255); WIDTH * HEIGHT as usize],
        }
    }

    pub fn draw(&mut self, pixels: Vec<(Point, Color)>) {
        for (point, color) in pixels.iter() {
            let x = point.x();
            let y = point.y();
            if x < 0 || y < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 {
                continue;
            }

            self.frame[y as usize * WIDTH + x as usize] = *color;
        }
    }
    pub fn present(&mut self) {
        self.canvas.set_draw_color(Color::WHITE);
        self.canvas.clear();

        for (point, color) in self.frame.iter().enumerate() {
            if *color == Color::WHITE {
                continue;
            }
            self.canvas.set_draw_color(*color);
            self.canvas
                .draw_point(Point::new((point % WIDTH) as i32, (point / WIDTH) as i32));
        }
        self.canvas.present();
    }
}
