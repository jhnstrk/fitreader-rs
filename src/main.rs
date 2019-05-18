// In order to use the Serialize and Deserialize macros in the model,
// we need to declare in the main module, that we are using them.
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate chrono;

use chrono::{Utc};
use chrono::offset::TimeZone;

use serde_json::{Value, Map};

use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, BufWriter, Write, Seek};
use std::collections::HashMap;

use byteorder::{LittleEndian, BigEndian,  ReadBytesExt, WriteBytesExt};

use std::sync::Arc;

mod profile;
use crate::profile::{ProfileData};

mod fitcrc;
use crate::fitcrc::{FitCrc};

const INVALID_U32: u32 = 0xFFFFFFFF;


fn skip_bytes(reader: &mut BufReader<File>, count: u64) -> Result<u64, std::io::Error> {
    // Discard count bytes
    return std::io::copy(&mut reader.by_ref().take(count), &mut std::io::sink());
}

#[derive(Copy, Clone, Default)]
#[derive(Debug)]
struct FitFileHeader {
    header_size: u8,
    protocol_version: u8,
    profile_version: u16,
    data_size: u32,
    type_signature: [u8; 4],
    crc: u16,
}

#[derive(Debug)]
#[derive(Copy, Clone)]
enum Endianness {
    Little, Big,
}

impl Default for Endianness {
    fn default() -> Self { Endianness::Little }
}

#[derive(Debug)]
#[derive(Copy, Clone)]
enum FitDataType {
    FitEnum,
    FitSint8, FitUint8, FitSint16, FitUint16, FitSint32, FitUint32,
    FitString, FitF32, FitF64, FitU8z, FitU16z, FitU32z, FitByte,
    FitSInt64, FitUint64, FitUint64z,
}

#[derive(Debug)]
#[derive(Clone)]
enum FitFieldData {
    FitEnum(Vec<u8>),
    FitSint8(Vec<i8>), FitUint8(Vec<u8>), FitSint16(Vec<i16>), FitUint16(Vec<u16>),
    FitSint32(Vec<i32>), FitUint32(Vec<u32>),
    FitString(String,u8), FitF32(Vec<f32>), FitF64(Vec<f64>), FitU8z(Vec<u8>),
    FitU16z(Vec<u16>), FitU32z(Vec<u32>), FitByte(Vec<u8>),
    FitSInt64(Vec<i64>), FitUint64(Vec<u64>), FitUint64z(Vec<u64>),
}

fn int_to_fit_data_type(value: u8) -> Result<FitDataType, std::io::Error> {
    match value {
        0 => Ok(FitDataType::FitEnum),
        1 => Ok(FitDataType::FitSint8),
        2 => Ok(FitDataType::FitUint8),
        3 => Ok(FitDataType::FitSint16),
        4 => Ok(FitDataType::FitUint16),
        5 => Ok(FitDataType::FitSint32),
        6 => Ok(FitDataType::FitUint32),
        7 => Ok(FitDataType::FitString),
        8 => Ok(FitDataType::FitF32),
        9 => Ok(FitDataType::FitF64),
        10 => Ok(FitDataType::FitU8z),
        11 => Ok(FitDataType::FitU16z),
        12 => Ok(FitDataType::FitU32z),
        13 => Ok(FitDataType::FitByte),
        14 => Ok(FitDataType::FitSInt64),
        15 => Ok(FitDataType::FitUint64),
        16 => Ok(FitDataType::FitUint64z),
        _ => Err( std::io::Error::new(std::io::ErrorKind::Other, "Invalid FIT data type"))
    }
}

fn fit_data_type_to_int(value: &FitDataType) -> Result<u8, std::io::Error> {
    match value {
        FitDataType::FitEnum => Ok(0),
        FitDataType::FitSint8 => Ok(1),
        FitDataType::FitUint8 => Ok(2),
        FitDataType::FitSint16 => Ok(3),
        FitDataType::FitUint16 => Ok(4),
        FitDataType::FitSint32 => Ok(5),
        FitDataType::FitUint32 => Ok(6),
        FitDataType::FitString => Ok(7),
        FitDataType::FitF32 => Ok(8),
        FitDataType::FitF64 => Ok(9),
        FitDataType::FitU8z => Ok(10),
        FitDataType::FitU16z => Ok(11),
        FitDataType::FitU32z => Ok(12),
        FitDataType::FitByte => Ok(13),
        FitDataType::FitSInt64 => Ok(14),
        FitDataType::FitUint64 => Ok(15),
        FitDataType::FitUint64z => Ok(16),
        //unreachable _ => Err( std::io::Error::new(std::io::ErrorKind::Other, "Invalid FIT data type")),
    }
}


