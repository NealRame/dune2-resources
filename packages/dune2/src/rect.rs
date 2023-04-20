use std::cmp::{max, min};

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    top_left: Point,
    size: Size,
}

pub struct RectIterator {
    rect: Rect,
    current: u32,
    last: u32,
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
            x: self.right(),
            y: self.top(),
        }
    }

    pub fn bottom_left(&self) -> Point {
        Point {
            x: self.left(),
            y: self.bottom(),
        }
    }

    pub fn bottom_right(&self) -> Point {
        Point {
            x: self.right(),
            y: self.bottom(),
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
        } else { None }
    }

    pub fn iter(&self) -> RectIterator {
        RectIterator::new(*self)
    }
}

impl RectIterator {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            current: 0,
            last: rect.size.width*rect.size.height,
        }
    }
}

impl Iterator for RectIterator {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.last {
            let point = Point {
                x: self.rect.left() + (self.current % self.rect.width()) as i32,
                y: self.rect.top() + (self.current / self.rect.width()) as i32,
            };
            self.current += 1;
            Some(point)
        } else { None }
    }
}
