use std::error::{Error};
use std::io::{Read, Seek, SeekFrom};
use std::path;

use std::fs;
use std::iter;

use bitvec::prelude::*;

use serde::{Deserialize, Serialize};

use crate::constants::*;
use crate::io::*;
use crate::surface::*;

enum SHPVersion {
    V100,
    V107,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpriteFrame {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
    pub remap_table: Vec<usize>,
}

fn shp_read_version<T>(reader: &mut T)
    -> Result<SHPVersion, Box<dyn Error>>
    where T: Read + Seek
{
    reader.seek(SeekFrom::Start(4))?;

    let mut buf = [0; 2];
    reader.read_exact(&mut buf)?;

    let version = match u16::from_le_bytes(buf) {
        0 => SHPVersion::V107,
        _ => SHPVersion::V100,
    };

    reader.seek(SeekFrom::Start(0))?;

    Ok(version)
}

fn shp_read_frame_count<T>(reader: &mut T)
    -> Result<usize, Box<dyn Error>>
    where T: Read + Seek
{
    let mut buf = [0; 2];
    reader.read_exact(&mut buf)?;

    Ok(u16::from_le_bytes(buf) as usize)
}

fn shp_read_frame_offset_v100<T>(
    reader: &mut T,
) -> Result<u64, Box<dyn Error>> where T: Read + Seek {
    let mut buf = [0; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf) as u64)
}

fn shp_read_frame_offset_v107<T>(
    reader: &mut T,
) -> Result<u64, Box<dyn Error>> where T: Read + Seek {
    let mut buf = [0; 4];
    reader.read_exact(&mut buf)?;
    Ok((u32::from_le_bytes(buf) + 2) as u64)
}

fn shp_read_frame_offsets<T>(
    reader: &mut T,
    version: SHPVersion,
) -> Result<Vec<(u64, usize)>, Box<dyn Error>> where T: Read + Seek {
    let frame_count = shp_read_frame_count(reader)?;
    let mut offsets = Vec::with_capacity(frame_count);

    for _ in 0..=frame_count {
        offsets.push(match version {
            SHPVersion::V100 => shp_read_frame_offset_v100(reader)?,
            SHPVersion::V107 => shp_read_frame_offset_v107(reader)?,
        });
    }

    Ok(offsets
        .windows(2)
        .map(|offsets| (offsets[0], (offsets[1] - offsets[0]) as usize))
        .collect()
    )
}

fn copy_block(data: &mut Vec<u8>, count: usize, pos: usize, relative: bool) {
    let offset = if relative { data.len() - pos } else { pos };
    for i in 0..count {
        data.push(data[offset + i]);
    }
}

fn inflate_lcw_data<T: Read + Seek>(
    reader: &mut T,
    output: &mut Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    let relative = u8::try_read_from::<LSB>(reader)? == 0;

    if !relative {
        reader.seek(SeekFrom::Current(-1))?;
    }

    loop { match u8::try_read_from::<LSB>(reader)? {
        0x80 => break,
        cmd if (cmd & 0xc0) == 0x80 => {
            // command 1: short copy
            // 0b10cccccc
            let count = (cmd & 0x3f) as usize;
            let pos = output.len();

            output.resize(output.len() + count, 0);
            reader.read_exact(&mut output[pos..])?;
        },
        cmd if (cmd & 0x80) == 0 => {
            // command 2: existing block relative copy
            // 0b0cccpppp p
            let count = (((cmd & 0x70) as usize) >> 4) + 3;
            let pos   = (((cmd & 0x0f) as usize) << 8) | u8::try_read_from::<LSB>(reader)? as usize;

            if pos == 1 {
                output.extend(iter::repeat(*output.last().unwrap()).take(count));
            } else {
                copy_block(output, count, pos, true);
            }
        },
        0xfe => {
            // command 4: repeat value
            // 0b11111110 c c v
            let count = u16::try_read_from::<LSB>(reader)? as usize;
            let value = u8::try_read_from::<LSB>(reader)?;

            output.extend(iter::repeat(value).take(count));
        },
        0xff => {
            // command 5: existing block long copy
            // 0b11111111 c c p p
            let count = u16::try_read_from::<LSB>(reader)? as usize;
            let pos   = u16::try_read_from::<LSB>(reader)? as usize;

            copy_block(output, count, pos, relative);
        },
        cmd => {
            // command 3: existing block medium-length copy
            // 0b11cccccc p p
            let count = ((cmd & 0x3f) + 3) as usize;
            let pos   = u16::try_read_from::<LSB>(reader)? as usize;

            copy_block(output, count, pos, relative);
        },
    }}

    Ok(())
}

fn inflate_rle_zero_data(
    rle_data: &[u8],
    output: &mut Vec<u8>
) {
    let mut iter = rle_data.iter().copied();

    while let Some(value) = iter.next() {
        let count = if value == 0 {
            iter.next().unwrap()
        } else { 1 } as usize;
        output.extend(iter::repeat(value).take(count));
    }
}

