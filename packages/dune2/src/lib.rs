mod color;
mod point;
mod rect;
mod size;

use std::cmp::min;
use std::error::Error;
use std::io;
use std::iter::zip;

pub use crate::color::*;
pub use crate::point::*;
pub use crate::rect::*;
pub use crate::size::*;

fn ppi2ppm(ppi: u32) -> u32 {
    ((1000./254.)*(ppi as f32)) as u32
}

pub struct Surface {
    size: Size,
    pixels: Vec<Color>,
}

impl Surface {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            pixels: vec![Color::new(0, 0, 0); (size.width*size.height) as usize],
        }
    }

    fn index(&self, p: Point) -> usize {
        (p.y*self.size.width as i32 + p.x) as usize
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

    pub fn rect(&self) -> Rect {
        Rect::from_point_and_size(Point::zero(), self.size())
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
        bitmap: &Surface,
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

    pub fn write_with_palette<T>(
        &self,
        writer: &mut T,
        palette: &Palette,
    ) -> Result<(), Box<dyn Error>> where T: io::Write + io::Seek {
        // see https://en.wikipedia.org/wiki/BMP_file_format

        let palette_size = palette.len() as u32;
        let bits_per_pixel = if palette_size <= 256 { 8 } else { 24 } as u16;

        // write BMP file header
        writer.write_all(b"BM")?;

        let header_file_size_offset = writer.seek(io::SeekFrom::Current(0))?;

        writer.write_all(&[0; 4])?; // file size
        writer.write_all(&[0; 4])?; // reserved

        let header_pixel_array_offset = writer.seek(io::SeekFrom::Current(0))?;

        writer.write_all(&[0; 4])?; // offset of pixel array

        // write DIB header
        writer.write_all(&(40 as u32).to_le_bytes())?;       // DIB header size
        writer.write_all(&self.size.width.to_le_bytes())?;   // Width
        writer.write_all(&self.size.height.to_le_bytes())?;  // Height
        writer.write_all(&(1 as u16).to_le_bytes())?;        // Color planes
        writer.write_all(&bits_per_pixel.to_le_bytes())?;    // Bits per pixel
        writer.write_all(&[0; 4])?;                          // Compression
        writer.write_all(&[0; 4])?;                          // Image size
        writer.write_all(&ppi2ppm(300).to_le_bytes())?;      // X pixels per meter
        writer.write_all(&ppi2ppm(300).to_le_bytes())?;      // Y pixels per meter
        writer.write_all(&palette_size.to_le_bytes())?;      // Number of colors in the palette
        writer.write_all(&[0; 4])?;                          // Number of important colors used

        // write palette
        for (_, color) in palette.iter() {
            writer.write_all(&[color.blue, color.green, color.red, 0])?;
        }

        // write offset to pixel array in the section header
        let pixel_array_offset = writer.seek(io::SeekFrom::Current(0))?;

        writer.seek(io::SeekFrom::Start(header_pixel_array_offset))?;
        writer.write_all(&(pixel_array_offset as u32).to_le_bytes())?;
        writer.seek(io::SeekFrom::Start(pixel_array_offset))?;

        // write pixel array
        let row_size = ((bits_per_pixel as u32)*self.size.width + 31)/32*4;
        let pad_size = row_size - ((bits_per_pixel/8) as u32)*self.size.width;

        for row in (0..self.size.height).rev() {
            let index = self.index(Point { x: 0, y: row as i32 });

            // write row
            self.pixels[index..index + self.size.width as usize]
                .iter()
                .for_each(|color| {
                    if bits_per_pixel == 8 {
                        writer.write_all(&[palette.color_index(color).unwrap() as u8]).unwrap();
                    } else {
                        writer.write_all(&color.blue.to_le_bytes()).unwrap();
                        writer.write_all(&color.green.to_le_bytes()).unwrap();
                        writer.write_all(&color.red.to_le_bytes()).unwrap();
                    }
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
