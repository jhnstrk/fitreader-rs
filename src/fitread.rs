use std::io::Read;

use byteorder::{LittleEndian, BigEndian,  ReadBytesExt};

use crate::fittypes::{Endianness, FitFileContext};

pub fn fit_read_u8(context: &mut FitFileContext, reader: &mut dyn Read) -> Result<u8, std::io::Error> {
    let byte = reader.read_u8()?;
    context.data_bytes_read = context.data_bytes_read + 1;
    context.crc.consume(&[byte]);
    return Ok(byte);
}


pub fn fit_read_i8(context: &mut FitFileContext, reader: &mut dyn Read) -> Result<i8, std::io::Error> {
    let byte = reader.read_u8()?;
    context.data_bytes_read = context.data_bytes_read + 1;
    context.crc.consume(&[byte]);
    return Ok(byte as i8);
}

pub fn fit_read_u16(context: &mut FitFileContext, reader: &mut dyn Read) -> Result<u16, std::io::Error> {

    let mut buf: [u8; 2] = [0; 2];
    reader.read_exact(&mut buf)?;

    let mut rdr = std::io::Cursor::new(buf);
    let v = match context.architecture {
        Some(Endianness::Little) => rdr.read_u16::<LittleEndian>()?,
        Some(Endianness::Big) => rdr.read_u16::<BigEndian>()?,
        None => return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    context.data_bytes_read = context.data_bytes_read + 2;
    context.crc.consume(&buf);
    return Ok(v);
}


pub fn fit_read_i16(context: &mut FitFileContext, reader: &mut dyn Read) -> Result<i16, std::io::Error> {

    let mut buf: [u8; 2] = [0; 2];
    reader.read_exact(&mut buf)?;

    let mut rdr = std::io::Cursor::new(buf);
    let v = match context.architecture {
        Some(Endianness::Little) => rdr.read_i16::<LittleEndian>()?,
        Some(Endianness::Big) => rdr.read_i16::<BigEndian>()?,
        None => return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    context.data_bytes_read = context.data_bytes_read + 2;
    context.crc.consume(& buf);
    return Ok(v);
}

pub fn fit_read_u32(context: &mut FitFileContext, reader: &mut dyn Read) -> Result<u32, std::io::Error> {

    let mut buf: [u8; 4] = [0; 4];
    reader.read_exact(&mut buf)?;

    let mut rdr = std::io::Cursor::new(buf);
    let v = match context.architecture {
        Some(Endianness::Little) => rdr.read_u32::<LittleEndian>()?,
        Some(Endianness::Big) => rdr.read_u32::<BigEndian>()?,
        None => return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    context.data_bytes_read = context.data_bytes_read + 4;
    context.crc.consume(& buf);
    return Ok(v);
}

pub fn fit_read_i32(context: &mut FitFileContext, reader: &mut dyn Read) -> Result<i32, std::io::Error> {

    let mut buf: [u8; 4] = [0; 4];
    reader.read_exact(&mut buf)?;

    let mut rdr = std::io::Cursor::new(buf);
    let v = match context.architecture {
        Some(Endianness::Little) => rdr.read_i32::<LittleEndian>()?,
        Some(Endianness::Big) => rdr.read_i32::<BigEndian>()?,
        None => return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    context.data_bytes_read = context.data_bytes_read + 4;
    context.crc.consume(& buf);
    return Ok(v);
}

pub fn fit_read_u64(context: &mut FitFileContext, reader: &mut dyn Read) -> Result<u64, std::io::Error> {

    let mut buf: [u8; 8] = [0; 8];
    reader.read_exact(&mut buf)?;

    let mut rdr = std::io::Cursor::new(buf);
    let v = match context.architecture {
        Some(Endianness::Little) => rdr.read_u64::<LittleEndian>()?,
        Some(Endianness::Big) => rdr.read_u64::<BigEndian>()?,
        None => return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    context.data_bytes_read = context.data_bytes_read + 8;
    context.crc.consume(& buf);
    return Ok(v);
}



pub fn fit_read_i64(context: &mut FitFileContext, reader: &mut dyn Read) -> Result<i64, std::io::Error> {

    let mut buf: [u8; 8] = [0; 8];
    reader.read_exact(&mut buf)?;

    let mut rdr = std::io::Cursor::new(buf);
    let v = match context.architecture {
        Some(Endianness::Little) => rdr.read_i64::<LittleEndian>()?,
        Some(Endianness::Big) => rdr.read_i64::<BigEndian>()?,
        None => return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    context.data_bytes_read = context.data_bytes_read + 8;
    context.crc.consume(& buf);
    return Ok(v);
}

pub fn fit_read_f32(context: &mut FitFileContext, reader: &mut dyn Read) -> Result<f32, std::io::Error> {

    let mut buf: [u8; 4] = [0; 4];
    reader.read_exact(&mut buf)?;

    let mut rdr = std::io::Cursor::new(buf);
    let v = match context.architecture {
        Some(Endianness::Little) => rdr.read_f32::<LittleEndian>()?,
        Some(Endianness::Big) => rdr.read_f32::<BigEndian>()?,
        None => return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    context.data_bytes_read = context.data_bytes_read + 4;
    context.crc.consume(& buf);
    return Ok(v);
}

pub fn fit_read_f64(context: &mut FitFileContext, reader: &mut dyn Read) -> Result<f64, std::io::Error> {

    let mut buf: [u8; 8] = [0; 8];
    reader.read_exact(&mut buf)?;

    let mut rdr = std::io::Cursor::new(buf);
    let v = match context.architecture {
        Some(Endianness::Little) => rdr.read_f64::<LittleEndian>()?,
        Some(Endianness::Big) => rdr.read_f64::<BigEndian>()?,
        None => return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    context.data_bytes_read = context.data_bytes_read + 8;
    context.crc.consume(& buf);
    return Ok(v);
}


// From UTF-8 encoded binary string, null-terminated.
pub fn fit_read_string(context: &mut FitFileContext, reader: &mut dyn Read, width: &u8) -> Result<String, std::io::Error> {

    let mut buf: Vec<u8> = Vec::new();
    let len = *width as usize;
    // Read bytes
    for _i in 0..len {
        let byte = fit_read_u8(context, reader)?;
        buf.push(byte);
    }

    // Drop trailing null(s)
    while (!buf.is_empty()) && (*buf.last().unwrap() == 0) {
        buf.pop();
    }

    let the_string = String::from_utf8_lossy(&buf);
    return Ok(the_string.to_string());
}
