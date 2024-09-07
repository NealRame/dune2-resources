use serde::{Deserialize, Serialize};

use crate::prelude::*;
use crate::utils::point_to_index;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct TileAnchorPosition { left: i32, top: i32 }

/// TileAnchor is an enum used to how to place the source tile when resizing a
/// given tile.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum TileAnchor {
    Position (TileAnchorPosition),

    BottomLeft,
    BottomCenter,
    BottomRight,

    CenterLeft,
    Center,
    CenterRight,

    TopLeft,
    TopCenter,
    TopRight,
}

impl TileAnchor {
    fn to_position(
        &self,
        src: &Size,
        dst: &Size,
    ) -> TileAnchorPosition {
        let left = match self {
            Self::Position (TileAnchorPosition { left, .. }) => *left,

            Self::TopLeft |
            Self::CenterLeft => 0,
            Self::BottomLeft |

            Self::TopCenter |
            Self::Center |
            Self::BottomCenter => (dst.width as i32 - src.width as i32)/2,

            Self::TopRight |
            Self::CenterRight |
            Self::BottomRight => dst.width as i32 - src.width as i32,
        };

        let top = match self {
            Self::Position (TileAnchorPosition { top, .. }) => *top,

            Self::TopLeft |
            Self::TopCenter |
            Self::TopRight => 0,

            Self::CenterLeft |
            Self::Center |
            Self::CenterRight => (dst.height as i32 - src.height as i32)/2,

            Self::BottomLeft |
            Self::BottomCenter |
            Self::BottomRight => dst.height as i32 - src.height as i32,
        };

        TileAnchorPosition { left, top }
    }
}


/// TileTransformation is an enum to specify which transformation you want to
/// apply to a given tile.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum TileTransformation {
    FlipX,
    FlipY,
    FlipXY,
}


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tile {
    data: Box<[u8]>,
    size: Size,
}

struct TileTransformIterator<'a> {
    current_x: u32,
    current_y: u32,
    tile: &'a Tile,
    transformation: TileTransformation,
}

impl<'a> TileTransformIterator<'a> {
    pub fn new(
        tile: &'a Tile,
        transformation: TileTransformation,
    ) -> Self {
        return Self {
            current_x: 0,
            current_y: 0,
            tile,
            transformation,
        }
    }
}

impl Iterator for TileTransformIterator<'_> {
    type Item = (u8, (u32, u32));

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_y >= self.tile.size.height {
            return None
        };

        let (x, y) = match self.transformation {
            TileTransformation::FlipX => (
                self.tile.size.width - self.current_x - 1,
                self.current_y,
            ),
            TileTransformation::FlipY => (
                self.current_x,
                self.tile.size.height - self.current_y - 1,
            ),
            TileTransformation::FlipXY => (
                self.tile.size.width - self.current_x - 1,
                self.tile.size.height - self.current_y - 1,
            ),
        };
        let data = self.tile.data[(y*self.tile.size.width + x) as usize];
        let item = (data, (self.current_x, self.current_y));

        if self.current_x < self.tile.size.width - 1 {
            self.current_x += 1;
        } else {
            self.current_x  = 0;
            self.current_y += 1;
        }

        Some(item)
    }
}

impl Tile {
    pub fn new(
        data: &[u8],
        size: Size,
    ) -> Self {
        return Tile {
            data: data.into(),
            size,
        }
    }

    pub fn size(
        &self,
    ) -> Size {
        self.size
    }

    pub fn transform(
        &self,
        transform: Option<TileTransformation>,
    ) -> Tile {
        match transform {
            Some(transform) => {
                Self {
                    size: self.size,
                    data: TileTransformIterator::new(self, transform)
                        .map(|(d, _)| d)
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                }
            },
            None => self.clone()
        }
    }

    pub fn resize(
        &self,
        size: Size,
        maybe_anchor: Option<TileAnchor>,
    ) -> Tile {
        if size == self.size {
            return self.clone();
        }

        // 1. Get the anchor position
        let anchor = maybe_anchor.unwrap_or(TileAnchor::Center);
        let anchor_position = anchor.to_position(&self.size, &size);

        // 2. Get the source rect and the destination rect
        let (src_left, dst_left, width) = if anchor_position.left >= 0 {
            let left = anchor_position.left as u32;
            let width = if size.width >= left {
                u32::min(size.width - left, self.size.width)
            } else { 0 };

            (0, left as i32, width)
        } else {
            let left = anchor_position.left.abs() as u32;
            let width = if self.size.width >= left {
                u32::min(size.width, self.size.width - left)
            } else { 0 };

            (left as i32, 0, width)
        };

        let (src_top, dst_top, height) = if anchor_position.top >= 0 {
            let top = anchor_position.top as u32;
            let height = if size.height >= top {
                u32::min(size.height - top, self.size.height)
            } else { 0 };

            (0, top as i32, height)
        } else {
            let top = anchor_position.top.abs() as u32;
            let height = if self.size.height >= top {
                u32::min(size.height, self.size.height - top)
            } else { 0 };

            (top as i32, 0, height)
        };

        let src_rect = Rect::from_point_and_size(
            Point {x: src_left, y: src_top },
            Size { width, height },
        );

        let dst_rect = Rect::from_point_and_size(
            Point { x: dst_left, y: dst_top },
            Size { width, height },
        );

        // 3. Copying data from the src_rect to the dst_rect
        let mut data = vec![0u8; (size.width*size.height) as usize];

        for (src, dst) in Iterator::zip(src_rect.iter(), dst_rect.iter()) {
            match (
                point_to_index(src, self.size),
                point_to_index(dst, size),
            ) {
                (Some(src_off), Some(dst_off)) => {
                    data[dst_off] = self.data[src_off];
                },
                _ => {}
            }
        }

        Tile {
            data: data.into_boxed_slice(),
            size
        }
    }
}

pub struct TileBitmap<'a> {
    tile: &'a Tile,
    palette: &'a Palette,
    faction: Option<Dune2Faction>,
}

impl<'a> TileBitmap<'a> {
    pub fn with_resources(
        tile: &'a Tile,
        faction: Option<Dune2Faction>,
        resources: &'a Resources,
    ) -> Self {
        Self {
            faction,
            palette: &resources.palette,
            tile,
        }
    }

    pub fn with_palette(
        tile: &'a Tile,
        faction: Option<Dune2Faction>,
        palette: &'a Palette,
    ) -> Self {
        Self {
            faction,
            palette,
            tile,
        }
    }
}

impl Bitmap for TileBitmap<'_> {
    fn height(&self) -> u32 {
        self.tile.size.height
    }

    fn width(&self) -> u32 {
        self.tile.size.width
    }
}

impl BitmapGetPixel for TileBitmap<'_> {
    fn get_pixel(
        &self,
        p: Point,
    ) -> Option<Color> {
        point_to_index(p, self.size()).map(|index| {
            let mut color_index = self.tile.data[index] as usize;

            if let Some(faction) = self.faction {
                let faction_palette_offset = 16*(faction as usize);

                if color_index >= COLOR_HARKONNEN
                    && color_index < COLOR_HARKONNEN + 7 {
                    color_index = color_index + faction_palette_offset
                }
            }

            self.palette.color_at(color_index)
        })
    }
}
