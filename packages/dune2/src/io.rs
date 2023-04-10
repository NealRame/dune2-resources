use std::io::{Error, Read};

pub trait FromLEReader: Sized {
    fn from_le_reader<T>(reader: &mut T) -> Result<Self, Error> where T: Read;
}

macro_rules! impl_read_integer {
    ($($t:ty)*) => ($(
        impl FromLEReader for $t {
            fn from_le_reader<T>(
                reader: &mut T,
            ) -> Result<Self, Error> where T: Read {
                let mut buf: [u8; std::mem::size_of::<$t>()] = [0; std::mem::size_of::<$t>()];
                reader.read_exact(&mut buf)?;
                Ok(<$t>::from_le_bytes(buf))
            }
        }
    )*)
}

impl_read_integer! {
    i8 i16 i32 i64 i128
    u8 u16 u32 u64 u128
}