fn shp_read_frame<T>(
    reader: &mut T,
    offset: u64,
    size: u64,
) -> Result<SpriteFrame, Box<dyn Error>> where T: Read + Seek {

    const HAS_REMAP_TABLE: usize = 0;
    const NO_LCW: usize = 1;
    const CUSTOM_SIZE_REMAP: usize = 2;

    reader.seek(SeekFrom::Start(offset))?;

    let mut flags: BitArr!(for 16, in u8, Lsb0) = BitArray::<_, _>::ZERO;

    reader.read_exact(&mut flags.as_raw_mut_slice())?;
    
    let slices = u8::try_read_from::<LSB>(reader)? as u16;
    let width = u16::try_read_from::<LSB>(reader)? as u16;
    let height = u8::try_read_from::<LSB>(reader)? as u16;

    if slices != height {
        return Err(format!(
            "slices({}) != height({})",
            slices,
            height
        ).into());
    }

    let frame_size = u16::try_read_from::<LSB>(reader)? as usize;

    if size != frame_size as u64 {
        return Err(format!(
            "frame_size({}) != size({})",
            frame_size,
            size
        ).into());
    }

    let rle_data_size = u16::try_read_from::<LSB>(reader)? as usize;

    let remap_table_size = if flags[HAS_REMAP_TABLE] {
        if flags[CUSTOM_SIZE_REMAP] {
            u8::try_read_from::<LSB>(reader)?
        } else { 16 }
    } else { 0 } as usize;

    let remap_table = (0..remap_table_size)
        .map(|_| u8::try_read_from::<LSB>(reader).map(usize::from))
        .collect::<Result<Vec<_>, _>>()?;

    let mut rle_data = Vec::with_capacity(rle_data_size);

    if flags[NO_LCW] {
        rle_data.resize(rle_data_size, 0);
        reader.read_exact(&mut rle_data)?;
    } else {
        inflate_lcw_data(reader, &mut rle_data)?
    }

    let mut data = Vec::new();
    inflate_rle_zero_data(&rle_data, &mut data);

    Ok(SpriteFrame {
        width,
        height,
        remap_table,
        data,
    })
}

pub struct SpriteFrameBitmap<'a, 'b> {
    frame: &'a SpriteFrame,
    palette: &'b Palette,
    faction_palette_offset: usize,
}

impl<'a, 'b> SpriteFrameBitmap<'a, 'b> {
    fn new(
        frame: &'a SpriteFrame,
        palette: &'b Palette,
        faction: Faction,
    ) -> Self {
        Self {
            frame,
            palette,
            faction_palette_offset: 16*(faction as usize),
        }
    }
}

impl Bitmap for SpriteFrameBitmap<'_, '_> {
    fn width(&self) -> u32 {
        self.frame.width as u32
    }

    fn height(&self) -> u32 {
        self.frame.height as u32
    }
}

impl BitmapGetPixel for SpriteFrameBitmap<'_, '_> {
    fn get_pixel(&self, point: Point) -> Option<Color> {
        point_to_index(point, self.size()).map(|index| {
            let color_index = if self.frame.remap_table.len() > 0 {
                let mut color_remapped_index =
                    self.frame.remap_table[self.frame.data[index] as usize];

                if color_remapped_index >= COLOR_HARKONNEN
                && color_remapped_index < COLOR_HARKONNEN + 7 {
                    color_remapped_index += self.faction_palette_offset;
                }

                color_remapped_index
            } else {
                self.frame.data[index] as usize
            };

            self.palette.color_at(color_index as usize)
        })
    }
}

impl SpriteFrame {
    pub fn bitmap<'a, 'b>(
        &'a self,
        palette: &'b Palette,
        faction: Faction,
    ) -> SpriteFrameBitmap<'a, 'b> {
        SpriteFrameBitmap::new(self, palette, faction)
    }
}

impl SpriteFrame {
    pub fn from_shp_reader<T>(
        reader: &mut T,
    ) -> Result<Vec<SpriteFrame>, Box<dyn Error>> where T: Read + Seek {
        let mut frames = Vec::new();

        let shp_version = shp_read_version(reader)?;
        let shp_offsets = shp_read_frame_offsets(reader, shp_version)?;
    
        for (offset, size) in shp_offsets.iter().copied() {
            frames.push(shp_read_frame(reader, offset, size as u64)?);
        };

        Ok(frames)
    }

    pub fn from_shp_file<P>(
        path: P,
    ) -> Result<Vec<SpriteFrame>, Box<dyn Error>> where P: AsRef<path::Path> {
        let mut reader = fs::File::open(path)?;
        Self::from_shp_reader(&mut reader)
    }
}
