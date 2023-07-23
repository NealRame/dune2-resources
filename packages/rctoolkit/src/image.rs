use image::{Rgb, RgbImage};

use dune2_rc::{self as dune2, Bitmap};

pub struct BMPImage {
    pub buffer: RgbImage,
}

impl BMPImage {
    pub fn new(size: dune2::Size) -> Self {
        Self {
            buffer: RgbImage::new(size.width, size.height),
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

impl dune2::Bitmap for BMPImage {
    fn width(&self) -> u32 {
        self.buffer.width()
    }

    fn height(&self) -> u32 {
        self.buffer.height()
    }
}

impl dune2::BitmapPutPixel for BMPImage {
    fn put_pixel(&mut self, p: dune2::Point, color: dune2::Color) -> &mut Self {
        if p.x < self.width() || p.y < self.height() {
            self.buffer.put_pixel(p.x, p.y, Rgb([
                color.red,
                color.green,
                color.blue,
            ]));
        }
        self
    }
}