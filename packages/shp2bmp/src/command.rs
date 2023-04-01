use std::error::Error;
use std::fs;
use std::io;
use std::iter;

use bitvec::prelude::*;

use crate::config::Cli;


enum SHPVersion {
    V100,
    V107,
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

fn copy_block(data: &[u8], count: usize, pos: usize, relative: bool) -> Vec<u8> {
    let mut output = Vec::new();
    if relative {
        output.extend(data.iter().copied().skip(data.len() - pos).take(count));
    } else {
        output.extend(data.iter().skip(pos).take(count));
    }
    output
}

fn read_lcw_data(iter: &mut impl Iterator<Item = u8>)
    -> Result<Vec<u8>, Box<dyn Error>>
{
    let mut iter = iter.peekable();
    let relative = iter.peek() == Some(&0);

    // Ignore first byte if it is the relative mode flag
    if relative {
        iter.next();
    }

    let mut output = Vec::new();

    while let Some(cmd) = iter.next() {
        if (cmd & 0xc0) == 0x80 {
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
                output.extend(copy_block(&output, count, pos, relative));
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
            output.extend(copy_block(&output, count, pos, relative));
        } else {
            // command 3: existing block medium-length copy
            // 0b11cccccc p p
            let count = ((cmd & 0x3f) + 3) as usize;
            let pos   = read_integer::<u16>(&mut iter)? as usize;
            output.extend(copy_block(&output, count, pos, relative));
        }
    }

    Ok(output)
}

fn shp_read_frame<T>(
    reader: &mut T,
    offset: u64,
    size: u64,
) -> Result<(), Box<dyn Error>> where T: io::Read + io::Seek {
    println!("frame offset={:#04x}, size={}", offset, size);

    const HAS_REMAP_TABLE: usize = 0;
    const NO_LCW: usize = 1;
    const CUSTOM_SIZE_REMAP: usize = 2;

    reader.seek(io::SeekFrom::Start(offset))?;

    let mut buf = vec![0; size as usize];

    reader.read_exact(&mut buf)?;

    let flags = BitSlice::<_, Lsb0>::from_slice(&buf[0..2]);

    let mut iter = buf.iter().copied().skip(3);

    let width = read_integer::<u16>(&mut iter)?;
    let height = read_integer::<u8>(&mut iter)?;

    let size = read_integer::<u16>(&mut iter)?;
    let compressed_size = read_integer::<u16>(&mut iter)?;

    let remap_table_size = if flags[HAS_REMAP_TABLE] {
        if flags[CUSTOM_SIZE_REMAP] {
            read_integer::<u8>(&mut iter)?
        } else { 16 }
    } else { 0 } as usize;

    let remap_table = iter.by_ref().take(remap_table_size).collect::<Vec<_>>();
    let data_size = buf.len() - 10 - remap_table_size;

    println!("  - frame data:        {:?}",    buf);
    println!("  - flags:             {:?}",    flags);
    println!("  - has remap table:   {}",      flags[0]);
    println!("  - no LCW:            {}",      flags[1]);
    println!("  - custom size remap: {}",      flags[2]);
    println!("  - width:             {}",      width);
    println!("  - height:            {}",      height);
    println!("  - size:              {}",      size);
    println!("  - remap table size:  {}",      remap_table_size);
    println!("  - remap table:       {:?}",    remap_table);
    println!("  - data size:         {}",      data_size);
    println!("  - compressed size:   {}",      compressed_size);
    println!("  - data offset:       {:#04x}", offset + 10 + remap_table_size as u64);

    let data = if flags[NO_LCW] {
        iter.take(data_size).collect::<Vec<_>>()
    } else {
        read_lcw_data(&mut iter)?
    };

    println!("  - data:              {:?}", data.len());

    Ok(())
}

pub fn run(config: Cli) -> Result<(), Box<dyn Error>> {
    // let palette = dune2::Palette::try_from(config.pal_input_filepath)?;

    let mut shp_reader = fs::File::open(&config.shp_input_filepath)?;

    let shp_version = shp_read_version(&mut shp_reader)?;
    let shp_offsets = shp_read_frame_offsets(&mut shp_reader, shp_version)?;

    for (index, (offset, size)) in shp_offsets.iter().copied().enumerate() {
        println!("frame index={}", index);
        shp_read_frame(&mut shp_reader, offset, size as u64)?;
    };

    return Ok(());
}
