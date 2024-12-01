use image::{Rgb, RgbImage};

use dune2_assets::prelude::{
    Color,
    Bitmap,
    BitmapPutPixel,
    Point,
    Size,
};


pub struct BMPImage {
    pub buffer: RgbImage,
    background_color: Color
}

impl BMPImage {
    fn new(size: Size, key_color: Color) -> Self {
        Self {
            buffer: RgbImage::new(size.width, size.height),
            background_color: key_color,
        }
    }

    pub fn save<P>(
        &self,
        path: P,
    ) -> Result<(), image::ImageError>
        where P: AsRef<std::path::Path> {
        self.buffer.save_with_format(path, image::ImageFormat::Bmp)
    }
}

impl Bitmap for BMPImage {
    fn width(&self) -> u32 {
        self.buffer.width()
    }

    fn height(&self) -> u32 {
        self.buffer.height()
    }
}

impl BitmapPutPixel for BMPImage {
    fn put_pixel(&mut self, p: Point, color: Option<Color>) -> &mut Self {
        if p.x < 0 || (p.x as u32) >= self.width() {
            return self;
        }

        if p.y < 0 || (p.y as u32) >= self.height() {
            return self;
        }

        match color {
            Some(color) => self.buffer.put_pixel(p.x as u32, p.y as u32, Rgb([
                color.red,
                color.green,
                color.blue,
            ])),
            None => self.buffer.put_pixel(p.x as u32, p.y as u32, Rgb([
                self.background_color.red,
                self.background_color.green,
                self.background_color.blue,
            ])),
        };

        self
    }
}

pub struct BMPImageBuilder {
    size: Size,
    background_color: Color,
}

impl BMPImageBuilder {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            background_color: Color {
                red: 0,
                green: 0,
                blue: 0,
            },
        }
    }

    pub fn with_background_color(
        &mut self,
        background_color: Color,
    ) -> &mut Self {
        self.background_color = background_color;
        self
    }

    pub fn build(&self) -> BMPImage {
        BMPImage::new(self.size, self.background_color)
    }
}
