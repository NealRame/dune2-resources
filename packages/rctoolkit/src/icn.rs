use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::iter;
use std::path;

use anyhow::{anyhow, Result};

use dune2_rc::{Size, Tile};

use crate::io::*;


fn check_chunk_id(
    reader: &mut impl Read,
    value: &[u8],
) -> Result<()> {
    let mut buf = vec![0; value.len()];

    reader.read(&mut buf)?;
    if buf != value {
        return Err(anyhow!("ICN: invalid chunk ID"));
    }
    Ok(())
}

struct ICNInfo {
    width: u16,
    height: u16,
    bit_per_pixels: u16,
}

impl ICNInfo {
    fn read_from(
        reader: &mut impl Read,
    ) -> Result<ICNInfo> {
        check_chunk_id(reader, b"SINF")?;

        let sinf_chunk_size = u32::try_read_from::<MSB>(reader)?;

        if sinf_chunk_size != 4 {
            return Err(anyhow!(
                "ICN: Expected SINF chunk size to be 4, got {sinf_chunk_size}"
            ));
        }

        let width  = u8::try_read_from::<MSB>(reader)? as u16;
        let height = u8::try_read_from::<MSB>(reader)? as u16;
        let shift  = u8::try_read_from::<MSB>(reader)? as u16;
        let bit_per_pixels = u8::try_read_from::<MSB>(reader)? as u16;

        Ok(Self {
            width: width << shift,
            height: height << shift,
            bit_per_pixels,
        })
    }

    fn get_tile_size(&self) -> usize {
        ((self.width*self.height*self.bit_per_pixels)/8) as usize
    }

    fn get_palette_size(&self) -> usize {
        1 << self.bit_per_pixels
    }
}

pub struct ICNSSet;

impl ICNSSet {
    fn read_from<T: Read + Seek>(
        reader: &mut T,
        info: &ICNInfo,
    ) -> Result<Vec<Vec<u8>>> {
        check_chunk_id(reader, b"SSET")?;

        let sset_chunk_size = u32::try_read_from::<MSB>(reader)? as usize;
        let tile_size = info.get_tile_size();
        let tile_count = (sset_chunk_size - 4)/tile_size;

        reader.seek(SeekFrom::Current(8))?;

        let mut tiles = Vec::new();

        for _ in 0..tile_count {
            let mut tile_data = vec![0; tile_size];
            reader.read_exact(&mut tile_data)?;

            tiles.push(tile_data);
        }

        Ok(tiles)
    }
}

pub struct ICNRPal;

impl ICNRPal {
    fn read_from(
        reader: &mut impl Read,
        info: &ICNInfo,
    ) -> Result<Vec<Vec<u8>>> {
        check_chunk_id(reader, b"RPAL")?;

        let rpal_chunk_size = u32::try_read_from::<MSB>(reader)? as usize;
        let pal_size = info.get_palette_size();
        let pal_count = rpal_chunk_size/pal_size;

        let mut pals = Vec::new();

        for _ in 0..pal_count {
            let mut pal_data = vec![0; pal_size];
            reader.read_exact(&mut pal_data)?;

            pals.push(pal_data);
        }

        Ok(pals)
    }
}

struct ICNRTbl;

impl ICNRTbl {
    fn read_from(
        reader: &mut impl Read,
    ) -> Result<Vec<u8>> {
        check_chunk_id(reader, b"RTBL")?;

        let rtbl_chunk_size = u32::try_read_from::<MSB>(reader)? as usize;
        let mut rtbl = vec![0; rtbl_chunk_size];

        reader.read_exact(&mut rtbl)?;

        Ok(rtbl)
    }
}

fn read_tiles_from_reader<T>(
    reader: &mut T,
) -> Result<Vec<Tile>> where T: Read + Seek {
    check_chunk_id(reader, b"FORM")?;

    reader.seek(SeekFrom::Current(4))?; // Skip chunk size

    check_chunk_id(reader, b"ICON")?;

    let info = ICNInfo::read_from(reader)?;
    let sset = ICNSSet::read_from(reader, &info)?;
    let rpal = ICNRPal::read_from(reader, &info)?;
    let rtbl = ICNRTbl::read_from(reader)?;

    if sset.len() != rtbl.len() {
        return Err(anyhow!("ICN: SSET and RTBL size mismatch"));
    }

    let tile_size = Size {
        width: info.width as u32,
        height: info.height as u32,
    };

    let tiles = iter::zip(sset.iter(), rtbl.iter())
        .map(|(raw_data, rpal_index)| {
            let bpp = info.bit_per_pixels;
            let mut tile_data = Vec::new();

            for b in raw_data {
                for i in (0..8/bpp).rev() {
                    let p = (b >> i*bpp) & ((1 << bpp) - 1);
                    tile_data.push(rpal[*rpal_index as usize][p as usize]);
                }
            }

            Tile::new(&tile_data[..], tile_size)
        }).collect::<Vec<_>>();

    Ok(tiles)
}

pub fn read_tiles_from_file<P>(
    path: P,
) -> Result<Vec<Tile>> where P: AsRef<path::Path> {
    let mut reader = fs::File::open(path)?;
    read_tiles_from_reader(&mut reader)
}