fn fit_data_size(data_type: FitDataType) -> Result<u8, std::io::Error> {
    match data_type {
        FitDataType::FitEnum => Ok(1),
        FitDataType::FitSint8 => Ok(1),
        FitDataType::FitUint8 => Ok(1),
        FitDataType::FitSint16 => Ok(2),
        FitDataType::FitUint16 => Ok(2),
        FitDataType::FitSint32 => Ok(4),
        FitDataType::FitUint32 => Ok(4),
        FitDataType::FitString => Ok(0),
        FitDataType::FitF32 => Ok(4),
        FitDataType::FitF64 => Ok(8),
        FitDataType::FitU8z => Ok(1),
        FitDataType::FitU16z => Ok(2),
        FitDataType::FitU32z => Ok(4),
        FitDataType::FitByte => Ok(1),
        FitDataType::FitSInt64 => Ok(8),
        FitDataType::FitUint64 => Ok(8),
        FitDataType::FitUint64z => Ok(8),
    }
}

#[derive(Clone, Debug, Default)]
struct FitFieldDefinition{
    field_defn_num: u8,
    size_in_bytes: u8,
    data_type: Option<FitDataType>,
}

#[derive(Clone, Debug, Default)]
struct FitDeveloperFieldDefinition{
    field_defn_num: u8,
    size_in_bytes: u8,
    data_type: Option<FitDataType>,
    dev_data_index: u8,
}

#[derive(Clone, Debug,Default)]
struct FitDefinitionMessage {
    architecture:Endianness,
    global_message_number: u16,
    local_message_type: u8,
    field_defns: Vec< Arc<FitFieldDefinition> >,
    dev_field_defns: Vec< Arc<FitDeveloperFieldDefinition> >,
}

#[derive(Default)]
#[derive(Debug)]
struct FitFileContext {
    bytes_read: u32,
    bytes_written: u32,
    crc: FitCrc,
    architecture: Option<Endianness>,
    field_definitions: HashMap<u8, Arc<FitDefinitionMessage> >,
    timestamp: u32,
}


#[derive(Default)]
#[derive(Debug)]
struct FitFile {
    header: Arc<FitFileHeader>,
    context: FitFileContext,
}


#[derive(Debug)]
struct FitDataField {
    field_defn_num: u8,
    data: FitFieldData,
}

#[derive(Debug,Default)]
struct FitDataMessage {
    global_message_num: u16,
    local_message_type: u8,
    timestamp: Option<u32>,    // Only set for compressed messages.
    fields: Vec<FitDataField>,
    dev_fields: Vec<FitDataField>,
}

enum FitRecord {
    HeaderRecord(Arc<FitFileHeader>),
    DataRecord(Arc<FitDataMessage>),
    DefinitionMessage(Arc<FitDefinitionMessage>),
}



