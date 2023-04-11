use std::error::Error;
use std::fs;
use std::io;
use std::iter;
use std::path::PathBuf;

use crate::color::*;
use crate::surface::*;
use crate::io::*;

fn check_chunk_id(
    reader: &mut impl io::Read,
    value: &[u8],
) -> Result<(), Box<dyn Error>> {
    let mut buf = vec![0; value.len()];
    reader.read(&mut buf)?;
    if buf != value {
        return Err(format!("Expected {:?}, got {:?}", value, buf).into());
    }
    Ok(())
}

struct ICNInfo {
    width: u16,
    height: u16,
    bit_per_pixels: u16,
}

impl ICNInfo {
    fn try_read_from(
        reader: &mut impl io::Read,
    ) -> Result<ICNInfo, Box<dyn Error>> {
        check_chunk_id(reader, b"SINF")?;
    
        let sinf_chunk_size = u32::try_read_from::<MSB>(reader)?;

        if sinf_chunk_size != 4 {
            return Err(
                format!("Expected SINF chunk size to be 4, got {}",
                sinf_chunk_size,
            ).into());
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
    fn from_reader<T: io::Read + io::Seek>(
        reader: &mut T,
        info: &ICNInfo,
    ) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
        check_chunk_id(reader, b"SSET")?;

        let sset_chunk_size = u32::try_read_from::<MSB>(reader)? as usize;
        let tile_size = info.get_tile_size();
        let tile_count = (sset_chunk_size - 4)/tile_size;

        reader.seek(io::SeekFrom::Current(8))?;

        (0..tile_count).map(|_| {
            let mut tile = vec![0; tile_size];

            reader.read_exact(&mut tile)?;
            Ok::<Vec<_>, Box<dyn Error>>(tile)
        }).collect::<Result<Vec<_>, _>>()
    }
}

pub struct ICNRPal;

impl ICNRPal {
    fn from_reader(
        reader: &mut impl io::Read,
        info: &ICNInfo,
    ) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
        check_chunk_id(reader, b"RPAL")?;

        let rpal_chunk_size = u32::try_read_from::<MSB>(reader)? as usize;
        let pal_size = info.get_palette_size();
        let pal_count = rpal_chunk_size/pal_size;

        (0..pal_count).map(|_| {
            let mut pal = vec![0; pal_size];

            reader.read_exact(&mut pal)?;
            Ok::<Vec<_>, Box<dyn Error>>(pal)
        }).collect::<Result<Vec<_>, _>>()
    }
}

pub struct ICNRTbl;

impl ICNRTbl {
    fn from_reader(
        reader: &mut impl io::Read,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        check_chunk_id(reader, b"RTBL")?;

        let rtbl_chunk_size = u32::try_read_from::<MSB>(reader)? as usize;
        let mut rtbl = vec![0; rtbl_chunk_size];

        reader.read_exact(&mut rtbl)?;

        Ok(rtbl)
    }
}

pub struct ICNTile {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

impl ICNTile {
    pub fn surface(
        &self,
        palette: &Palette,
    ) -> Surface {
        let mut surface = Surface::new(Size {
            width: self.width as u32,
            height: self.height as u32,
        });

        for (i, &color_index) in self.data.iter().enumerate() {
            let x = ((i as u16)%self.width) as i32;
            let y = ((i as u16)/self.width) as i32;
            let color = palette.color_at(color_index as usize);

            surface.put_pixel(Point { x, y }, color);
        }

        surface
    }
}

pub struct ICN {
    pub tiles: Vec<ICNTile>,
}

impl ICN {
    pub fn from_reader<T>(
        reader: &mut T,
    ) -> Result<ICN, Box<dyn Error>> where T: io::Read + io::Seek {
        check_chunk_id(reader, b"FORM")?;

        reader.seek(io::SeekFrom::Current(4))?; // Skip chunk size

        check_chunk_id(reader, b"ICON")?;

        let info = ICNInfo::try_read_from(reader)?;
        let sset = ICNSSet::from_reader(reader, &info)?;
        let rpal = ICNRPal::from_reader(reader, &info)?;
        let rtbl = ICNRTbl::from_reader(reader)?;

        if sset.len() != rtbl.len() {
            return Err("SSET and RTBL size mismatch".into());
        }

        let tiles = iter::zip(sset.iter(), rtbl.iter())
            .map(|(raw_data, rpal_index)| {
                let bpp = info.bit_per_pixels;
                let mut data = Vec::new();

                for b in raw_data {
                    for i in (0..8/bpp).rev() {
                        let p = (b >> i*bpp) & ((1 << bpp) - 1);
                        data.push(rpal[*rpal_index as usize][p as usize]);
                    }
                }

                ICNTile {
                    width: info.width,
                    height: info.height,
                    data,
                }
            }).collect::<Vec<_>>();

        Ok(ICN { tiles })
    }
}

impl std::convert::TryFrom<PathBuf> for ICN {
    type Error = Box<dyn Error>;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let mut reader = fs::File::open(path)?;
        return ICN::from_reader(&mut reader);
    }
}
