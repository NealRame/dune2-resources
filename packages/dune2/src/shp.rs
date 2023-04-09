use std::error::Error;
use std::fs;
use std::io;
use std::iter;
use std::path::PathBuf;

use bitvec::prelude::*;

use crate::COLOR_HARKONNEN;
use crate::Faction;
use crate::color::*;
use crate::surface::*;

enum SHPVersion {
    V100,
    V107,
}

pub struct SHPFrame {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
    pub remap_table: Vec<u8>,
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
            let x = ((i as u16) % self.width) as i32;
            let y = ((i as u16) / self.width) as i32;

            let color = if self.remap_table.len() > 0 {
                let mut color_remapped_index = self.remap_table[color_index as usize] as usize;

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

fn read_integer<T>(iter: &mut impl Iterator<Item = u8>) -> Result<T, Box<dyn Error>>
where
    T: Default
        + std::ops::BitOr<Output = T>
        + std::ops::Shl<usize, Output = T>
        + From<u8>,
{
    let mut num = T::default();

    for i in 0..std::mem::size_of::<T>() {
        let byte = iter.next().ok_or_else(|| {
            format!(
                "{} bytes are not enough to read {} of size {}",
                i,
                std::any::type_name::<T>(),
                std::mem::size_of::<T>(),
            )
        })?;
        num = num | (T::from(byte) << (i*8));
    }
    Ok(num)
}

fn copy_block(data: &mut Vec<u8>, count: usize, pos: usize, relative: bool) {
    let offset = if relative { data.len() - pos } else { pos };
    for i in 0..count {
        data.push(data[offset + i]);
    }
}

fn inflate_lcw_data(
    lcw_data: &[u8],
    output: &mut Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    let relative = lcw_data[0] == 0;
    let mut iter = lcw_data.iter().cloned().skip(if relative { 1 } else { 0 });

    while let Some(cmd) = iter.next() {
        if cmd == 0x80 {
            break;
        } else if (cmd & 0xc0) == 0x80 {
            // command 1: short copy
            // 0b10cccccc
            let count = (cmd & 0x3f) as usize;

            output.extend(iter.by_ref().take(count));
        } else if (cmd & 0x80) == 0 {
            // command 2: existing block relative copy
            // 0b0cccpppp p
            let count = (((cmd & 0x70) as usize) >> 4) + 3;
            let pos   = (((cmd & 0x0f) as usize) << 8) | read_integer::<u8>(&mut iter)? as usize;

            if pos == 1 {
                output.extend(iter::repeat(*output.last().unwrap()).take(count));
            } else {
                copy_block(output, count, pos, true);
            }
        } else if cmd == 0xfe {
            // command 4: repeat value
            // 0b11111110 c c v
            let count = read_integer::<u16>(&mut iter)? as usize;
            let value = read_integer::<u8>(&mut iter)?;

            output.extend(iter::repeat(value).take(count));
        } else if cmd == 0xff {
            // command 5: existing block long copy
            // 0b11111111 c c p p
            let count = read_integer::<u16>(&mut iter)? as usize;
            let pos   = read_integer::<u16>(&mut iter)? as usize;

            copy_block(output, count, pos, relative);
        } else {
            // command 3: existing block medium-length copy
            // 0b11cccccc p p
            let count = ((cmd & 0x3f) + 3) as usize;
            let pos   = read_integer::<u16>(&mut iter)? as usize;

            copy_block(output, count, pos, relative);
        }
    }

    Ok(())
}

fn inflate_rle_data(rle_data: &[u8], output: &mut Vec<u8>) {
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

    let mut buf = vec![0; size as usize];

    reader.read_exact(&mut buf)?;

    let flags = BitSlice::<_, Lsb0>::from_slice(&buf[0..2]);

    let mut iter = buf.iter().copied().skip(3);

    let width = read_integer::<u16>(&mut iter)? as u16;
    let height = read_integer::<u8>(&mut iter)? as u16;

    let frame_size = read_integer::<u16>(&mut iter)? as usize;
    if frame_size != size as usize {
        return Err(format!("File size mismatch: {} != {}", frame_size, size).into());
    }

    let rle_data_size = read_integer::<u16>(&mut iter)? as usize;
    let remap_table_size = if flags[HAS_REMAP_TABLE] {
        if flags[CUSTOM_SIZE_REMAP] {
            read_integer::<u8>(&mut iter)?
        } else { 16 }
    } else { 0 } as usize;

    let remap_table = iter.by_ref().take(remap_table_size).collect::<Vec<_>>();

    let compressed_data_offset = 10 + remap_table_size;

    let mut data = Vec::new();

    if flags[NO_LCW] {
        inflate_rle_data(&buf[compressed_data_offset..], &mut data);
    } else {
        let mut rle_data = Vec::with_capacity(rle_data_size);

        inflate_lcw_data(&buf[compressed_data_offset..], &mut rle_data)?;
        inflate_rle_data(&rle_data, &mut data);
    }

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