fn fit_read_u8(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<u8, std::io::Error> {
    let byte = reader.read_u8()?;
    my_file.context.bytes_read = my_file.context.bytes_read + 1;
    my_file.context.crc.consume(&[byte]);
    return Ok(byte);
}

fn fit_write_u8(my_file: &mut FitFile, writer: &mut BufWriter<File>, byte: u8) -> Result<(), std::io::Error> {
    writer.write_u8(byte)?;
    my_file.context.bytes_written = my_file.context.bytes_written + 1;
    return Ok(());
}

fn fit_read_i8(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<i8, std::io::Error> {
    let byte = reader.read_u8()?;
    my_file.context.bytes_read = my_file.context.bytes_read + 1;
    my_file.context.crc.consume(&[byte]);
    return Ok(byte as i8);
}

fn fit_write_i8(my_file: &mut FitFile, writer: &mut BufWriter<File>, byte: i8) -> Result<(), std::io::Error> {
    writer.write_i8(byte)?;
    my_file.context.bytes_written = my_file.context.bytes_written + 1;
    return Ok(());
}

fn fit_read_u16(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<u16, std::io::Error> {

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

fn fit_write_u16(my_file: &mut FitFile, writer: &mut BufWriter<File>, v: u16) -> Result<(), std::io::Error> {
    match my_file.context.architecture {
        Some(Endianness::Little) => writer.write_u16::<LittleEndian>(v)?,
        Some(Endianness::Big) => writer.write_u16::<BigEndian>(v)?,
        None =>  return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_written = my_file.context.bytes_written + 2;
    return Ok(());
}

fn fit_read_i16(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<i16, std::io::Error> {

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

fn fit_write_i16(my_file: &mut FitFile, writer: &mut BufWriter<File>, v: i16) -> Result<(), std::io::Error> {
    match my_file.context.architecture {
        Some(Endianness::Little) => writer.write_i16::<LittleEndian>(v)?,
        Some(Endianness::Big) => writer.write_i16::<BigEndian>(v)?,
        None =>  return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_written = my_file.context.bytes_written + 2;
    return Ok(());
}

fn fit_read_u32(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<u32, std::io::Error> {

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

fn fit_write_u32(my_file: &mut FitFile, writer: &mut BufWriter<File>, v: u32) -> Result<(), std::io::Error> {
    match my_file.context.architecture {
        Some(Endianness::Little) => writer.write_u32::<LittleEndian>(v)?,
        Some(Endianness::Big) => writer.write_u32::<BigEndian>(v)?,
        None =>  return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_written = my_file.context.bytes_written + 4;
    return Ok(());
}

fn fit_read_i32(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<i32, std::io::Error> {

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

fn fit_write_i32(my_file: &mut FitFile, writer: &mut BufWriter<File>, v: i32) -> Result<(), std::io::Error> {
    match my_file.context.architecture {
        Some(Endianness::Little) => writer.write_i32::<LittleEndian>(v)?,
        Some(Endianness::Big) => writer.write_i32::<BigEndian>(v)?,
        None =>  return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_written = my_file.context.bytes_written + 4;
    return Ok(());
}

fn fit_read_u64(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<u64, std::io::Error> {

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

fn fit_write_u64(my_file: &mut FitFile, writer: &mut BufWriter<File>, v: u64) -> Result<(), std::io::Error> {
    match my_file.context.architecture {
        Some(Endianness::Little) => writer.write_u64::<LittleEndian>(v)?,
        Some(Endianness::Big) => writer.write_u64::<BigEndian>(v)?,
        None =>  return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_written = my_file.context.bytes_written + 8;
    return Ok(());
}


fn fit_read_i64(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<i64, std::io::Error> {

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

fn fit_write_i64(my_file: &mut FitFile, writer: &mut BufWriter<File>, v: i64) -> Result<(), std::io::Error> {
    match my_file.context.architecture {
        Some(Endianness::Little) => writer.write_i64::<LittleEndian>(v)?,
        Some(Endianness::Big) => writer.write_i64::<BigEndian>(v)?,
        None =>  return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_written = my_file.context.bytes_written + 8;
    return Ok(());
}

fn fit_read_f32(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<f32, std::io::Error> {

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

fn fit_write_f32(my_file: &mut FitFile, writer: &mut BufWriter<File>, v: f32) -> Result<(), std::io::Error> {
    match my_file.context.architecture {
        Some(Endianness::Little) => writer.write_f32::<LittleEndian>(v)?,
        Some(Endianness::Big) => writer.write_f32::<BigEndian>(v)?,
        None =>  return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_written = my_file.context.bytes_written + 4;
    return Ok(());
}

fn fit_read_f64(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<f64, std::io::Error> {

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

fn fit_write_f64(my_file: &mut FitFile, writer: &mut BufWriter<File>, v: f64) -> Result<(), std::io::Error> {
    match my_file.context.architecture {
        Some(Endianness::Little) => writer.write_f64::<LittleEndian>(v)?,
        Some(Endianness::Big) => writer.write_f64::<BigEndian>(v)?,
        None =>  return Err( std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set"))
    };
    my_file.context.bytes_written = my_file.context.bytes_written + 8;
    return Ok(());
}


// From UTF-8 encoded binary string, null-terminated.
fn fit_read_string(my_file: &mut FitFile, reader: &mut BufReader<File>, width: &u8) -> Result<String, std::io::Error> {

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
fn fit_write_string(my_file: &mut FitFile, writer: &mut BufWriter<File>, v: &str, width: &u8) -> Result<(), std::io::Error> {
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

fn read_global_header(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result< Arc<FitFileHeader>, std::io::Error> {

    let mut header_buf: [u8; 12] = [0; 12];
    reader.read_exact(&mut header_buf)?;


    let mut header_rdr = std::io::Cursor::new(header_buf);

    let mut header: FitFileHeader = Default::default();

    header.header_size = header_rdr.read_u8().unwrap();
    header.protocol_version = header_rdr.read_u8().unwrap();
    header.profile_version = header_rdr.read_u16::<LittleEndian>().unwrap();
    header.data_size = header_rdr.read_u32::<LittleEndian>().unwrap();
    header_rdr.read_exact(&mut header.type_signature )?;

    let expected_signature : [u8;4] = ['.' as u8, 'F' as u8, 'I' as u8, 'T' as u8 ];
    if header.type_signature != expected_signature {
        return Err( std::io::Error::new(std::io::ErrorKind::Other, "Invalid FIT signature"));
    }

    my_file.context.bytes_read = 12;

    // CRC is not present in older FIT formats.
    if header.header_size >= 14 {
        header.crc = reader.read_u16::<LittleEndian>().unwrap();
        my_file.context.bytes_read += 2;

        let actual_crc = fitcrc::compute(&header_buf);
        //println!("Actual: {} Expected: {}", actual_crc, my_file.header.crc);
        if (header.crc != 0) && (actual_crc != header.crc) {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Header CRC is invalid"));
        }
    }

    if header.header_size as u32 > my_file.context.bytes_read {
        skip_bytes(reader, (header.header_size as u64 - my_file.context.bytes_read as u64) as u64)?;
    }
    Ok( Arc::new(header) )
}

fn write_global_header(my_file: &mut FitFile, writer: &mut BufWriter<File>, header: &FitFileHeader)
    -> Result< (), std::io::Error>
{
    let mut header_buf: [u8; 12] = [0; 12];
    {
        let mut header_writer = vec![];

        header_writer.write_u8(header.header_size)?;
        header_writer.write_u8(header.protocol_version)?;
        header_writer.write_u16::<LittleEndian>(header.profile_version)?;
        header_writer.write_u32::<LittleEndian>(header.data_size)?;

        let signature: [u8; 4] = ['.' as u8, 'F' as u8, 'I' as u8, 'T' as u8];
        header_writer.write_all(&signature)?;

        header_buf.copy_from_slice(header_writer.as_slice());
    }
    writer.write_all(&header_buf)?;

    my_file.context.bytes_written = 12;

    // CRC is not present in older FIT formats.
    if header.header_size >= 14 {
        let crc = fitcrc::compute(&header_buf);
        writer.write_u16::<LittleEndian>(crc)?;
        my_file.context.bytes_written += 2;
    }

    if header.header_size as u32 > 14 {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Header size is invalid"));
    }
    Ok( () )
}

fn read_field_defn( my_file: &mut FitFile, reader: &mut BufReader<File>)
    -> Result< Arc<FitFieldDefinition>, std::io::Error> {
    let field_defn_num = fit_read_u8(my_file, reader)?;
    if field_defn_num == 0xFF {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid field: defn_num=255"));
    }
    let size_in_bytes = fit_read_u8(my_file, reader)?;
    if size_in_bytes == 0x0 {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid field: size=0"));
    }
    let base_type = fit_read_u8(my_file, reader)?;

    let mut field_defn: FitFieldDefinition = Default::default();

    let base_type_num = base_type & 0x1F;
    //let base_type_is_endian = base_type & 0x80;

    field_defn.data_type = Some(int_to_fit_data_type(base_type_num)?);
    if field_defn.data_type.is_none() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid field: data type>16"));
    }
    field_defn.size_in_bytes = size_in_bytes;
    field_defn.field_defn_num = field_defn_num;

    Ok( Arc::new(field_defn) )
}

fn write_field_defn( my_file: &mut FitFile, writer: &mut BufWriter<File>, field_defn: &FitFieldDefinition )
                    -> Result< (), std::io::Error>
{
    let base_type_num = fit_data_type_to_int(&field_defn.data_type.unwrap() )?;
    let base_type_is_endian = fit_data_size( field_defn.data_type.unwrap() )? > 1;
    let base_type = base_type_num | ( if base_type_is_endian {0x80} else {0x00} );

    fit_write_u8(my_file, writer, field_defn.field_defn_num)?;
    fit_write_u8(my_file, writer, field_defn.size_in_bytes)?;
    fit_write_u8(my_file, writer, base_type)?;

    Ok( () )
}


fn read_dev_field_defn( my_file: &mut FitFile, reader: &mut BufReader<File>)
    -> Result< Arc<FitDeveloperFieldDefinition>, std::io::Error> {
    let field_defn_num = fit_read_u8(my_file, reader)?;
    let size_in_bytes = fit_read_u8(my_file, reader)?;
    let dev_data_index = fit_read_u8(my_file, reader)?;

    let mut field_defn: FitDeveloperFieldDefinition = Default::default();
    field_defn.field_defn_num = field_defn_num;
    field_defn.size_in_bytes = size_in_bytes;
    field_defn.dev_data_index = dev_data_index;
    Ok(Arc::new(field_defn))
}

fn write_dev_field_defn( my_file: &mut FitFile, writer: &mut BufWriter<File>, field_defn: &FitDeveloperFieldDefinition )
                     -> Result< (), std::io::Error>
{
    fit_write_u8(my_file, writer, field_defn.field_defn_num)?;
    fit_write_u8(my_file, writer, field_defn.size_in_bytes)?;
    fit_write_u8(my_file, writer, field_defn.dev_data_index)?;
    Ok( () )
}

fn read_definition_message( my_file: &mut FitFile, reader: &mut BufReader<File>,
                            local_message_type: u8, is_developer: bool)
                            -> Result< Arc<FitDefinitionMessage>, std::io::Error> {
    let _reserved0 = fit_read_u8(my_file, reader)?;  // Read and discard a reserved byte

    let architecture = fit_read_u8(my_file, reader)?;
    let endian:Endianness = if architecture == 1 { Endianness::Big } else { Endianness::Little };

    my_file.context.architecture = Some(endian);

    let global_message_number = fit_read_u16(my_file, reader)?;
    let number_of_fields = fit_read_u8(my_file, reader)?;

    println!("Definition message: Local ID: {:}, Global ID = {:}, Num. of fields: {}, offset {}",
             local_message_type, global_message_number, number_of_fields, my_file.context.bytes_read);

    let mut defn_mesg = FitDefinitionMessage {
        architecture: endian,
        global_message_number,
        local_message_type,
        ..Default::default()
    };

    for _ifield in 0..number_of_fields {
        defn_mesg.field_defns.push( read_field_defn(my_file, reader)? );
        println!("Field {}: {:?}", _ifield, defn_mesg.field_defns.last().unwrap());
    }

    if is_developer {
        let number_of_dev_fields = fit_read_u8(my_file, reader)?;
        for _ifield in 0..number_of_dev_fields {
            defn_mesg.dev_field_defns.push( read_dev_field_defn(my_file, reader)? );
        }
    }

    let v = Arc::new(defn_mesg);
    my_file.context.field_definitions.insert(local_message_type, v.clone());

    Ok(v)
}

fn write_definition_message( my_file: &mut FitFile, writer: &mut BufWriter<File>, defn_mesg: &FitDefinitionMessage)
                            -> Result< (), std::io::Error>
{
    let is_developer = !defn_mesg.dev_field_defns.is_empty();
    assert!(defn_mesg.local_message_type <= 0x0F);

    let record_hdr = defn_mesg.local_message_type |
        (if is_developer {0x20} else {0x0}) |
        0x40; // Definition message

    fit_write_u8(my_file, writer, record_hdr)?;  // Write header byte
    fit_write_u8(my_file, writer, 0u8)?;  // Write a reserved byte

    match my_file.context.architecture {
        Some(x) => {
            match x {
                Endianness::Big => fit_write_u8(my_file, writer, 1u8)?,
                Endianness::Little => fit_write_u8(my_file, writer, 0u8)?,
            };
        }

        None => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set")),
    };

    fit_write_u16(my_file, writer, defn_mesg.global_message_number)?;
    fit_write_u8(my_file, writer, defn_mesg.field_defns.len() as u8)?;

    println!("Writing definition message: Local ID: {:}, Global ID = {:}, Num. of fields: {}, offset {}",
             defn_mesg.local_message_type, defn_mesg.global_message_number,
             defn_mesg.field_defns.len(), my_file.context.bytes_written);

    for field in &defn_mesg.field_defns {
        write_field_defn(my_file, writer, field)?;
    }

    if is_developer {
        fit_write_u8(my_file, writer, defn_mesg.dev_field_defns.len() as u8)?;
        for field in &defn_mesg.dev_field_defns {
            write_dev_field_defn(my_file, writer, field)?;
        }
    }

//    let v = Arc::new(defn_mesg);
//    my_file.context.field_definitions.insert(local_message_type, v.clone());

    Ok(())
}

fn read_fit_field( my_file: &mut FitFile, reader: &mut BufReader<File>,
                   data_type: FitDataType, count: u8)
    -> Result< FitFieldData, std::io::Error >
{
    //reader.read_u16_into::<NativeEndian>(&mut buffer[..])?;
    match data_type {
        FitDataType::FitEnum => {
            let mut v: Vec<u8> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u8(my_file, reader)?);
            }
            Ok(FitFieldData::FitEnum(v))
        },
        FitDataType::FitSint8 => {
            let mut v: Vec<i8> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_i8(my_file, reader)?);
            }
            Ok(FitFieldData::FitSint8(v))
        },
        FitDataType::FitUint8 => {
            let mut v: Vec<u8> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u8(my_file, reader)?);
            }
            Ok(FitFieldData::FitUint8(v))
        },
        FitDataType::FitSint16 => {
            let mut v: Vec<i16> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_i16(my_file, reader)?);
            }
            Ok(FitFieldData::FitSint16(v))
        },
        FitDataType::FitUint16 => {
            let mut v: Vec<u16> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u16(my_file, reader)?);
            }
            Ok(FitFieldData::FitUint16(v))
        },
        FitDataType::FitSint32 => {
            let mut v: Vec<i32> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_i32(my_file, reader)?);
            }
            Ok(FitFieldData::FitSint32(v))
        },
        FitDataType::FitUint32 => {
            let mut v: Vec<u32> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u32(my_file, reader)?);
            }
            Ok(FitFieldData::FitUint32(v))
        },
        FitDataType::FitString => {
            let v = fit_read_string(my_file, reader, &count)?;
            Ok(FitFieldData::FitString(v,count))
        },
        FitDataType::FitF32 => {
            let mut v: Vec<f32> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_f32(my_file, reader)?);
            }
            Ok(FitFieldData::FitF32(v))
        },
        FitDataType::FitF64 => {
            let mut v: Vec<f64> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_f64(my_file, reader)?);
            }
            Ok(FitFieldData::FitF64(v))
        },
        FitDataType::FitU8z => {
            let mut v: Vec<u8> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u8(my_file, reader)?);
            }
            Ok(FitFieldData::FitU8z(v))
        },
        FitDataType::FitU16z => {
            let mut v: Vec<u16> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u16(my_file, reader)?);
            }
            Ok(FitFieldData::FitU16z(v))
        },
        FitDataType::FitU32z => {
            let mut v: Vec<u32> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u32(my_file, reader)?);
            }
            Ok(FitFieldData::FitU32z(v))
        },
        FitDataType::FitByte => {
            let mut v: Vec<u8> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u8(my_file, reader)?);
            }
            Ok(FitFieldData::FitByte(v))
        },
        FitDataType::FitSInt64 => {
            let mut v: Vec<i64> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_i64(my_file, reader)?);
            }
            Ok(FitFieldData::FitSInt64(v))
        },
        FitDataType::FitUint64 => {
            let mut v: Vec<u64> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u64(my_file, reader)?);
            }
            Ok(FitFieldData::FitUint64(v))
        },
        FitDataType::FitUint64z => {
            let mut v: Vec<u64> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u64(my_file, reader)?);
            }
            Ok(FitFieldData::FitUint64z(v))
        },
    }
}

