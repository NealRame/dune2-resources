use std::error::Error;
use std::fs;
use std::io;
use std::iter;
use std::path::PathBuf;

use bitvec::prelude::*;

use crate::color::*;
use crate::constants::*;
use crate::io::*;
use crate::surface::*;

enum SHPVersion {
    V100,
    V107,
}

pub struct SHPFrame {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
    pub remap_table: Vec<usize>,
}

impl SHPFrame {
    pub fn surface(
        &self,
        palette: &Palette,
        faction: Faction,
    ) -> Surface {
        let mut surface = Surface::new(Size {
            width: self.width as u32,
            height: self.height as u32,
        });

        let faction_palette_offset = 16*(faction as usize);

        for (i, &color_index) in self.data.iter().enumerate() {
            let x = ((i as u16)%self.width) as i32;
            let y = ((i as u16)/self.width) as i32;

            let color = if self.remap_table.len() > 0 {
                let mut color_remapped_index = self.remap_table[color_index as usize];

                if color_remapped_index >= COLOR_HARKONNEN
                    && color_remapped_index < COLOR_HARKONNEN + 7 {
                    color_remapped_index += faction_palette_offset;
                }
                palette.color_at(color_remapped_index)
            } else {
                palette.color_at(color_index as usize)
            };

            surface.put_pixel(Point { x, y }, color);
            surface.put_pixel(Point { x, y }, color);
        }

        surface
    }
}

pub struct SHP {
    pub frames: Vec<SHPFrame>,
}

fn shp_read_version<T>(reader: &mut T)
    -> Result<SHPVersion, Box<dyn Error>>
    where T: io::Read + io::Seek
{
    reader.seek(io::SeekFrom::Start(4))?;

    let mut buf = [0; 2];
    reader.read_exact(&mut buf)?;

    let version = match u16::from_le_bytes(buf) {
        0 => SHPVersion::V107,
        _ => SHPVersion::V100,
    };

    reader.seek(io::SeekFrom::Start(0))?;

    Ok(version)
}

fn shp_read_frame_count<T>(reader: &mut T)
    -> Result<usize, Box<dyn Error>>
    where T: io::Read + io::Seek
{
    let mut buf = [0; 2];
    reader.read_exact(&mut buf)?;

    Ok(u16::from_le_bytes(buf) as usize)
}

fn shp_read_frame_offset_v100<T>(
    reader: &mut T,
) -> Result<u64, Box<dyn Error>> where T: io::Read + io::Seek {
    let mut buf = [0; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf) as u64)
}

fn shp_read_frame_offset_v107<T>(
    reader: &mut T,
) -> Result<u64, Box<dyn Error>> where T: io::Read + io::Seek {
    let mut buf = [0; 4];
    reader.read_exact(&mut buf)?;
    Ok((u32::from_le_bytes(buf) + 2) as u64)
}

fn shp_read_frame_offsets<T>(
    reader: &mut T,
    version: SHPVersion,
) -> Result<Vec<(u64, usize)>, Box<dyn Error>> where T: io::Read + io::Seek {
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

fn inflate_lcw_data<T: io::Read + io::Seek>(
    reader: &mut T,
    output: &mut Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    let relative = u8::try_read_from::<LSB>(reader)? == 0;

    if !relative {
        reader.seek(io::SeekFrom::Current(-1))?;
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
) -> Result<SHPFrame, Box<dyn Error>> where T: io::Read + io::Seek {

    const HAS_REMAP_TABLE: usize = 0;
    const NO_LCW: usize = 1;
    const CUSTOM_SIZE_REMAP: usize = 2;

    reader.seek(io::SeekFrom::Start(offset))?;

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

    Ok(SHPFrame {
        width,
        height,
        remap_table,
        data,
    })
}

impl SHP {
    pub fn from_reader<T>(
        reader: &mut T,
    ) -> Result<SHP, Box<dyn Error>> where T: io::Read + io::Seek {
        let mut frames = Vec::new();

        let shp_version = shp_read_version(reader)?;
        let shp_offsets = shp_read_frame_offsets(reader, shp_version)?;
    
        for (offset, size) in shp_offsets.iter().copied() {
            frames.push(shp_read_frame(reader, offset, size as u64)?);
        };

        Ok(SHP { frames })
    }
}

impl std::convert::TryFrom<PathBuf> for SHP {
    type Error = Box<dyn Error>;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let mut reader = fs::File::open(path)?;
        return SHP::from_reader(&mut reader);
    }
}
