use crate::prelude::{
    Point,
    Size,
};


///
pub fn point_to_index(
    p: Point,
    size: Size,
) -> Option<usize> {
    if p.x < 0 && (p.x as u32) >= size.width {
        return None;
    }
    if p.y < 0 && (p.y as u32) >= size.height {
        return None;
    }
    Some(((p.y as u32)*size.width + (p.x as u32)) as usize)
}