fn write_fit_field(my_file: &mut FitFile, writer: &mut BufWriter<File>, field: &FitFieldData)
                   -> Result< (), std::io::Error >
{
    match field {
        FitFieldData::FitEnum(x) |
        FitFieldData::FitUint8(x) |
            FitFieldData::FitU8z(x) |
            FitFieldData::FitByte(x)  => {
            for item in x {
                fit_write_u8(my_file, writer, *item)?;
            }
            Ok(())
        },
        FitFieldData::FitSint8(x)  => {
            for item in x {
                fit_write_i8(my_file, writer, *item)?;
            }
            Ok(())
        },
        FitFieldData::FitSint16(x)   => {
            for item in x {
                fit_write_i16(my_file, writer, *item)?;
            }
            Ok(())
        },
        FitFieldData::FitUint16(x)  |
        FitFieldData::FitU16z(x) => {
            for item in x {
                fit_write_u16(my_file, writer, *item)?;
            }
            Ok(())
        },
        FitFieldData::FitSint32(x)   => {
            for item in x {
                fit_write_i32(my_file, writer, *item)?;
            }
            Ok(())
        },
        FitFieldData::FitUint32(x)  |
        FitFieldData::FitU32z(x) => {
            for item in x {
                fit_write_u32(my_file, writer, *item)?;
            }
            Ok(())
        },
        FitFieldData::FitString(x, width) => {
            fit_write_string(my_file, writer, x.as_str(), width )?;
            Ok(())
        },
        FitFieldData::FitF32(x)  => {
            for item in x {
                fit_write_f32(my_file, writer, *item)?;
            }
            Ok(())
        },
        FitFieldData::FitF64(x)  => {
            for item in x {
                fit_write_f64(my_file, writer, *item)?;
            }
            Ok(())
        },

        FitFieldData::FitSInt64(x) => {
            for item in x {
                fit_write_i64(my_file, writer, *item)?;
            }
            Ok(())
        },
        FitFieldData::FitUint64(x) |
        FitFieldData::FitUint64z(x)  => {
            for item in x {
                fit_write_u64(my_file, writer, *item)?;
            }
            Ok(())
        },
    }
}


