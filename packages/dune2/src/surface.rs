pub use crate::bitmap::*;
pub use crate::color::*;
pub use crate::point::*;
pub use crate::rect::*;
pub use crate::size::*;

pub struct Surface {
    size: Size,
    pixels: Vec<Color>,
}

fn index(p: Point, width: u32) -> usize {
    let x = p.x as u32;
    let y = p.y as u32;
    (y*width + x) as usize
}

impl Surface {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            pixels: vec![Color::default(); (size.width*size.height) as usize],
        }
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
    fn get_pixel(&self, p: Point) -> Color {
        let index = index(p, self.size.width);
        self.pixels[index]
    }
}

impl BitmapPutPixel for Surface {
    fn put_pixel(&mut self, p: Point, color: Color) -> &mut Self {
        let index = index(p, self.size.width);
        self.pixels[index] = color;
        self
    }
}
