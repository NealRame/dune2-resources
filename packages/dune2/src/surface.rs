pub use crate::bitmap::*;
pub use crate::color::*;
pub use crate::point::*;
pub use crate::rect::*;
pub use crate::size::*;
pub use crate::transformations::*;

pub struct Surface {
    size: Size,
    pixels: Vec<Color>,
    transform_stack: Vec<TransformMatrix>,
    transform: TransformMatrix,
}

impl Surface {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            pixels: vec![Color::default(); (size.width*size.height) as usize],
            transform_stack: Vec::new(),
            transform: TransformMatrix::identity(),
        }
    }

    pub fn reset(&mut self) -> &mut Self {
        self.transform = TransformMatrix::identity();
        self.transform_stack.clear();
        self
    }

    pub fn restore(&mut self) -> &mut Self {
        if let Some(transform) = self.transform_stack.pop() {
            self.transform = transform;
        }
        self
    }

    pub fn save(&mut self) -> &mut Self {
        self.transform_stack.push(self.transform.clone());
        self
    }

    pub fn translate(&mut self, p: Point) -> &mut Self {
        self.transform = self.transform*TransformMatrix::translate(
            p.x as f32,
            p.y as f32,
        );
        self
    }

    pub fn rotate(&mut self, angle: f32) -> &mut Self {
        self.transform = self.transform*TransformMatrix::rotate(angle);
        self
    }

    pub fn from_bitmap_scaled<T: Bitmap + BitmapGetPixel>(
        bitmap: &T,
        scale: u32,
    ) -> Self {
        let mut surface = Surface::new(scale*bitmap.size());

        let src_rect = bitmap.rect();
        let dst_rect = surface.rect();

        blit(bitmap, &src_rect, &mut surface, &dst_rect);

        surface
    }
}

impl Bitmap for Surface {
    fn width(&self) -> u32 {
        self.size.width
    }

    fn height(&self) -> u32 {
        self.size.height
    }
}

impl BitmapGetPixel for Surface {
    fn get_pixel(&self, p: Point) -> Option<Color> {
        let p = self.transform*p;
        point_to_index(p, self.size).map(|index| self.pixels[index])
    }
}

impl BitmapPutPixel for Surface {
    fn put_pixel(&mut self, p: Point, color: Color) -> &mut Self {
        let p = self.transform*p;
        if let Some(index) = point_to_index(p, self.size) {
            self.pixels[index] = color;
        }
        self
    }
}