fn read_data_message( my_file: &mut FitFile, reader: &mut BufReader<File>,
                            local_message_type: u8, timestamp: Option<u32>) -> Result< Arc<FitDataMessage>, std::io::Error> {

    println!("Data message, local ID: {:} at byte {:}", local_message_type, my_file.context.bytes_read);

    let defn_mesg=
        match my_file.context.field_definitions.get(&local_message_type) {
          Some(v) => v,
            None => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Field id not found")),
        }.clone();

    let mut mesg = FitDataMessage{
        global_message_num: defn_mesg.global_message_number,
        local_message_type,
        timestamp,
        ..Default::default()
    };


    for field in &defn_mesg.field_defns {
        let data_size = fit_data_size(field.data_type.unwrap())?;
        let count:u8 = match data_size {
            0 => field.size_in_bytes,
            _ => field.size_in_bytes / data_size };

        my_file.context.architecture = Some(defn_mesg.architecture);

        let field_value_data = read_fit_field(my_file, reader,
                                        field.data_type.unwrap(), count)?;
        // If this is a timestamp, then update the file timestamp, for any compressed messages.
        if field.field_defn_num == 253 {
            match field_value_data.clone() {
                FitFieldData::FitUint32(value) => my_file.context.timestamp = value[0],
                _ => println!("Warning, bad timestamp type")
            }
        }

        let field_value = FitDataField {
            field_defn_num: field.field_defn_num,
            data: field_value_data };
        mesg.fields.push(field_value);

    }

    my_file.context.architecture = Some(defn_mesg.architecture);

    for field in &defn_mesg.dev_field_defns {
        let data_size = fit_data_size(field.data_type.unwrap())?;
        let count:u8 = match data_size {
            0 => field.size_in_bytes,
            _ => field.size_in_bytes / data_size };

        let field_value_data = read_fit_field(my_file, reader,
                                              field.data_type.unwrap(), count)?;

        let field_value = FitDataField {
            field_defn_num: field.field_defn_num,
            data: field_value_data };
        mesg.fields.push(field_value);
    }


    println!("Data message: {:?}", mesg);

    Ok( Arc::new(mesg) )
}

