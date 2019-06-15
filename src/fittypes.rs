
use crate::fitcrc::{FitCrc};

use chrono::{DateTime, TimeZone, Utc};

use std::sync::Arc;
use std::collections::HashMap;
use std::convert::{TryFrom};

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

    pub fn name(&self) -> &str {
        match self {
            FitDataType::FitEnum => "enum",
            FitDataType::FitSint8 => "sint8",
            FitDataType::FitUint8 => "uint8",
            FitDataType::FitSint16 => "sint16",
            FitDataType::FitUint16 => "uint16",
            FitDataType::FitSint32 => "sint32",
            FitDataType::FitUint32 => "uint32",
            FitDataType::FitString => "string",
            FitDataType::FitF32 => "f32",
            FitDataType::FitF64 => "f64",
            FitDataType::FitU8z => "u8z",
            FitDataType::FitU16z => "u16z",
            FitDataType::FitU32z => "u32z",
            FitDataType::FitByte => "byte",
            FitDataType::FitSInt64 => "sint64",
            FitDataType::FitUint64 => "uint64",
            FitDataType::FitUint64z => "uint64z",
        }
    }

    pub fn from_name(name: &str) -> Result<FitDataType, std::io::Error> {
        match name {
            "enum" => Ok(FitDataType::FitEnum),
            "sint8" => Ok(FitDataType::FitSint8),
            "uint8" => Ok(FitDataType::FitUint8),
            "sint16" => Ok(FitDataType::FitSint16),
            "uint16" => Ok(FitDataType::FitUint16),
            "sint32" => Ok(FitDataType::FitSint32),
            "uint32" => Ok(FitDataType::FitUint32),
            "string" => Ok(FitDataType::FitString),
            "f32" => Ok(FitDataType::FitF32),
            "f64" => Ok(FitDataType::FitF64),
            "u8z" => Ok(FitDataType::FitU8z),
            "u16z" => Ok(FitDataType::FitU16z),
            "u32z" => Ok(FitDataType::FitU32z),
            "byte" => Ok(FitDataType::FitByte),
            "sint64" => Ok(FitDataType::FitSInt64),
            "uint64" => Ok(FitDataType::FitUint64),
            "uint64z" => Ok(FitDataType::FitUint64z),
            _ => Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid FIT data name"))
        }
    }
}

impl TryFrom<&FitFieldData> for u8 {
    type Error = &'static str;
    fn try_from(value: &FitFieldData) -> Result<Self, Self::Error>  {
        match value {
            FitFieldData::FitUint8(x) =>
                {
                    if !x.is_empty() {
                        Ok(x[0])
                    } else {
                        Err("Empty vec")
                    }
                },
            _ => Err("Bad cast!"),
        }
    }
}

impl TryFrom<&FitFieldData> for String {
    type Error = &'static str;
    fn try_from(value: &FitFieldData) -> Result<Self, Self::Error>  {
        match value {
            FitFieldData::FitString(x, _) =>
                {
                        Ok(x.clone())
                },
            _ => Err("Bad cast!"),
        }
    }
}

fn attempt_to_cast_first<T,U>(x: &Vec<T>) -> Result<U, &'static str>
where U: From<T>, T:Copy
{
    if let Some(v) = x.get(0) {
        Ok(U::from(v.clone()))
    } else {
        Err("Empty Vec")
    }
}

impl TryFrom<&FitFieldData> for f64 {
    type Error = &'static str;

    fn try_from(value: &FitFieldData) -> Result<Self, Self::Error> {
        match value {
            FitFieldData::FitEnum(_) => Err("Bad cast!"),
            FitFieldData::FitSint8(x) => attempt_to_cast_first(x),
            FitFieldData::FitUint8(x) => attempt_to_cast_first(x),
            FitFieldData::FitSint16(x) => attempt_to_cast_first(x),
            FitFieldData::FitUint16(x) => attempt_to_cast_first(x),
            FitFieldData::FitSint32(x) => attempt_to_cast_first(x),
            FitFieldData::FitUint32(x) => attempt_to_cast_first(x),
            FitFieldData::FitString(x,_) => if let Ok(f) = x.parse::<f64>() {
                Ok(f)
            } else {
                Err("Bad cast!")
            },
            FitFieldData::FitF32(x) => attempt_to_cast_first(x),
            FitFieldData::FitF64(x) => attempt_to_cast_first(x),
            FitFieldData::FitU8z(_) =>  Err("Bad cast!"),
            FitFieldData::FitU16z(_) =>  Err("Bad cast!"),
            FitFieldData::FitU32z(_) =>  Err("Bad cast!"),
            FitFieldData::FitByte(x) => attempt_to_cast_first(x),
            FitFieldData::FitSInt64(_) => Err("Bad cast!"),  // Needs try_into.
            FitFieldData::FitUint64(_) => Err("Bad cast!"),// Needs try_into.
            FitFieldData::FitUint64z(_) => Err("Bad cast!"),
        }
    }
}


