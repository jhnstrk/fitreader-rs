
// std imports
use std::io::{Read, Write};

use crate::fittypes::{FitDataType, FitFieldData, FitDataMessage, FitDataField, FitFileContext,
                      FitDevDataDescription, FitDevDataField};
use crate::fitwrite::{fit_write_u8};

use crate::fitfield::{read_fit_field, write_fit_field};
use std::convert::TryFrom;
use std::sync::Arc;

pub fn read_data_message( context: &mut FitFileContext, reader: &mut Read,
                      local_message_type: u8, timestamp: Option<u32>) -> Result< FitDataMessage, std::io::Error> {

    debug!("Data message, local ID: {:} at byte {:}", local_message_type, context.data_bytes_read);

    let defn_mesg=
        match context.field_definitions.get(&local_message_type) {
            Some(v) => v,
            None => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Field id not found")),
        }.clone();

    let mut mesg = FitDataMessage{
        global_message_number: defn_mesg.global_message_number,
        local_message_type,
        timestamp,
        ..Default::default()
    };


    for field in &defn_mesg.field_defns {
        let data_size = field.data_type.unwrap().data_size();
        let count:u8 = match data_size {
            0 => field.size_in_bytes,
            _ => field.size_in_bytes / data_size };

        context.architecture = Some(defn_mesg.architecture);

        let field_value_data = read_fit_field(context, reader,
                                              field.data_type.unwrap(), count)?;
        // If this is a timestamp, then update the file timestamp, for any compressed messages.
        if field.field_defn_num == 253 {
            match field_value_data.clone() {
                FitFieldData::FitUint32(value) => context.timestamp = value[0],
                _ => warn!("Warning, bad timestamp type")
            }
        }

        let field_value = FitDataField {
            field_defn_num: field.field_defn_num,
            data: field_value_data };
        mesg.fields.push(field_value);

    }

    context.architecture = Some(defn_mesg.architecture);

    for field in &defn_mesg.dev_field_defns {

        println!("Reading dev field {:?}", context.developer_field_definitions);
        let desc2 = match context.developer_field_definitions.get(
            &field.dev_data_index) {
            None => {None },
            Some(x) => {Some(x.clone())},
        };

        if let Some(desc) = desc2
        {
            println!("Reading field");
            let base_type = desc.base_type.unwrap();
            let data_size = base_type.data_size();
            let count:u8 = match data_size {
                0 => field.size_in_bytes,
                _ => field.size_in_bytes / data_size };

            let field_value_data = read_fit_field(context, reader,
                                                  base_type, count)?;

            let field_value = FitDevDataField {
                field_defn_num: field.field_defn_num,
                data: field_value_data,
                description: Some( desc.clone() ),
            };
            mesg.dev_fields.push(field_value);
        } else {
            // Field description not found. Load as bytes.
            warn!("Unknown dev field index={} defn_num={}", field.dev_data_index, field.field_defn_num);
            let base_type = FitDataType::FitByte;
            let count = field.size_in_bytes;
            let field_value_data = read_fit_field(context, reader,
                                                  base_type, count)?;

            let field_value = FitDevDataField {
                field_defn_num: field.field_defn_num,
                data: field_value_data,
                description: None,
            };
            mesg.dev_fields.push(field_value);
        }
    }


    debug!("Data message: {:?}", mesg);

    const FIELD_DESCRIPTION: u16 = 206;

    if defn_mesg.global_message_number == FIELD_DESCRIPTION {
        add_dev_field_description( context, &mesg );
    }

    Ok(mesg)
}

fn add_dev_field_description( context: &mut FitFileContext, mesg: &FitDataMessage )
{
    // ARRAY = 4,
    // 5	components
    //9	bits
    //10	accumulate
    //13	fit_base_unit_id
    //14	native_mesg_num
    //15	native_field_num

    let mut has_field_defn = false;

    const DEV_DATA_INDEX: u8 = 0;

    let mut dev_data_desc: FitDevDataDescription = Default::default();
    for ifield in &mesg.fields {
        println!("ifield {:?}",ifield);
        match ifield.field_defn_num {
            DEV_DATA_INDEX => { // dev_data_index
                if let Ok(x) = u8::try_from(&ifield.data) {
                    dev_data_desc.dev_data_index = x;
                } else {
                    warn!("Bad type for dev_data_index");
                }
            },
            1 => { // field_defn_num
                println!("field_defn_num");
                if let  Ok(x) = u8::try_from(&ifield.data) {
                    has_field_defn = true;
                    dev_data_desc.field_defn_num = x;
                } else {
                    warn!("Bad type for field_defn_num");
                }
            },
            2 => { // base_type_id
                if  let Ok(x) = u8::try_from(&ifield.data) {
                    dev_data_desc.base_type = Some(FitDataType::from_type_id(x).unwrap());
                } else {
                    warn!("Bad type for base_type");
                }
            },
            3 => { // name
                if let Ok(x) = String::try_from(&ifield.data) {
                    dev_data_desc.field_name = x.clone();
                } else {
                    warn!("Bad type for field_name");
                }
            },
            6 => { // Scale
                if let Ok(x) = f64::try_from(&ifield.data) {
                    dev_data_desc.scale = Some(x);
                } else {
                    warn!("Bad type for scale");
                }
            },
            7 => { // Offset
                if let Ok(x) = f64::try_from(&ifield.data) {
                    dev_data_desc.offset = Some(x);
                } else {
                    warn!("Bad type for offset");
                }
            },
            8 => { // units
                if let Ok(x) = String::try_from(&ifield.data) {
                    dev_data_desc.units = Some(x.clone());
                } else {
                    warn!("Bad type for units");
                }
            },
            _ => {
                println!("Other value");
            }
        }
    }
    if has_field_defn {
        println!("Adding defn num");
        context.developer_field_definitions.insert(dev_data_desc.field_defn_num, Arc::new(dev_data_desc));
    } else {
        println!("Dev field has no defn num");
    }
}

pub fn write_data_message( context: &mut FitFileContext, writer: &mut Write, mesg: &FitDataMessage)
                       -> Result< (), std::io::Error>
{
    let is_compressed = mesg.timestamp.is_some();
    let record_hdr = if is_compressed {
        assert!(mesg.local_message_type <= 0x03);

        let prev_time_stamp = context.timestamp;
        let new_timestamp = mesg.timestamp.unwrap();
        assert!((prev_time_stamp & 0xFFFFFFE0) < new_timestamp);

        if (new_timestamp - prev_time_stamp) > 0x1f {
            warn!("Warning: compressed timestamp overflow");
        }
        let time_offset = (new_timestamp & 0x1F) as u8;

        0x80u8 | ((mesg.local_message_type & 0x3 ) << 5) | time_offset
    }else {
        assert!(mesg.local_message_type <= 0x0F);
        mesg.local_message_type
    };

    fit_write_u8(context, writer, record_hdr)?;  // Write header byte

    let defn = context.field_definitions.get(&mesg.local_message_type);
    match defn {
        None => {debug!("Using defaults");},
        Some(x) => {context.architecture = Some(x.architecture);},
    }
    for field in &mesg.fields {
        write_fit_field(context, writer, &field.data)?;

        // If this is a timestamp, then update the file timestamp, for any compressed messages.
        if field.field_defn_num == 253 {
            match &field.data {
                FitFieldData::FitUint32(value) => context.timestamp = value[0],
                _ => warn!("Warning, bad timestamp type")
            }
        }
    }

    Ok( () )
}

