use std::error::Error;
use std::io;

pub use crate::color::*;
pub use crate::point::*;
pub use crate::rect::*;
pub use crate::size::*;
pub use crate::surface::*;

fn ppi2ppm(ppi: u32) -> u32 {
    ((1000./254.)*(ppi as f32)) as u32
}

pub fn write_bmp_with_palette<T>(
    bitmap: &Surface,
    palette: &Palette,
    writer: &mut T,
) -> Result<(), Box<dyn Error>> where T: io::Write + io::Seek {
    // see https://en.wikipedia.org/wiki/BMP_file_format

    let size = bitmap.size();

    let palette_size = palette.len() as u32;
    let bits_per_pixel = if palette_size <= 256 { 8 } else { 24 } as u16;

    // write BMP file header
    writer.write_all(b"BM")?;

    let header_file_size_offset = writer.seek(io::SeekFrom::Current(0))?;

    writer.write_all(&[0; 4])?; // file size
    writer.write_all(&[0; 4])?; // reserved

    let header_pixel_array_offset = writer.seek(io::SeekFrom::Current(0))?;

    writer.write_all(&[0; 4])?; // offset of pixel array

    // write DIB header
    writer.write_all(&(40 as u32).to_le_bytes())?;     // DIB header size
    writer.write_all(&size.width.to_le_bytes())?;      // Width
    writer.write_all(&size.height.to_le_bytes())?;     // Height
    writer.write_all(&(1 as u16).to_le_bytes())?;      // Color planes
    writer.write_all(&bits_per_pixel.to_le_bytes())?;  // Bits per pixel
    writer.write_all(&[0; 4])?;                        // Compression
    writer.write_all(&[0; 4])?;                        // Image size
    writer.write_all(&ppi2ppm(300).to_le_bytes())?;    // X pixels per meter
    writer.write_all(&ppi2ppm(300).to_le_bytes())?;    // Y pixels per meter
    writer.write_all(&palette_size.to_le_bytes())?;    // Number of colors in the palette
    writer.write_all(&[0; 4])?;                        // Number of important colors used

    // write palette
    for (_, color) in palette.iter() {
        writer.write_all(&[color.blue, color.green, color.red, 0])?;
    }

    // write offset to pixel array in the section header
    let pixel_array_offset = writer.seek(io::SeekFrom::Current(0))?;

    writer.seek(io::SeekFrom::Start(header_pixel_array_offset))?;
    writer.write_all(&(pixel_array_offset as u32).to_le_bytes())?;
    writer.seek(io::SeekFrom::Start(pixel_array_offset))?;

    // write pixel array
    let row_size = ((bits_per_pixel as u32)*size.width + 31)/32*4;
    let pad_size = row_size - ((bits_per_pixel/8) as u32)*size.width;

    for y in (0..size.height).rev() {
        for x in 0..size.width {
            let color = bitmap.get_pixel(Point { x, y }).unwrap();
            if bits_per_pixel == 8 {
                writer.write_all(&[palette.color_index(&color).unwrap() as u8]).unwrap();
            } else {
                writer.write_all(&color.blue.to_le_bytes()).unwrap();
                writer.write_all(&color.green.to_le_bytes()).unwrap();
                writer.write_all(&color.red.to_le_bytes()).unwrap();
            }
        }

        // write padding
        (0..pad_size).for_each(|_| {
            writer.write_all(&[0]).unwrap();
        });
    }

    // write file size in the section header
    let file_size = writer.seek(io::SeekFrom::Current(0))?;

    writer.seek(io::SeekFrom::Start(header_file_size_offset))?;
    writer.write_all(&(file_size as u32).to_le_bytes())?;
    writer.seek(io::SeekFrom::Start(file_size))?;

    // flush
    writer.flush()?;

    Ok(())
}
