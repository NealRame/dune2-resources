use std::error::Error;

use std::fs;

use std::io;
use std::io::Write;
use std::io::Read;
use std::io::Seek;

use std::path::PathBuf;

use crate::config::Cli;

fn read_offset<T>(reader: &mut T)
    -> Option<u64>
    where T: io::BufRead + io::Seek
{
    let mut buf = [0; 4];

    match reader.read(&mut buf) {
        Ok(4) => {
            let offset = u32::from_le_bytes(buf);

            if offset == 0 {
                Some(reader.seek(io::SeekFrom::End(0)).unwrap())
            } else {
                Some(offset as u64)
            }
        },
        _ => None,
    }
}

fn read_cstring<T>(reader: &mut T)
    -> Option<String>
    where T: io::BufRead
{
    let mut buf = Vec::new();
    let mut name = String::new();

    match reader.read_until(0, &mut buf) {
        Ok(_) => {
            buf.pop();
            name.push_str(&String::from_utf8_lossy(&buf));
            Some(name)
        },
        Err(_) => None,
    }
}

struct PAKRawEntry(u64, String);

struct PAKRawEntryReader<T>
    where T: io::BufRead + io::Seek {
    source: T,
}

impl<T> PAKRawEntryReader<T>
    where T: io::BufRead + io::Seek {
    fn new(source: T) -> PAKRawEntryReader<T> {
        PAKRawEntryReader { source }
    }
}

impl<T> Iterator for PAKRawEntryReader<T>
    where T: io::BufRead + io::Seek {
    type Item = PAKRawEntry;

    fn next(&mut self) -> Option<Self::Item> {
        match read_offset(&mut self.source) {
            Some(offset) => {
                let name = read_cstring(&mut self.source).unwrap();
                Some(PAKRawEntry(offset, name))
            },
            None => None,
        }
    }
}

pub fn run(config: Cli) -> Result<(), Box<dyn Error>> {
    let output_dirpath = config.output_dir.unwrap_or_else(|| {
        PathBuf::from(&config.input_filepath.file_stem().unwrap())
    });

    fs::create_dir_all(&output_dirpath)?;

    let reader = PAKRawEntryReader::new(io::BufReader::new(fs::File::open(&config.input_filepath)?));
    let pak_raw_entries = reader.collect::<Vec<PAKRawEntry>>();

    for (i, entry) in pak_raw_entries[0..(pak_raw_entries.len() - 1)].iter().enumerate() {
        let next_entry = &pak_raw_entries[i + 1];

        let offset = entry.0;
        let size = (next_entry.0 - entry.0) as usize;
        let name = &entry.1;

        let mut data = vec![0; size];
        let mut input = fs::File::open(&config.input_filepath)?;

        input.seek(io::SeekFrom::Start(offset))?;
        input.read(&mut data)?;

        if config.list {
            println!("{}: {} bytes", name, size);
        } else {
            let output_filepath = output_dirpath.join(name);

            if config.verbose {
                println!("Extracting {} ...", output_filepath.display());
            }
    
            if output_filepath.exists() && !config.overwrite {
                let message = format!("File already exists: {}", output_filepath.display());
                return Err(Box::new(io::Error::new(io::ErrorKind::AlreadyExists, message)));
            }

            fs::File::create(output_filepath)?.write_all(&data)?;
        }
    }

    Ok(())
}
