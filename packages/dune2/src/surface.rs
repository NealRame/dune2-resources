use std::cmp::min;
use std::iter::zip;

pub use crate::color::*;
pub use crate::point::*;
pub use crate::rect::*;
pub use crate::size::*;

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
}
