// In order to use the Serialize and Deserialize macros in the model,
// we need to declare in the main module, that we are using them.
#[macro_use]
extern crate serde_derive;

mod profile;

use std::fs::File;
use std::io::{BufReader, Read};
use std::collections::HashMap;

use byteorder::{LittleEndian, BigEndian,  ReadBytesExt};

use std::borrow::Cow;

fn fit_crc_16_u8(mut crc: u16, byte: &u8) -> u16 {
    let crc_table: [u16; 16] =  [
        0x0000, 0xCC01, 0xD801, 0x1400, 0xF001, 0x3C00, 0x2800, 0xE401,
        0xA001, 0x6C00, 0x7800, 0xB401, 0x5000, 0x9C01, 0x8801, 0x4400
    ];

    // compute checksum of lower four bits of byte
    let mut tmp = crc_table[(crc & 0xF) as usize];
    crc = (crc >> 4) & 0x0FFF;
    crc = crc ^ tmp ^ crc_table[(byte & 0xF) as usize];

    // now compute checksum of upper four bits of byte
    tmp = crc_table[(crc & 0xF) as usize];
    crc = (crc >> 4) & 0x0FFF;
    crc = crc ^ tmp ^ crc_table[((byte >> 4) & 0xFu8) as usize];

    return crc;
}

fn fit_crc_16(mut crc: u16, byte_array: &[u8]) -> u16 {

    for byte in byte_array.iter() {
        crc = fit_crc_16_u8(crc, byte);
    }
    return crc;
}

fn skip_bytes(reader: &mut BufReader<File>, count: u64) -> Result<u64, std::io::Error> {
    // Discard count bytes
    return std::io::copy(&mut reader.by_ref().take(count), &mut std::io::sink());
}

#[derive(Default)]
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
    FitString(String), FitF32(Vec<f32>), FitF64(Vec<f64>), FitU8z(Vec<u8>),
    FitU16z(Vec<i16>), FitU32z(Vec<u32>), FitByte(Vec<u8>),
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
    field_defns: Vec<FitFieldDefinition>,
    dev_field_defns: Vec<FitDeveloperFieldDefinition>,
}

#[derive(Default)]
#[derive(Debug)]
struct FitFileContext {
    bytes_read: u32,
    crc: u16,
    architecture: Option<Endianness>,
    field_definitions: HashMap<u8, FitDefinitionMessage>,
    time_stamp: u32,
}


#[derive(Default)]
#[derive(Debug)]
struct FitFile {
    header: FitFileHeader,
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
    is_compressed: bool,
    time_stamp: u32,
    fields: Vec<FitDataField>,
    dev_fields: Vec<FitDataField>,
}

enum FitRecord {
    HeaderRecord(FitFileHeader),
    DataRecord(FitDataMessage),
    DefinitionMessage(FitDefinitionMessage),
}



