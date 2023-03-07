use std::cmp::{max, min};
use std::iter::zip;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(u8, u8, u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    x: u32,
    y: u32,
}

impl Point {
    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    width: u32,
    height: u32,
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
            width: bottom_right.x - top_left.x,
            height: bottom_right.y - top_left.y,
        };

        Self { top_left, size }
    }

    pub fn top_left(&self) -> Point {
        self.top_left
    }

    pub fn top_right(&self) -> Point {
        Point {
            x: self.top_left.x + self.size.width,
            y: self.top_left.y,
        }
    }

    pub fn bottom_left(&self) -> Point {
        Point {
            x: self.top_left.x,
            y: self.top_left.y + self.size.height,
        }
    }

    pub fn bottom_right(&self) -> Point {
        Point {
            x: self.top_left.x + self.size.width,
            y: self.top_left.y + self.size.height,
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

    pub fn left(&self) -> u32 {
        self.top_left.x
    }

    pub fn right(&self) -> u32 {
        self.top_left.x + self.size.width
    }

    pub fn top(&self) -> u32 {
        self.top_left.y
    }

    pub fn bottom(&self) -> u32 {
        self.top_left.y + self.size.height
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
        if self.current.y < self.rect.bottom() {
            let point = self.current;
            self.current = match self.current.x < self.rect.right() {
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

impl Bitmap {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![Color(0, 0, 0); (width*height) as usize],
        }
    }

    fn index(&self, x: u32, y: u32) -> usize {
        (y*self.width + x) as usize
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
        let index = self.index(p.x, p.y);
        self.pixels[index]
    }

    pub fn put_pixel(&mut self, p: Point, color: Color) -> &mut Self {
        let index = self.index(p.x, p.y);
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
}