fn write_data_message( my_file: &mut FitFile, writer: &mut BufWriter<File>, mesg: &FitDataMessage)
    -> Result< (), std::io::Error>
{
    let is_compressed = mesg.timestamp.is_some();
    let record_hdr = if is_compressed {
        assert!(mesg.local_message_type <= 0x03);

        let prev_time_stamp = my_file.context.timestamp;
        let new_timestamp = mesg.timestamp.unwrap();
        if (new_timestamp - prev_time_stamp) > 0x1f {
            println!("Warning: compressed timestamp overflow");
        }
        let time_offset = (new_timestamp & 0x1F) as u8;

        0x80u8 | ((mesg.local_message_type & 0x3 ) << 5) | time_offset
    }else {
        assert!(mesg.local_message_type <= 0x0F);
        mesg.local_message_type
    };

    fit_write_u8(my_file, writer, record_hdr)?;  // Write header byte

    for field in &mesg.fields {
        write_fit_field(my_file, writer, &field.data)?;

        // If this is a timestamp, then update the file timestamp, for any compressed messages.
        if field.field_defn_num == 253 {
            match &field.data {
                FitFieldData::FitUint32(value) => my_file.context.timestamp = value[0],
                _ => println!("Warning, bad timestamp type")
            }
        }
    }

    Ok( () )
}


