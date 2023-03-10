mod color;
mod point;

use std::cmp::{max, min};
use std::io;
use std::iter::zip;

pub use crate::color::*;
pub use crate::point::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn zero() -> Self {
        Self { width: 0, height: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    top_left: Point,
    size: Size,
}

pub struct RectIterator {
    rect: Rect,
    current: Point,
}

impl Rect {
    pub fn from_point_and_size(
        top_left: Point,
        size: Size,
    ) -> Self {
        Self {
            top_left,
            size,
        }
    }

    pub fn zero() -> Self {
        Self {
            top_left: Point::zero(),
            size: Size::zero(),
        }
    }

    pub fn from_points(
        p1: Point,
        p2: Point,
    ) -> Self {
        let top_left = Point {
            x: min(p1.x, p2.x),
            y: min(p1.y, p2.y),
        };

        let bottom_right = Point {
            x: max(p1.x, p2.x),
            y: max(p1.y, p2.y),
        };

        let size = Size {
            width: (bottom_right.x - top_left.x) as u32,
            height: (bottom_right.y - top_left.y) as u32,
        };

        Self { top_left, size }
    }

    pub fn top_left(&self) -> Point {
        self.top_left
    }

    pub fn top_right(&self) -> Point {
        Point {
            x: self.top_left.x + self.size.width as i32,
            y: self.top_left.y,
        }
    }

    pub fn bottom_left(&self) -> Point {
        Point {
            x: self.top_left.x,
            y: self.top_left.y + self.size.height as i32,
        }
    }

    pub fn bottom_right(&self) -> Point {
        Point {
            x: self.top_left.x + self.size.width as i32,
            y: self.top_left.y + self.size.height as i32,
        }
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn left(&self) -> i32 {
        self.top_left.x
    }

    pub fn right(&self) -> i32 {
        self.top_left.x + self.size.width as i32
    }

    pub fn top(&self) -> i32 {
        self.top_left.y
    }

    pub fn bottom(&self) -> i32 {
        self.top_left.y + self.size.height as i32
    }

    pub fn intersected(&self, other: &Rect) -> Option<Rect> {
        let left = max(self.left(), other.left());
        let right = min(self.right(), other.right());
        let top = max(self.top(), other.top());
        let bottom = min(self.bottom(), other.bottom());

        if left < right && top < bottom {
            Some(Rect::from_points(
                Point { x: left, y: top },
                Point { x: right, y: bottom },
            ))
        } else {
            None
        }
    }

    pub fn iter(&self) -> RectIterator {
        RectIterator::new(*self)
    }
}

impl RectIterator {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            current: rect.top_left(),
        }
    }
}

impl Iterator for RectIterator {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.y < self.rect.bottom() - 1 {
            let point = self.current;
            self.current = match self.current.x < self.rect.right() - 1 {
                true => Point {
                    x: self.current.x + 1,
                    y: self.current.y,
                },
                false => Point {
                    x: self.rect.left(),
                    y: self.current.y + 1,
                },
            };
            Some(point)
        } else { None }
    }
}

pub struct Bitmap {
    width: u32,
    height: u32,
    pixels: Vec<Color>,
}

pub fn ppi2ppm(ppi: u32) -> u32 {
    ((1000./254.)*(ppi as f32)) as u32
}

