
use crate::fitcrc::{FitCrc};

use chrono::{DateTime, TimeZone, Utc};

use std::sync::Arc;
use std::collections::HashMap;

pub const INVALID_U32: u32 = 0xFFFFFFFF;

#[derive(Copy, Clone, Default)]
#[derive(Debug)]
pub struct FitFileHeader {
    pub header_size: u8,
    pub protocol_version: u8,
    pub profile_version: u16,
    pub data_size: u32,
    pub type_signature: [u8; 4],
    pub crc: u16,
}

#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum Endianness {
    Little, Big,
}

impl Default for Endianness {
    fn default() -> Self { Endianness::Little }
}

#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum FitDataType {
    FitEnum,
    FitSint8, FitUint8, FitSint16, FitUint16, FitSint32, FitUint32,
    FitString, FitF32, FitF64, FitU8z, FitU16z, FitU32z, FitByte,
    FitSInt64, FitUint64, FitUint64z,
}

#[derive(Debug)]
#[derive(Clone)]
pub enum FitFieldData {
    FitEnum(Vec<u8>),
    FitSint8(Vec<i8>), FitUint8(Vec<u8>), FitSint16(Vec<i16>), FitUint16(Vec<u16>),
    FitSint32(Vec<i32>), FitUint32(Vec<u32>),
    FitString(String,u8), FitF32(Vec<f32>), FitF64(Vec<f64>), FitU8z(Vec<u8>),
    FitU16z(Vec<u16>), FitU32z(Vec<u32>), FitByte(Vec<u8>),
    FitSInt64(Vec<i64>), FitUint64(Vec<u64>), FitUint64z(Vec<u64>),
}

impl FitDataType {
    pub fn from_type_id(value: u8) -> Result<FitDataType, std::io::Error> {
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
            _ => Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid FIT data type"))
        }
    }

    pub fn type_id(&self) -> u8 {
        match self {
            FitDataType::FitEnum => 0,
            FitDataType::FitSint8 => 1,
            FitDataType::FitUint8 => 2,
            FitDataType::FitSint16 => 3,
            FitDataType::FitUint16 => 4,
            FitDataType::FitSint32 => 5,
            FitDataType::FitUint32 => 6,
            FitDataType::FitString => 7,
            FitDataType::FitF32 => 8,
            FitDataType::FitF64 => 9,
            FitDataType::FitU8z => 10,
            FitDataType::FitU16z => 11,
            FitDataType::FitU32z => 12,
            FitDataType::FitByte => 13,
            FitDataType::FitSInt64 => 14,
            FitDataType::FitUint64 => 15,
            FitDataType::FitUint64z => 16,
        }
    }

    pub fn data_size(&self) -> u8 {
        match self {
            FitDataType::FitEnum => 1,
            FitDataType::FitSint8 => 1,
            FitDataType::FitUint8 => 1,
            FitDataType::FitSint16 => 2,
            FitDataType::FitUint16 => 2,
            FitDataType::FitSint32 => 4,
            FitDataType::FitUint32 => 4,
            FitDataType::FitString => 0,
            FitDataType::FitF32 => 4,
            FitDataType::FitF64 => 8,
            FitDataType::FitU8z => 1,
            FitDataType::FitU16z => 2,
            FitDataType::FitU32z => 4,
            FitDataType::FitByte => 1,
            FitDataType::FitSInt64 => 8,
            FitDataType::FitUint64 => 8,
            FitDataType::FitUint64z => 8,
        }
    }
}



#[derive(Clone, Debug, Default)]
pub struct FitFieldDefinition{
    pub field_defn_num: u8,
    pub size_in_bytes: u8,
    pub data_type: Option<FitDataType>,
}

#[derive(Clone, Debug, Default)]
pub struct FitDeveloperFieldDefinition{
    pub field_defn_num: u8,
    pub size_in_bytes: u8,
    pub data_type: Option<FitDataType>,
    pub dev_data_index: u8,
}

#[derive(Clone, Debug,Default)]
pub struct FitDefinitionMessage {
    pub architecture:Endianness,
    pub global_message_number: u16,
    pub local_message_type: u8,
    pub field_defns: Vec< Arc<FitFieldDefinition> >,
    pub dev_field_defns: Vec< Arc<FitDeveloperFieldDefinition> >,
}

#[derive(Default)]
#[derive(Debug)]
pub struct FitFileContext {
    pub data_bytes_read: u32,
    pub data_bytes_written: u32,
    pub crc: FitCrc,
    pub architecture: Option<Endianness>,
    pub field_definitions: HashMap<u8, Arc<FitDefinitionMessage> >,
    pub timestamp: u32,
}


#[derive(Default)]
#[derive(Debug)]
pub struct FitFile {
    pub header: FitFileHeader,
    pub records: Vec<FitRecord>,
}


#[derive(Debug)]
pub struct FitDataField {
    pub field_defn_num: u8,
    pub data: FitFieldData,
}

#[derive(Debug,Default)]
pub struct FitDataMessage {
    pub global_message_number: u16,
    pub local_message_type: u8,
    pub timestamp: Option<u32>,    // Only set for compressed messages.
    pub fields: Vec<FitDataField>,
    pub dev_fields: Vec<FitDataField>,
}

#[derive(Debug)]
pub enum FitRecord {
    HeaderRecord(FitFileHeader),
    DataRecord(FitDataMessage),
    DefinitionMessage(Arc<FitDefinitionMessage>),
    EndOfFile(u16),
}

pub fn base_datetime() -> DateTime<Utc> {
    Utc.ymd(1989, 12, 31).and_hms(0, 0, 0)
}