fn read_record(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result< FitRecord, std::io::Error> {
    let record_hdr = fit_read_u8(my_file, reader)?;
    let is_normal_header = (record_hdr & 0x80) == 0;
    let reserve_bit = (record_hdr & 0x10) != 0;  // Bit 4 is reserved and should be zero.

    if reserve_bit {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Reserved bit is set."))
    }

    if is_normal_header {
        let local_message_type = record_hdr & 0x0F;
        if (record_hdr & 0x40) != 0 {
            //Definition message
            let is_developer = record_hdr & 0x20 != 0;
            return Ok(FitRecord::DefinitionMessage(
                read_definition_message( my_file, reader, local_message_type, is_developer)?));
        } else {
            // Data message
            return Ok(FitRecord::DataRecord(
                read_data_message( my_file, reader, local_message_type, None)?));
        }
    } else {
        // Compressed timestamp header
        println!("Compressed message");
        let local_message_type = (record_hdr >> 5) & 0x03;
        let time_offset = (record_hdr & 0x1F) as u32;

        let prev_time_stamp = my_file.context.timestamp;
        let new_timestamp = if time_offset >= (prev_time_stamp & 0x1fu32) {
            (prev_time_stamp & 0xFFFFFFE0) + time_offset
        } else {
            (prev_time_stamp & 0xFFFFFFE0) + time_offset+ 0x20
        };
        // Data message
        return Ok(FitRecord::DataRecord(
            read_data_message( my_file, reader, local_message_type, Some(new_timestamp))?) );
    }
}

fn handle_fit_value<T: Clone>(x: &Vec<T>) -> Value
    where Value: std::convert::From<T> + std::convert::From< Vec<T> >
{
    if x.is_empty() {
        return Value::Null;
    } else if x.len() == 1 {
        return Value::from(x[0].clone() );
    } else {
        return (x.clone()).into();
    }
}

fn write_rec(my_file: &mut FitFile, writer: &mut BufWriter<File>, rec: &FitRecord)
-> Result< (), std::io::Error>
{
    match rec {
        FitRecord::HeaderRecord(header) => write_global_header(my_file, writer, header.as_ref()),
        FitRecord::DefinitionMessage(defn) => write_definition_message(my_file, writer, defn.as_ref()),
        FitRecord::DataRecord(data_message) => write_data_message(my_file, writer, data_message.as_ref()),
    }
}

fn get_timestamp(data_message: &FitDataMessage) -> Option< u32 >
{
    match data_message.timestamp {
        None => {
            for f in &data_message.fields {
                if f.field_defn_num == 253 {
                    match &f.data {
                        FitFieldData::FitUint32(x) => {
                            if !x.is_empty() {
                                return Some(x[0])
                            }},
                        _ => {},
                    }
                }
            }
            None
        },
        Some(x) => {Some(x)},
    }
}

fn clamp_timestamp(v: i64) -> u32
{
    if v < 0 {
        0u32
    } else if v >= std::u32::MAX as i64 {
        std::u32::MAX - 1  // MAX is reserved for a bad value.
    } else {
        v as u32
    }
}

fn check_rec(my_file: &FitFile, rec: &FitRecord)
             -> Result< (), std::io::Error>
{
    let now = Utc::now();
    let base_datetime = Utc.ymd(1989, 12, 31).and_hms(0, 0, 0);
    let earliest_datetime = Utc.ymd(2018, 1, 1).and_hms(0, 0, 0);
    let latest_datetime = now.checked_add_signed(chrono::Duration::weeks(1) ).unwrap();

    //now.checked_sub_signed(Duration::years(2) );
    let offset_min = clamp_timestamp( earliest_datetime.timestamp() - base_datetime.timestamp());  // in seconds
    let offset_max = clamp_timestamp( latest_datetime.timestamp() - base_datetime.timestamp());  // in seconds

    match rec {
        FitRecord::HeaderRecord(_) => {},
        FitRecord::DefinitionMessage(_) => {},
        FitRecord::DataRecord(data_message) => {
            let timestamp_opt = get_timestamp(data_message.as_ref());
            match timestamp_opt {
                None => {},
                Some(x) => {
                    // Seconds since UTC 00:00 Dec 31 1989
                    let utc_dt = base_datetime + chrono::Duration::seconds(x as i64);
                    if x < offset_min || x > offset_max {
                        let errstr = format!("Timestamp error: Out of permitted range {}", utc_dt.to_rfc3339());
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, errstr));
                    } else if x < my_file.context.timestamp {
                        let errstr = format!("Timestamp error: Timestamp is before previous one {}", utc_dt.to_rfc3339());
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, errstr));
                    } else {
                        println!("Timestamp: {}", utc_dt.to_rfc3339());
                    }
                },
            }
        },
    };
    Ok(())
}