fn contains_invalid_f32(x: &Vec<f32>) -> bool
{
    for item in x {
        let bitpattern = unsafe {
            std::mem::transmute::<f32, u32>(*item)
        };
        if bitpattern != 0xFFFFFFFF_u32 {
            return true;
        }
    }
    return false;
}

fn contains_invalid_f64(x: &Vec<f64>) -> bool
{
    for item in x {
        let bitpattern = unsafe {
            std::mem::transmute::<f64, u64>(*item)
        };
        if bitpattern != 0xFFFFFFFF_FFFFFFFF_u64 {
            return true;
        }
    }
    return false;
}

impl FitFieldData
{
    pub fn is_valid(&self) -> bool
    {
        match self {
            FitFieldData::FitEnum(x) => (!x.is_empty()) && (!x.contains(&0xFF)),
            FitFieldData::FitSint8(x) => (!x.is_empty()) && (!x.contains(&0x7F)),
            FitFieldData::FitUint8(x) => (!x.is_empty()) && (!x.contains(&0xFF)),
            FitFieldData::FitSint16(x) => (!x.is_empty()) && (!x.contains(&0x7FFF)),
            FitFieldData::FitUint16(x) => (!x.is_empty()) && (!x.contains(&0xFFFF)),
            FitFieldData::FitSint32(x) => (!x.is_empty()) && (!x.contains(&0x7FFFFFFF)),
            FitFieldData::FitUint32(x) => (!x.is_empty()) && (!x.contains(&0xFFFFFFFF)),
            FitFieldData::FitString(x, _) => (!x.is_empty()),
            FitFieldData::FitF32(x) => (!x.is_empty()) && (!contains_invalid_f32(x)),
            FitFieldData::FitF64(x) => (!x.is_empty()) && (!contains_invalid_f64(x)),
            FitFieldData::FitU8z(x) => (!x.is_empty()) && (!x.contains(&0x0_u8)),
            FitFieldData::FitU16z(x) => (!x.is_empty()) && (!x.contains(&0x0_u16)),
            FitFieldData::FitU32z(x) => (!x.is_empty()) && (!x.contains(&0x0_u32)),
            FitFieldData::FitByte(x) => (!x.is_empty()) && (!x.contains(&0xFF)),
            FitFieldData::FitSInt64(x) => (!x.is_empty()) && (!x.contains(&0x7FFFFFFF_FFFFFFFF_i64)),
            FitFieldData::FitUint64(x) => (!x.is_empty()) && (!x.contains(&0xFFFFFFFF_FFFFFFFF_u64)),
            FitFieldData::FitUint64z(x) => (!x.is_empty()) && (!x.contains(&0x0_u64)),
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

#[derive(Debug)]
pub struct Checks {
    pub reserved_bits_zero: bool,  // Error if reserved bits are non-zero.
}

impl Default for Checks {
    fn default() -> Self { Self{reserved_bits_zero: true,} }
}

#[derive(Default)]
#[derive(Debug)]
pub struct FitFileContext {
    pub data_bytes_read: u32,
    pub data_bytes_written: u32,
    pub crc: FitCrc,
    pub architecture: Option<Endianness>,
    pub field_definitions: HashMap<u8, Arc<FitDefinitionMessage> >,
    pub developer_field_definitions: HashMap<u8, Arc<FitDevDataDescription> >,
    pub timestamp: u32,
    pub checks: Checks,
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

#[derive(Debug)]
pub struct FitDevDataField {
    pub field_defn_num: u8,
    pub data: FitFieldData,
    pub description: Option< Arc<FitDevDataDescription> >,
}

#[derive(Debug,Default)]
pub struct FitDevDataDescription {
    pub field_defn_num:u8,
    pub field_name: String,
    pub scale: Option<f64>,
    pub offset: Option<f64>,
    pub units: Option<String>,
    pub is_array: Option<bool>,
    pub base_type: Option<FitDataType>,
    pub dev_data_index: u8,
}

#[derive(Debug,Default)]
pub struct FitDataMessage {
    pub global_message_number: u16,
    pub local_message_type: u8,
    pub timestamp: Option<u32>,    // Only set for compressed messages.
    pub fields: Vec<FitDataField>,
    pub dev_fields: Vec<FitDevDataField>,
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