fn fit_read_u8(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<u8, std::io::Error> {
    let byte = reader.read_u8()?;
    my_file.context.bytes_read = my_file.context.bytes_read + 1;
    my_file.context.crc = fit_crc_16_u8(my_file.context.crc, &byte);
    return Ok(byte);
}

fn fit_read_i8(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<i8, std::io::Error> {
    let byte = reader.read_u8()?;
    my_file.context.bytes_read = my_file.context.bytes_read + 1;
    my_file.context.crc = fit_crc_16_u8(my_file.context.crc, &byte);
    return Ok(byte as i8);
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
    my_file.context.crc = fit_crc_16(my_file.context.crc, & buf);
    return Ok(v);
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
    my_file.context.crc = fit_crc_16(my_file.context.crc, & buf);
    return Ok(v);
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
    my_file.context.crc = fit_crc_16(my_file.context.crc, & buf);
    return Ok(v);
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
    my_file.context.crc = fit_crc_16(my_file.context.crc, & buf);
    return Ok(v);
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
    my_file.context.crc = fit_crc_16(my_file.context.crc, & buf);
    return Ok(v);
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
    my_file.context.crc = fit_crc_16(my_file.context.crc, & buf);
    return Ok(v);
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
    my_file.context.crc = fit_crc_16(my_file.context.crc, & buf);
    return Ok(v);
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
    my_file.context.crc = fit_crc_16(my_file.context.crc, & buf);
    return Ok(v);
}

// From UTF-8 encoded binary string, null-terminated.
fn fit_read_string(my_file: &mut FitFile, reader: &mut BufReader<File>, max_len: usize) -> Result<String, std::io::Error> {

    let mut buf: Vec<u8> = Vec::new();

    // Read to terminating null. Include the null because we need it for the CRC.
    for _i in 0..max_len {
        let byte = reader.read_u8()?;
        buf.push(byte);
    }

    let the_string = String::from_utf8_lossy(&buf);

    my_file.context.bytes_read = my_file.context.bytes_read + buf.len() as u32;
    my_file.context.crc = fit_crc_16(my_file.context.crc, & buf);
    return Ok(the_string.to_string());
}

fn read_global_header(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<(), std::io::Error> {

    let mut header_buf: [u8; 12] = [0; 12];
    reader.read_exact(&mut header_buf)?;


    let mut header_rdr = std::io::Cursor::new(header_buf);

    my_file.header.header_size = header_rdr.read_u8().unwrap();
    my_file.header.protocol_version = header_rdr.read_u8().unwrap();
    my_file.header.profile_version = header_rdr.read_u16::<LittleEndian>().unwrap();
    my_file.header.data_size = header_rdr.read_u32::<LittleEndian>().unwrap();
    header_rdr.read_exact(&mut my_file.header.type_signature )?;

    let expected_signature : [u8;4] = ['.' as u8, 'F' as u8, 'I' as u8, 'T' as u8 ];
    if my_file.header.type_signature != expected_signature {
        return Err( std::io::Error::new(std::io::ErrorKind::Other, "Invalid FIT signature"));
    }

    my_file.context.bytes_read = 12;

    // CRC is not present in older FIT formats.
    if my_file.header.header_size >= 14 {
        my_file.header.crc = reader.read_u16::<LittleEndian>().unwrap();
        my_file.context.bytes_read += 2;

        let actual_crc = fit_crc_16(0,&header_buf);
        //println!("Actual: {} Expected: {}", actual_crc, my_file.header.crc);
        if (my_file.header.crc != 0) && (actual_crc != my_file.header.crc) {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Header CRC is invalid"));
        }
    }

    if my_file.header.header_size as u32 > my_file.context.bytes_read {
        skip_bytes(reader, (my_file.header.header_size as u64 - my_file.context.bytes_read as u64) as u64)?;
    }
    Ok(())
}

fn read_field_defn( my_file: &mut FitFile, reader: &mut BufReader<File>)
    -> Result<FitFieldDefinition, std::io::Error> {
    let field_defn_num = fit_read_u8(my_file, reader)?;
    let size_in_bytes = fit_read_u8(my_file, reader)?;
    let base_type = fit_read_u8(my_file, reader)?;

    if field_defn_num == 0xFF {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid field"));
    }
    let mut field_defn: FitFieldDefinition = Default::default();

    let base_type_num = base_type & 0x0F;
    //let base_type_is_endian = base_type & 0x80;

    field_defn.data_type = Some(int_to_fit_data_type(base_type_num)?);
    field_defn.size_in_bytes = size_in_bytes;
    field_defn.field_defn_num = field_defn_num;

    Ok(field_defn)
}


fn read_dev_field_defn( my_file: &mut FitFile, reader: &mut BufReader<File>)
    -> Result<FitDeveloperFieldDefinition, std::io::Error> {
    let field_defn_num = fit_read_u8(my_file, reader)?;
    let size_in_bytes = fit_read_u8(my_file, reader)?;
    let dev_data_index = fit_read_u8(my_file, reader)?;

    let mut field_defn: FitDeveloperFieldDefinition = Default::default();
    field_defn.field_defn_num = field_defn_num;
    field_defn.size_in_bytes = size_in_bytes;
    field_defn.dev_data_index = dev_data_index;
    Ok(field_defn)
}

fn read_definition_message( my_file: &mut FitFile, reader: &mut BufReader<File>,
                            local_message_type: u8, is_developer: bool) -> Result<(), std::io::Error> {
    let reserved0 = fit_read_u8(my_file, reader)?;  // Read and discard a reserved byte

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
        ..Default::default()
    };

    for _iField in 0..number_of_fields {
        defn_mesg.field_defns.push( read_field_defn(my_file, reader)? );
        println!("Field {}: {:?}", _iField, defn_mesg.field_defns.last().unwrap());
    }

    if is_developer {
        let number_of_dev_fields = fit_read_u8(my_file, reader)?;
        for _iField in 0..number_of_dev_fields {
            defn_mesg.dev_field_defns.push( read_dev_field_defn(my_file, reader)? );
        }
    }

    my_file.context.field_definitions.insert(local_message_type, defn_mesg);

    Ok(())
}


fn read_fit_field( my_file: &mut FitFile, reader: &mut BufReader<File>,
    architecture: Endianness, data_type: FitDataType, count: u8)
    -> Result< FitFieldData, std::io::Error >
{
    //reader.read_u16_into::<NativeEndian>(&mut buffer[..])?;
    match data_type {
        FitDataType::FitEnum => {
            let mut v: Vec<u8> = Vec::new();
            for i in 0..count {
                v.push(fit_read_u8(my_file, reader)?);
            }
            Ok(FitFieldData::FitEnum(v))
        },
        FitDataType::FitSint8 => {
            let mut v: Vec<i8> = Vec::new();
            for i in 0..count {
                v.push(fit_read_i8(my_file, reader)?);
            }
            Ok(FitFieldData::FitSint8(v))
        },
        FitDataType::FitUint8 => {
            let mut v: Vec<u8> = Vec::new();
            for i in 0..count {
                v.push(fit_read_u8(my_file, reader)?);
            }
            Ok(FitFieldData::FitUint8(v))
        },
        FitDataType::FitSint16 => {
            let mut v: Vec<i16> = Vec::new();
            for i in 0..count {
                v.push(fit_read_i16(my_file, reader)?);
            }
            Ok(FitFieldData::FitSint16(v))
        },
        FitDataType::FitUint16 => {
            let mut v: Vec<u16> = Vec::new();
            for i in 0..count {
                v.push(fit_read_u16(my_file, reader)?);
            }
            Ok(FitFieldData::FitUint16(v))
        },
        FitDataType::FitSint32 => {
            let mut v: Vec<i32> = Vec::new();
            for i in 0..count {
                v.push(fit_read_i32(my_file, reader)?);
            }
            Ok(FitFieldData::FitSint32(v))
        },
        FitDataType::FitUint32 => {
            let mut v: Vec<u32> = Vec::new();
            for i in 0..count {
                v.push(fit_read_u32(my_file, reader)?);
            }
            Ok(FitFieldData::FitUint32(v))
        },
        FitDataType::FitString => {
            let v = fit_read_string(my_file, reader, count as usize)?;
            Ok(FitFieldData::FitString(v))
        },
        FitDataType::FitF32 => {
            let mut v: Vec<f32> = Vec::new();
            for i in 0..count {
                v.push(fit_read_f32(my_file, reader)?);
            }
            Ok(FitFieldData::FitF32(v))
        },
        FitDataType::FitF64 => {
            let mut v: Vec<f64> = Vec::new();
            for i in 0..count {
                v.push(fit_read_f64(my_file, reader)?);
            }
            Ok(FitFieldData::FitF64(v))
        },
        FitDataType::FitU8z => {
            let mut v: Vec<u8> = Vec::new();
            for i in 0..count {
                v.push(fit_read_u8(my_file, reader)?);
            }
            Ok(FitFieldData::FitU8z(v))
        },
        FitDataType::FitU16z => {
            let mut v: Vec<u16> = Vec::new();
            for i in 0..count {
                v.push(fit_read_u16(my_file, reader)?);
            }
            Ok(FitFieldData::FitUint16(v))
        },
        FitDataType::FitU32z => {
            let mut v: Vec<u32> = Vec::new();
            for i in 0..count {
                v.push(fit_read_u32(my_file, reader)?);
            }
            Ok(FitFieldData::FitU32z(v))
        },
        FitDataType::FitByte => {
            let mut v: Vec<u8> = Vec::new();
            for i in 0..count {
                v.push(fit_read_u8(my_file, reader)?);
            }
            Ok(FitFieldData::FitByte(v))
        },
        FitDataType::FitSInt64 => {
            let mut v: Vec<i64> = Vec::new();
            for i in 0..count {
                v.push(fit_read_i64(my_file, reader)?);
            }
            Ok(FitFieldData::FitSInt64(v))
        },
        FitDataType::FitUint64 => {
            let mut v: Vec<u64> = Vec::new();
            for i in 0..count {
                v.push(fit_read_u64(my_file, reader)?);
            }
            Ok(FitFieldData::FitUint64(v))
        },
        FitDataType::FitUint64z => {
            let mut v: Vec<u64> = Vec::new();
            for i in 0..count {
                v.push(fit_read_u64(my_file, reader)?);
            }
            Ok(FitFieldData::FitUint64z(v))
        },
    }
}

fn read_data_message( my_file: &mut FitFile, reader: &mut BufReader<File>,
                            local_message_type: u8) -> Result<FitDataMessage, std::io::Error> {

    println!("Data message, local ID: {:} at byte {:}", local_message_type, my_file.context.bytes_read);

    let defn_mesg=
        match my_file.context.field_definitions.get(&local_message_type) {
          Some(v) => v,
            None => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Field id not found")),
        }.clone();

    let mut mesg = FitDataMessage{
        global_message_num: defn_mesg.global_message_number,
        ..Default::default()
    };

    for field in &defn_mesg.field_defns {
        let data_size = fit_data_size(field.data_type.unwrap())?;
        let count:u8 = match data_size {
            0 => field.size_in_bytes,
            _ => field.size_in_bytes / data_size };

        let field_value_data = read_fit_field(my_file, reader, defn_mesg.architecture,
                                        field.data_type.unwrap(), count)?;

        let field_value = FitDataField {
            field_defn_num: field.field_defn_num,
            data: field_value_data };
        mesg.fields.push(field_value);
    }

    for field in &defn_mesg.dev_field_defns {
        let data_size = fit_data_size(field.data_type.unwrap())?;
        let count:u8 = match data_size {
            0 => field.size_in_bytes,
            _ => field.size_in_bytes / data_size };

        let field_value_data = read_fit_field(my_file, reader, defn_mesg.architecture,
                                              field.data_type.unwrap(), count)?;

        let field_value = FitDataField {
            field_defn_num: field.field_defn_num,
            data: field_value_data };
        mesg.fields.push(field_value);
    }


    println!("Data message: {:?}", mesg);

    Ok( mesg )
}


fn read_record(my_file: &mut FitFile, reader: &mut BufReader<File>) -> Result<(), std::io::Error> {
    let record_hdr = fit_read_u8(my_file, reader)?;
    let is_normal_header = (record_hdr & 0x80) == 0;

    if is_normal_header {
        let local_message_type = record_hdr & 0x0F;
        if (record_hdr & 0x40) != 0 {
            //Definition message
            let is_developer = record_hdr & 0x20 != 0;
            read_definition_message( my_file, reader, local_message_type, is_developer);
        } else {
            // Data message
            read_data_message( my_file, reader, local_message_type);
        }
    } else {
        // Compressed timestamp header
        println!("Compressed message");
        let local_message_type = (record_hdr >> 5) & 0x03;
        let time_offset = (record_hdr & 0x1F) as u32;

        let prev_time_stamp = my_file.context.time_stamp;
        let new_timestamp = if time_offset >= (prev_time_stamp & 0x1fu32) {
            (prev_time_stamp & 0xFFFFFFE0) + time_offset
        } else {
            (prev_time_stamp & 0xFFFFFFE0) + time_offset+ 0x20
        };
    }
    Ok(())
}

fn read_file(path: &str) -> std::io::Result<()> {
    let mut my_file: FitFile = Default::default();
    let p = profile::build_profile();

    println!("Opening file: {}", path);
    let file = File::open(path)?;

    let mut reader = BufReader::new(file);

    println!("Reading header from: {}", path);
    read_global_header(&mut my_file, &mut reader)?;

    let mut num_rec = 1;  // Count the header as one record.
    while my_file.context.bytes_read < my_file.header.data_size {
        read_record(&mut my_file, &mut reader)?;
        num_rec = num_rec + 1;
    }

    println!("Info: Read {:} records from {:} bytes", num_rec, my_file.context.bytes_read );

    // Read directly as we don't want the crc value included in the crc computation.
    let crc = reader.read_u16::<LittleEndian>()?;
    println!("CRC: Computed 0x{:x}, Provided 0x{:x}", my_file.context.crc, crc);

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
