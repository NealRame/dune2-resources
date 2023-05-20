use std::cmp::min;
use std::iter::zip;

pub use crate::color::*;
pub use crate::point::*;
pub use crate::rect::*;
pub use crate::size::*;

pub trait Bitmap {
    fn width(&self) -> u32;
    fn height(&self) -> u32;

    fn get_pixel(&self, p: Point) -> Color;
    fn put_pixel(&mut self, p: Point, color: Color) -> &mut Self;

    fn size(&self) -> Size {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }

    fn rect(&self) -> Rect {
        Rect::from_point_and_size(Point::zero(), self.size())
    }

    fn fill_rect(&mut self, rect: &Rect, color: Color) -> &mut Self {
        if let Some(rect) = rect.intersected(&self.rect()) {
            for y in rect.top()..rect.bottom() {
                for x in rect.left()..rect.right() {
                    self.put_pixel(Point { x, y }, color);
                }
            }
        }
        self
    }

    fn clear(&mut self, color: Color) -> &mut Self {
        self.fill_rect(&self.rect(), color);
        self
    }

    fn blit(
        &mut self,
        bitmap: &impl Bitmap,
        src_rect: &Rect,
        dst_rect: &Rect,
    ) -> &mut Self {
        let src_rect =
            if let Some(rect) = src_rect.intersected(&bitmap.rect()) {
                rect
            } else {
                Rect::zero()
            };

        let dst_rect =
            if let Some(rect) = dst_rect.intersected(&self.rect()) {
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
            self.put_pixel(dst, bitmap.get_pixel(src));
        });

        self
    }
}