impl Bitmap {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![Color::new(0, 0, 0); (width*height) as usize],
        }
    }

    fn index(&self, p: Point) -> usize {
        (p.y*self.width as i32 + p.x) as usize
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn size(&self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    pub fn rect(&self) -> Rect {
        Rect {
            top_left: Point { x: 0, y: 0 },
            size: self.size(),
        }
    }

    pub fn pixel(&self, p: Point) -> Color {
        let index = self.index(p);
        self.pixels[index]
    }

    pub fn put_pixel(&mut self, p: Point, color: Color) -> &mut Self {
        let index = self.index(p);
        self.pixels[index] = color;
        self
    }

    pub fn fill_rect(&mut self, rect: &Rect, color: Color) -> &mut Self {
        rect.iter().for_each(|p| {
            self.put_pixel(p, color);
        });
        self
    }

    pub fn clear(&mut self, color: Color) -> &mut Self {
        self.fill_rect(&self.rect(), color)
    }

    pub fn blit(
        &mut self,
        bitmap: &Bitmap,
        src_rect: Rect,
        dst_rect: Rect,
    ) -> &mut Self {
        let src_rect = if let Some(rect) = src_rect.intersected(&bitmap.rect()) {
            rect
        } else {
            Rect::zero()
        };

        let dst_rect = if let Some(rect) = dst_rect.intersected(&self.rect()) {
            rect
        } else {
            Rect::zero()
        };

        let size = Size {
            width: min(src_rect.width(), dst_rect.width()),
            height: min(src_rect.height(), dst_rect.height()),
        };

        zip(
            Rect::from_point_and_size(src_rect.top_left(), size).iter(),
            Rect::from_point_and_size(dst_rect.top_left(), size).iter(),
        ).for_each(|(src, dst)| {
            self.put_pixel(dst, bitmap.pixel(src));
        });

        self
    }

    pub fn write<T>(
        &self,
        writer: &mut T,
    ) -> Result<(), io::Error> where T: io::Write + io::Seek {
        // see https://en.wikipedia.org/wiki/BMP_file_format

        // write BMP file header
        writer.write_all(b"BM")?;

        let header_file_size_offset = writer.seek(io::SeekFrom::Current(0))?;

        writer.write_all(&[0; 4])?; // file size
        writer.write_all(&[0; 4])?; // reserved

        let header_pixel_array_offset = writer.seek(io::SeekFrom::Current(0))?;

        writer.write_all(&[0; 4])?; // offset of pixel array

        // write DIB header
        writer.write_all(&(40 as u32).to_le_bytes())?;   // DIB header size
        writer.write_all(&self.width.to_le_bytes())?;    // width
        writer.write_all(&self.height.to_le_bytes())?;   // height
        writer.write_all(&(1 as u16).to_le_bytes())?;    // color planes
        writer.write_all(&(24 as u16).to_le_bytes())?;   // bits per pixel
        writer.write_all(&[0; 4])?;                      // compression
        writer.write_all(&[0; 4])?;                      // image size
        writer.write_all(&ppi2ppm(300).to_be_bytes())?;  // x pixels per meter
        writer.write_all(&ppi2ppm(300).to_be_bytes())?;  // y pixels per meter
        writer.write_all(&[0; 4])?;                      // number of colors in the palette
        writer.write_all(&[0; 4])?;                      // number of important colors used

        // write offset to pixel array in the section header
        let pixel_array_offset = writer.seek(io::SeekFrom::Current(0))?;

        writer.seek(io::SeekFrom::Start(header_pixel_array_offset))?;
        writer.write_all(&(pixel_array_offset as u32).to_le_bytes())?;
        writer.seek(io::SeekFrom::Start(pixel_array_offset))?;

        // write pixel array
        let row_size = 4*(f32::ceil(3.*(self.width as f32)/4.) as u32);
        let pad_size = row_size - 3*self.width;
        for row in (0..self.height).rev() {
            let index = self.index(Point { x: 0, y: row as i32 });

            // write row
            self.pixels[index..index + self.width as usize]
                .iter()
                .map(|color| [color.blue, color.green, color.red])
                .for_each(|color| {
                    writer.write_all(&color).unwrap();
                });

            // write padding
            (0..pad_size).for_each(|_| {
                writer.write_all(&[0]).unwrap();
            });
        }

        // write file size in the section header
        let file_size = writer.seek(io::SeekFrom::Current(0))?;

        writer.seek(io::SeekFrom::Start(header_file_size_offset))?;
        writer.write_all(&(file_size as u32).to_le_bytes())?;
        writer.seek(io::SeekFrom::Start(file_size))?;

        // flush
        writer.flush()?;

        Ok(())
    }
}
