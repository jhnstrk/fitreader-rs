
// std imports
use std::io::{Read, Write};

use crate::fittypes::{FitFieldData, FitDataMessage, FitDataField, FitFileContext};
use crate::fitwrite::{fit_write_u8};

use crate::fitfield::{read_fit_field, write_fit_field};

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
        let data_size = field.data_type.unwrap().data_size();
        let count:u8 = match data_size {
            0 => field.size_in_bytes,
            _ => field.size_in_bytes / data_size };

        let field_value_data = read_fit_field(context, reader,
                                              field.data_type.unwrap(), count)?;

        let field_value = FitDataField {
            field_defn_num: field.field_defn_num,
            data: field_value_data };
        mesg.fields.push(field_value);
    }


    debug!("Data message: {:?}", mesg);

    Ok(mesg)
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

