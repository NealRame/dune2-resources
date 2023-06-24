pub use crate::color::*;
pub use crate::point::*;
pub use crate::rect::*;
pub use crate::size::*;

pub trait Bitmap {
    fn width(&self) -> u32;
    fn height(&self) -> u32;

    fn size(&self) -> Size {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }

    fn rect(&self) -> Rect {
        Rect::from_point_and_size(Point::zero(), self.size())
    }
}

pub trait BitmapGetPixel {
    fn get_pixel(&self, p: Point) -> Option<Color>;
}

pub trait BitmapPutPixel {
    fn put_pixel(&mut self, p: Point, color: Color) -> &mut Self;
}

pub fn fill_rect<T>(
    bitmap: &mut T,
    rect: &Rect,
    color: Color,
) where T: Bitmap + BitmapPutPixel {
    if let Some(rect) = rect.intersected(&bitmap.rect()) {
        for y in rect.top()..rect.bottom() {
            for x in rect.left()..rect.right() {
                bitmap.put_pixel(Point { x, y }, color);
            }
        }
    }
}

pub fn clear<T>(
    bitmap: &mut T,
    color: Color,
) where T: Bitmap + BitmapPutPixel {
    fill_rect(bitmap, &bitmap.rect(), color);
}

pub enum BlitSizePolicy {
    Clip,
    Stretch,
}

fn create_range_mapper(
    i_min: u32, i_max: u32,
    o_min: u32, o_max: u32,
) -> impl Fn(u32) -> u32 {
    let i_min = i_min as f32;
    let i_max = i_max as f32;
    let o_min = o_min as f32;
    let o_max = o_max as f32;
    return move |n| {
        let n = n as f32;
        ((n - i_min)/(i_max - i_min)*(o_max - o_min) + o_min) as u32
    };
}

pub fn blit<T, U>(
    src_bitmap: &T,
    src_rect: &Rect,
    dst_bitmap: &mut U,
    dst_rect: &Rect,
) where
    T: Bitmap + BitmapGetPixel,
    U: Bitmap + BitmapPutPixel,
{
    let src_rect =
        if let Some(rect) = src_rect.intersected(&src_bitmap.rect()) {
            rect
        } else {
            Rect::zero()
        };

    let dst_rect =
        if let Some(rect) = dst_rect.intersected(&dst_bitmap.rect()) {
            rect
        } else {
            Rect::zero()
        };

    let x_map = create_range_mapper(
        dst_rect.left(), dst_rect.right(),
        src_rect.left(), src_rect.right(),
    );

    let y_map = create_range_mapper(
        dst_rect.top(), dst_rect.bottom(),
        src_rect.top(), src_rect.bottom(),
    );

    for y in dst_rect.top()..dst_rect.bottom() {
        for x in dst_rect.left()..dst_rect.right() {
            let dst = Point { x, y };
            let src = Point {
                x: x_map(x) as u32,
                y: y_map(y) as u32,
            };
            dst_bitmap.put_pixel(dst, src_bitmap.get_pixel(src).unwrap());
        }
    }
}

/// 
pub fn point_to_index(
    p: Point,
    size: Size,
) -> Option<usize> {
    if p.x >= size.width || p.y >= size.height {
        return None;
    }
    Some((p.y*size.width + p.x) as usize)
}
