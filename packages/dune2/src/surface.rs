pub use crate::bitmap::*;
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
            pixels: vec![Color::default(); (size.width*size.height) as usize],
        }
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
        point_to_index(p, self.size).map(|index| self.pixels[index])
    }
}

impl BitmapPutPixel for Surface {
    fn put_pixel(&mut self, p: Point, color: Color) -> &mut Self {
        if let Some(index) = point_to_index(p, self.size) {
            self.pixels[index] = color;
        }
        self
    }
}
