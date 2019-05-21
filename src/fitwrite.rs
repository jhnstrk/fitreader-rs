use std::io::BufWriter;
use std::fs::File;
use byteorder::{LittleEndian, BigEndian, WriteBytesExt};

use crate::fittypes::{Endianness, FitFile};

pub fn fit_write_u8(my_file: &mut FitFile, writer: &mut BufWriter<File>, byte: u8) -> Result<(), std::io::Error> {
    writer.write_u8(byte)?;
    my_file.context.bytes_written = my_file.context.bytes_written + 1;
    return Ok(());
}

pub fn fit_write_i8(my_file: &mut FitFile, writer: &mut BufWriter<File>, byte: i8) -> Result<(), std::io::Error> {
    writer.write_i8(byte)?;
    my_file.context.bytes_written = my_file.context.bytes_written + 1;
    return Ok(());
}
pub fn fit_write_u16(my_file: &mut FitFile, writer: &mut BufWriter<File>, v: u16) -> Result<(), std::io::Error> {
    match my_file.context.architecture {
        Some(Endianness::Little) => writer.write_u16::<LittleEndian>(v)?,
        Some(Endianness::Big) => writer.write_u16::<BigEndian>(v)?,
        None =>  return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_written = my_file.context.bytes_written + 2;
    return Ok(());
}

pub fn fit_write_i16(my_file: &mut FitFile, writer: &mut BufWriter<File>, v: i16) -> Result<(), std::io::Error> {
    match my_file.context.architecture {
        Some(Endianness::Little) => writer.write_i16::<LittleEndian>(v)?,
        Some(Endianness::Big) => writer.write_i16::<BigEndian>(v)?,
        None =>  return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_written = my_file.context.bytes_written + 2;
    return Ok(());
}

pub fn fit_write_u32(my_file: &mut FitFile, writer: &mut BufWriter<File>, v: u32) -> Result<(), std::io::Error> {
    match my_file.context.architecture {
        Some(Endianness::Little) => writer.write_u32::<LittleEndian>(v)?,
        Some(Endianness::Big) => writer.write_u32::<BigEndian>(v)?,
        None =>  return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_written = my_file.context.bytes_written + 4;
    return Ok(());
}

pub fn fit_write_i32(my_file: &mut FitFile, writer: &mut BufWriter<File>, v: i32) -> Result<(), std::io::Error> {
    match my_file.context.architecture {
        Some(Endianness::Little) => writer.write_i32::<LittleEndian>(v)?,
        Some(Endianness::Big) => writer.write_i32::<BigEndian>(v)?,
        None =>  return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_written = my_file.context.bytes_written + 4;
    return Ok(());
}

pub fn fit_write_u64(my_file: &mut FitFile, writer: &mut BufWriter<File>, v: u64) -> Result<(), std::io::Error> {
    match my_file.context.architecture {
        Some(Endianness::Little) => writer.write_u64::<LittleEndian>(v)?,
        Some(Endianness::Big) => writer.write_u64::<BigEndian>(v)?,
        None =>  return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_written = my_file.context.bytes_written + 8;
    return Ok(());
}

pub fn fit_write_i64(my_file: &mut FitFile, writer: &mut BufWriter<File>, v: i64) -> Result<(), std::io::Error> {
    match my_file.context.architecture {
        Some(Endianness::Little) => writer.write_i64::<LittleEndian>(v)?,
        Some(Endianness::Big) => writer.write_i64::<BigEndian>(v)?,
        None =>  return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_written = my_file.context.bytes_written + 8;
    return Ok(());
}

pub fn fit_write_f32(my_file: &mut FitFile, writer: &mut BufWriter<File>, v: f32) -> Result<(), std::io::Error> {
    match my_file.context.architecture {
        Some(Endianness::Little) => writer.write_f32::<LittleEndian>(v)?,
        Some(Endianness::Big) => writer.write_f32::<BigEndian>(v)?,
        None =>  return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_written = my_file.context.bytes_written + 4;
    return Ok(());
}

pub fn fit_write_f64(my_file: &mut FitFile, writer: &mut BufWriter<File>, v: f64) -> Result<(), std::io::Error> {
    match my_file.context.architecture {
        Some(Endianness::Little) => writer.write_f64::<LittleEndian>(v)?,
        Some(Endianness::Big) => writer.write_f64::<BigEndian>(v)?,
        None =>  return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_written = my_file.context.bytes_written + 8;
    return Ok(());
}

pub fn fit_write_string(my_file: &mut FitFile, writer: &mut BufWriter<File>, v: &str, width: &u8) -> Result<(), std::io::Error> {
    let vbytes = v.as_bytes();
    let sz = *width as usize;
    let mut string_bytes = vbytes.len();
    if string_bytes > sz {
        println!("Warning: String is longer than field width");
        string_bytes = sz;
    }
    // Write bytes
    for _i in 0..string_bytes {
        writer.write_u8( vbytes[_i])?;
    }
    // zero terminate and pad.
    for _i in string_bytes..sz {
        writer.write_u8( 0)?;
    }
    my_file.context.bytes_written = my_file.context.bytes_written + (sz as u32);
    return Ok(());
}