fn print_rec(rec: &FitRecord, pf: &ProfileData) {
    match rec {
        FitRecord::HeaderRecord(_) => {},
        FitRecord::DataRecord(data_message) => {

            let mut map = Map::new();

            if !data_message.timestamp.is_none() {
                let value = data_message.timestamp.unwrap();
                if value != INVALID_U32 {
                    map.insert(String::from("timestamp"), Value::from(value));
                }
            }
            let message = pf.get_message(data_message.global_message_num);
            let mut field_name: String;
            for ifield in &data_message.fields{
                let mut field_desc = Option::None;
                if message.is_some() {
                    field_desc = message.unwrap().find_field(ifield.field_defn_num);
                }
                if field_desc.is_some() {
                    field_name = field_desc.unwrap().field_name.clone();
                } else {
                    let field_string = format!("Field{}", ifield.field_defn_num);
                    field_name = field_string;
                }
                let value =
                    match &ifield.data {
                        FitFieldData::FitEnum(x) => handle_fit_value(x),
                        FitFieldData::FitSint8(x)  => handle_fit_value(x),
                        FitFieldData::FitUint8(x)  => handle_fit_value(x),
                        FitFieldData::FitSint16(x)  => handle_fit_value(x),
                        FitFieldData::FitUint16(x)  => handle_fit_value(x),
                        FitFieldData::FitSint32(x)  => handle_fit_value(x),
                        FitFieldData::FitUint32(x) => handle_fit_value(x),
                        FitFieldData::FitString(x,_) => Value::from(x.as_str()),
                        FitFieldData::FitF32(x) => handle_fit_value(x),
                        FitFieldData::FitF64(x) => handle_fit_value(x),
                        FitFieldData::FitU8z(x) => handle_fit_value(x),
                        FitFieldData::FitU16z(x) => handle_fit_value(x),
                        FitFieldData::FitU32z(x) => handle_fit_value(x),
                        FitFieldData::FitByte(x)  => handle_fit_value(x),
                        FitFieldData::FitSInt64(x) => handle_fit_value(x),
                        FitFieldData::FitUint64(x)  => handle_fit_value(x),
                        FitFieldData::FitUint64z(x)  => handle_fit_value(x),
                    };
                map.insert(field_name.to_string(), value);
            }
            let message_name = if message.is_some() {
                message.unwrap().message_name.clone()
            } else {
                format!("Message_{}", data_message.global_message_num)
            };
            println!("{} {}",message_name,Value::Object(map));
        },
        FitRecord::DefinitionMessage(_) => {},
    }
}


fn read_file(path: &str) -> std::io::Result<()> {
    let mut my_file: FitFile = Default::default();
    let p = profile::build_profile()?;

    println!("Opening file: {}", path);
    let file = File::open(path)?;

    let mut reader = BufReader::new(file);

    println!("Reading header from: {}", path);
    my_file.header = read_global_header(&mut my_file, &mut reader)?;


    let file_out = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open("/tmp/fit_out.fit")?;

    let mut writer = BufWriter::new(file_out);
    let mut out_file: FitFile = Default::default();
    out_file.context.architecture = Some(Endianness::Little);
    out_file.header = Arc::new((*my_file.header).clone() );

    let new_header_rec = FitRecord::HeaderRecord(out_file.header.clone());
    write_rec(&mut out_file, &mut writer, &new_header_rec)?;

    let mut num_rec = 1;  // Count the header as one record.

    // Read data, total file size is header + data + crc
    let len_to_read = my_file.header.header_size as u32 + my_file.header.data_size;
    while my_file.context.bytes_read < len_to_read {
        let rec = read_record(&mut my_file, &mut reader);
        match rec {
            Ok(v) => {
                print_rec(&v, &p);
                match check_rec(&my_file, &v ) {
                    Ok(_) => {write_rec(&mut out_file, &mut writer, &v) ?;},
                    Err(e) => println!("Skipping bad values in rec {}", e),
                }

            },
            Err(e) => println!("Skipping bad rec {}", e),
        }
        num_rec = num_rec + 1;
    }

    writer.flush()?;
    // Update data size, write new header.

    let mut new_header = (*out_file.header).clone();
    new_header.data_size = out_file.context.bytes_written - new_header.header_size as u32;
    writer.seek(std::io::SeekFrom::Start(0) )?;
    write_rec(&mut out_file, &mut writer, &FitRecord::HeaderRecord(Arc::new(new_header)))?;
    writer.flush()?;  // Required.

    // compute new crc
    let crc_out = fitcrc::crc_for_file(writer.get_mut() )?;  // "inadvisable"
    writer.seek(std::io::SeekFrom::End(0) )?;
    writer.write_u16::<LittleEndian>(crc_out)?;
    println!("Info: Read {:} records from {:} bytes", num_rec, my_file.context.bytes_read );

    // Read directly as we don't want the crc value included in the crc computation.
    let crc = reader.read_u16::<LittleEndian>()?;
    println!("CRC: Computed 0x{:x}, Provided 0x{:x}", my_file.context.crc.digest(), crc);

    Ok(())
}

fn main() {
    let res: std::io::Result<()> = read_file("/tmp/foo.fit"); // Looks bad around byte 287037
    //let res: std::io::Result<()> = read_file("/tmp/good.fit");
    match res {
        Ok(val) => val,
        Err(e) => println!("Error: {:?}", e),
    }
    println!("Done");
}
