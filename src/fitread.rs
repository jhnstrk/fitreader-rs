use std::io::{BufReader, Read, BufWriter, Write, Seek};
use std::collections::HashMap;
use std::sync::Arc;
use std::env;

use byteorder::{LittleEndian, BigEndian,  ReadBytesExt};
use std::fs::File;

use crate::fittypes::{Endianness, FitFile};

pub fn fit_read_u8(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<u8, std::io::Error> {
    let byte = reader.read_u8()?;
    my_file.context.bytes_read = my_file.context.bytes_read + 1;
    my_file.context.crc.consume(&[byte]);
    return Ok(byte);
}


pub fn fit_read_i8(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<i8, std::io::Error> {
    let byte = reader.read_u8()?;
    my_file.context.bytes_read = my_file.context.bytes_read + 1;
    my_file.context.crc.consume(&[byte]);
    return Ok(byte as i8);
}

pub fn fit_read_u16(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<u16, std::io::Error> {

    let mut buf: [u8; 2] = [0; 2];
    reader.read_exact(&mut buf)?;

    let mut rdr = std::io::Cursor::new(buf);
    let v = match my_file.context.architecture {
        Some(Endianness::Little) => rdr.read_u16::<LittleEndian>()?,
        Some(Endianness::Big) => rdr.read_u16::<BigEndian>()?,
        None => return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_read = my_file.context.bytes_read + 2;
    my_file.context.crc.consume(&buf);
    return Ok(v);
}


pub fn fit_read_i16(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<i16, std::io::Error> {

    let mut buf: [u8; 2] = [0; 2];
    reader.read_exact(&mut buf)?;

    let mut rdr = std::io::Cursor::new(buf);
    let v = match my_file.context.architecture {
        Some(Endianness::Little) => rdr.read_i16::<LittleEndian>()?,
        Some(Endianness::Big) => rdr.read_i16::<BigEndian>()?,
        None => return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_read = my_file.context.bytes_read + 2;
    my_file.context.crc.consume(& buf);
    return Ok(v);
}

pub fn fit_read_u32(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<u32, std::io::Error> {

    let mut buf: [u8; 4] = [0; 4];
    reader.read_exact(&mut buf)?;

    let mut rdr = std::io::Cursor::new(buf);
    let v = match my_file.context.architecture {
        Some(Endianness::Little) => rdr.read_u32::<LittleEndian>()?,
        Some(Endianness::Big) => rdr.read_u32::<BigEndian>()?,
        None => return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_read = my_file.context.bytes_read + 4;
    my_file.context.crc.consume(& buf);
    return Ok(v);
}

pub fn fit_read_i32(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<i32, std::io::Error> {

    let mut buf: [u8; 4] = [0; 4];
    reader.read_exact(&mut buf)?;

    let mut rdr = std::io::Cursor::new(buf);
    let v = match my_file.context.architecture {
        Some(Endianness::Little) => rdr.read_i32::<LittleEndian>()?,
        Some(Endianness::Big) => rdr.read_i32::<BigEndian>()?,
        None => return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_read = my_file.context.bytes_read + 4;
    my_file.context.crc.consume(& buf);
    return Ok(v);
}

pub fn fit_read_u64(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<u64, std::io::Error> {

    let mut buf: [u8; 8] = [0; 8];
    reader.read_exact(&mut buf)?;

    let mut rdr = std::io::Cursor::new(buf);
    let v = match my_file.context.architecture {
        Some(Endianness::Little) => rdr.read_u64::<LittleEndian>()?,
        Some(Endianness::Big) => rdr.read_u64::<BigEndian>()?,
        None => return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_read = my_file.context.bytes_read + 8;
    my_file.context.crc.consume(& buf);
    return Ok(v);
}



pub fn fit_read_i64(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<i64, std::io::Error> {

    let mut buf: [u8; 8] = [0; 8];
    reader.read_exact(&mut buf)?;

    let mut rdr = std::io::Cursor::new(buf);
    let v = match my_file.context.architecture {
        Some(Endianness::Little) => rdr.read_i64::<LittleEndian>()?,
        Some(Endianness::Big) => rdr.read_i64::<BigEndian>()?,
        None => return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_read = my_file.context.bytes_read + 8;
    my_file.context.crc.consume(& buf);
    return Ok(v);
}

pub fn fit_read_f32(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<f32, std::io::Error> {

    let mut buf: [u8; 4] = [0; 4];
    reader.read_exact(&mut buf)?;

    let mut rdr = std::io::Cursor::new(buf);
    let v = match my_file.context.architecture {
        Some(Endianness::Little) => rdr.read_f32::<LittleEndian>()?,
        Some(Endianness::Big) => rdr.read_f32::<BigEndian>()?,
        None => return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_read = my_file.context.bytes_read + 4;
    my_file.context.crc.consume(& buf);
    return Ok(v);
}

pub fn fit_read_f64(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<f64, std::io::Error> {

    let mut buf: [u8; 8] = [0; 8];
    reader.read_exact(&mut buf)?;

    let mut rdr = std::io::Cursor::new(buf);
    let v = match my_file.context.architecture {
        Some(Endianness::Little) => rdr.read_f64::<LittleEndian>()?,
        Some(Endianness::Big) => rdr.read_f64::<BigEndian>()?,
        None => return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_read = my_file.context.bytes_read + 8;
    my_file.context.crc.consume(& buf);
    return Ok(v);
}


// From UTF-8 encoded binary string, null-terminated.
pub fn fit_read_string(my_file: &mut FitFile, reader: &mut BufReader<File>, width: &u8) -> Result<String, std::io::Error> {

    let mut buf: Vec<u8> = Vec::new();
    let len = *width as usize;
    // Read bytes
    for _i in 0..len {
        let byte = reader.read_u8()?;
        buf.push(byte);
    }

    let the_string = String::from_utf8_lossy(&buf);

    my_file.context.bytes_read = my_file.context.bytes_read + buf.len() as u32;
    my_file.context.crc.consume(& buf);
    return Ok(the_string.to_string());
}
