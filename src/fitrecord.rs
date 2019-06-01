// std imports
use std::io::{Read, Write};

use serde_json::{Value, Map};

use crate::fittypes::{ Endianness, FitFileContext,
                       FitFieldData, FitRecord,
                        INVALID_U32};
use crate::fitread::{fit_read_u8};

use crate::profile::ProfileData;
use crate::fitheader;
use crate::fitdatamesg;
use crate::fitdefnmesg;
use byteorder::{LittleEndian, WriteBytesExt};

fn handle_fit_enum_value( x: Value, type_name: &str, p: &ProfileData )-> Value{
    match &x {
        Value::Null => x,
        Value::Number(v) => {

            if let Some(field_value) = v.as_u64() {
                match p.value_name(type_name, field_value as u32) {
                    None => { x },
                    Some(str) => { Value::from(str) },
                }
            } else {
                x
            }
        }
        Value::Array(xa) => {
            let mut ret = Vec::new();
            for v in xa {
                ret.push(handle_fit_enum_value(v.clone(), type_name, p));
            }
            Value::from(ret)
        },
        _ => x,
    }
}

fn handle_fit_scale_offset( x: Value, scale: &Option<f64>, offset: &Option<f64> )-> Value{
    if (scale.is_none() && offset.is_none()) {
        return x;
    }

    match x {
        Value::Null => x,
        Value::Number(v) => {

            if let Some(field_value) = v.as_f64() {
                let mut value_copy = field_value;
                if let Some(offset_f) = offset {
                    value_copy = value_copy - offset_f;
                }
                if let Some(scale_f) = scale {
                    value_copy = value_copy / scale_f;
                }

                Value::from(value_copy)
            } else {
                Value::Number(v)
            }
        }
        Value::Array(xa) => {
            let mut ret = Vec::new();
            for v in xa {
                ret.push(handle_fit_scale_offset(v, scale, offset));
            }
            Value::from(ret)
        },
        _ => x,
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

fn to_json(rec: &FitRecord, pf: &ProfileData) -> (String, Value){
    match rec {
        FitRecord::HeaderRecord(header) => {
            let mut map = Map::new();
            map.insert("data_size".to_string(), Value::from(header.data_size));
            map.insert("protocol_version".to_string(), Value::from(header.protocol_version));
            map.insert("profile_version".to_string(), Value::from(header.profile_version));
            ("Header".to_string(), Value::Object(map))},
        FitRecord::DataRecord(data_message) => {
            let mut map = Map::new();
            if !data_message.timestamp.is_none() {
                let value = data_message.timestamp.unwrap();
                if value != INVALID_U32 {
                    map.insert(String::from("timestamp"), Value::from(value));
                }
            }
            let message = pf.get_message(data_message.global_message_number);
            let mut field_vec: Vec<Value> = vec!();
            for ifield in &data_message.fields {
                let field_name: String;
                let mut field_units = None;
                let mut field_desc = None;
                let mut field_type = None;
                let mut field_scale = None;
                let mut field_offset = None;
                if message.is_some() {
                    field_desc = message.unwrap().find_field(ifield.field_defn_num);
                }
                if field_desc.is_some() {
                    field_name = field_desc.unwrap().field_name.clone();
                    field_units = field_desc.unwrap().units.clone();
                    field_type = Some(field_desc.unwrap().field_type.clone());
                    field_scale = field_desc.unwrap().scale.clone();
                    field_offset = field_desc.unwrap().offset.clone();
                } else {
                    let field_string = format!("Field_{}", ifield.field_defn_num);
                    field_name = field_string;
                }
                let mut value =
                    match &ifield.data {
                        FitFieldData::FitEnum(x)  => handle_fit_value(x),
                        FitFieldData::FitSint8(x) => handle_fit_value(x),
                        FitFieldData::FitUint8(x) => handle_fit_value(x),
                        FitFieldData::FitSint16(x) => handle_fit_value(x),
                        FitFieldData::FitUint16(x) => handle_fit_value(x),
                        FitFieldData::FitSint32(x) => handle_fit_value(x),
                        FitFieldData::FitUint32(x) => handle_fit_value(x),
                        FitFieldData::FitString(x, _) => Value::from(x.as_str()),
                        FitFieldData::FitF32(x) => handle_fit_value(x),
                        FitFieldData::FitF64(x) => handle_fit_value(x),
                        FitFieldData::FitU8z(x) => handle_fit_value(x),
                        FitFieldData::FitU16z(x) => handle_fit_value(x),
                        FitFieldData::FitU32z(x) => handle_fit_value(x),
                        FitFieldData::FitByte(x) => handle_fit_value(x),
                        FitFieldData::FitSInt64(x) => handle_fit_value(x),
                        FitFieldData::FitUint64(x) => handle_fit_value(x),
                        FitFieldData::FitUint64z(x) => handle_fit_value(x),
                    };

                match field_type {
                    Some(ft) => {
                        value = handle_fit_enum_value(value, &ft, pf)
                    },
                    None =>  {},
                }
                value = handle_fit_scale_offset(value,  &field_scale, &field_offset );
                let mut field_map = Map::new();
                field_map.insert("name".to_string(), Value::from(field_name));
                field_map.insert("value".to_string(), value);
                if field_units.is_some() {
                    field_map.insert("units".to_string(), Value::from(field_units.unwrap()));
                }
                field_vec.push(Value::from(field_map));
            }
            let message_name = if message.is_some() {
                message.unwrap().message_name.clone()
            } else {
                format!("Message_{}", data_message.global_message_number)
            };
            map.insert("message".to_string(), Value::from(message_name));
            map.insert( "fields".to_string(), Value::Array(field_vec));
            return ("data".to_string(), Value::Object(map));
        },
        FitRecord::DefinitionMessage(defn_message) => {
            let mut map = Map::new();
            match defn_message.architecture {
                Endianness::Little => {
                    map.insert("architecture".to_string(), Value::from("Little"));
                },
                Endianness::Big => {
                    map.insert("architecture".to_string(), Value::from("Big"));
                },
            }
            map.insert("local_message_type".to_string(), Value::from(defn_message.local_message_type));
            map.insert("global_message_number".to_string(), Value::from(defn_message.global_message_number));
            let message = pf.get_message(defn_message.global_message_number);

            let mut field_vec: Vec<Value> = vec!();
            for ifield in &defn_message.field_defns {
                let mut field_desc = Option::None;
                if message.is_some() {
                    field_desc = message.unwrap().find_field(ifield.field_defn_num);
                }
                let field_name: String;
                if field_desc.is_some() {
                    field_name = field_desc.unwrap().field_name.clone();
                } else {
                    let field_string = format!("Field_{}", ifield.field_defn_num);
                    field_name = field_string;
                }
                let mut field_map = Map::new();
                field_map.insert("name".to_string(), Value::from(field_name));
                field_map.insert("size".to_string(), Value::from(ifield.size_in_bytes));
                field_vec.push(Value::from(field_map));
            }
            map.insert("field_defns".to_string(), Value::from(field_vec));
            return ("definition".to_string(), Value::Object(map));
        }
        FitRecord::EndOfFile(crc) => {
            let mut map = Map::new();
            map.insert("crc".to_string(), Value::from(*crc));
            return ("EOF".to_string(), Value::Object(map));
        }
    }
}

pub fn print_rec(rec: &FitRecord, pf: &ProfileData) {
    let (name, value) = to_json(rec, pf);
    println!("{}: {}", name, value);
}

pub fn read_record(context: &mut FitFileContext, reader: &mut Read) -> Result< FitRecord, std::io::Error> {
    let record_hdr = fit_read_u8(context, reader)?;
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
                fitdefnmesg::read_definition_message( context, reader, local_message_type, is_developer)?));
        } else {
            // Data message
            return Ok(FitRecord::DataRecord(
                fitdatamesg::read_data_message( context, reader, local_message_type, None)?));
        }
    } else {
        // Compressed timestamp header
        println!("Compressed message");
        let local_message_type = (record_hdr >> 5) & 0x03;
        let time_offset = (record_hdr & 0x1F) as u32;

        let prev_time_stamp = context.timestamp;
        let new_timestamp = if time_offset >= (prev_time_stamp & 0x1fu32) {
            (prev_time_stamp & 0xFFFFFFE0) + time_offset
        } else {
            (prev_time_stamp & 0xFFFFFFE0) + time_offset+ 0x20
        };
        // Data message
        return Ok(FitRecord::DataRecord(
            fitdatamesg::read_data_message( context, reader, local_message_type, Some(new_timestamp))?) );
    }
}

pub fn write_record(context: &mut FitFileContext, writer: &mut Write, rec: &FitRecord)
                    -> Result< (), std::io::Error>
{
    match rec {
        FitRecord::HeaderRecord(header)
            => fitheader::write_global_header(context, writer, header),
        FitRecord::DefinitionMessage(defn) =>
            fitdefnmesg::write_definition_message(context, writer, defn.as_ref()),
        FitRecord::DataRecord(data_message) =>
            fitdatamesg::write_data_message(context, writer, data_message),
        FitRecord::EndOfFile(crc) => {
            writer.write_u16::<LittleEndian>(*crc)
        }
    }
}
