
// std imports
use std::io::{Read, Write};
use std::sync::Arc;

use crate::fittypes::{ Endianness, FitFileContext,
                       FitFieldDefinition, FitDeveloperFieldDefinition,
                                   FitDefinitionMessage,
                                   int_to_fit_data_type, fit_data_type_to_int, fit_data_size};

use crate::fitread::{fit_read_u8, fit_read_u16};
use crate::fitwrite::{fit_write_u8, fit_write_u16};


fn read_field_defn( context: &mut FitFileContext, reader: &mut Read)
                    -> Result< Arc<FitFieldDefinition>, std::io::Error> {
    let field_defn_num = fit_read_u8(context, reader)?;
    if field_defn_num == 0xFF {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid field: defn_num=255"));
    }
    let size_in_bytes = fit_read_u8(context, reader)?;
    if size_in_bytes == 0x0 {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid field: size=0"));
    }
    let base_type = fit_read_u8(context, reader)?;

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

fn write_field_defn( context: &mut FitFileContext, writer: &mut Write, field_defn: &FitFieldDefinition )
                     -> Result< (), std::io::Error>
{
    let base_type_num = fit_data_type_to_int(&field_defn.data_type.unwrap() )?;
    let base_type_is_endian = fit_data_size( field_defn.data_type.unwrap() )? > 1;
    let base_type = base_type_num | ( if base_type_is_endian {0x80} else {0x00} );

    fit_write_u8(context, writer, field_defn.field_defn_num)?;
    fit_write_u8(context, writer, field_defn.size_in_bytes)?;
    fit_write_u8(context, writer, base_type)?;

    Ok( () )
}


fn read_dev_field_defn( context: &mut FitFileContext, reader: &mut Read)
                        -> Result< Arc<FitDeveloperFieldDefinition>, std::io::Error> {
    let field_defn_num = fit_read_u8(context, reader)?;
    let size_in_bytes = fit_read_u8(context, reader)?;
    let dev_data_index = fit_read_u8(context, reader)?;

    let mut field_defn: FitDeveloperFieldDefinition = Default::default();
    field_defn.field_defn_num = field_defn_num;
    field_defn.size_in_bytes = size_in_bytes;
    field_defn.dev_data_index = dev_data_index;
    Ok(Arc::new(field_defn))
}

fn write_dev_field_defn( context: &mut FitFileContext, writer: &mut Write, field_defn: &FitDeveloperFieldDefinition )
                         -> Result< (), std::io::Error>
{
    fit_write_u8(context, writer, field_defn.field_defn_num)?;
    fit_write_u8(context, writer, field_defn.size_in_bytes)?;
    fit_write_u8(context, writer, field_defn.dev_data_index)?;
    Ok( () )
}

pub fn read_definition_message( context: &mut FitFileContext, reader: &mut Read,
                            local_message_type: u8, is_developer: bool)
                            -> Result< Arc<FitDefinitionMessage>, std::io::Error> {
    let _reserved0 = fit_read_u8(context, reader)?;  // Read and discard a reserved byte

    let architecture = fit_read_u8(context, reader)?;
    let endian:Endianness = if architecture == 1 { Endianness::Big } else { Endianness::Little };

    context.architecture = Some(endian);

    let global_message_number = fit_read_u16(context, reader)?;
    let number_of_fields = fit_read_u8(context, reader)?;

    println!("Definition message: Local ID: {:}, Global ID = {:}, Num. of fields: {}, offset {}",
             local_message_type, global_message_number, number_of_fields, context.bytes_read);

    let mut defn_mesg = FitDefinitionMessage {
        architecture: endian,
        global_message_number,
        local_message_type,
        ..Default::default()
    };

    for _ifield in 0..number_of_fields {
        defn_mesg.field_defns.push( read_field_defn(context, reader)? );
        println!("Field {}: {:?}", _ifield, defn_mesg.field_defns.last().unwrap());
    }

    if is_developer {
        let number_of_dev_fields = fit_read_u8(context, reader)?;
        for _ifield in 0..number_of_dev_fields {
            defn_mesg.dev_field_defns.push( read_dev_field_defn(context, reader)? );
        }
    }

    let v = Arc::new(defn_mesg);
    context.field_definitions.insert(local_message_type, v.clone());

    Ok(v)
}

pub fn write_definition_message( context: &mut FitFileContext, writer: &mut Write, defn_mesg: &FitDefinitionMessage)
                             -> Result< (), std::io::Error>
{
    let is_developer = !defn_mesg.dev_field_defns.is_empty();
    assert!(defn_mesg.local_message_type <= 0x0F);

    let record_hdr = defn_mesg.local_message_type |
        (if is_developer {0x20} else {0x0}) |
        0x40; // Definition message

    fit_write_u8(context, writer, record_hdr)?;  // Write header byte
    fit_write_u8(context, writer, 0u8)?;  // Write a reserved byte

    match context.architecture {
        Some(x) => {
            match x {
                Endianness::Big => fit_write_u8(context, writer, 1u8)?,
                Endianness::Little => fit_write_u8(context, writer, 0u8)?,
            };
        }

        None => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Endianness not set")),
    };

    fit_write_u16(context, writer, defn_mesg.global_message_number)?;
    fit_write_u8(context, writer, defn_mesg.field_defns.len() as u8)?;

    println!("Writing definition message: Local ID: {:}, Global ID = {:}, Num. of fields: {}, offset {}",
             defn_mesg.local_message_type, defn_mesg.global_message_number,
             defn_mesg.field_defns.len(), context.bytes_written);

    for field in &defn_mesg.field_defns {
        write_field_defn(context, writer, field)?;
    }

    if is_developer {
        fit_write_u8(context, writer, defn_mesg.dev_field_defns.len() as u8)?;
        for field in &defn_mesg.dev_field_defns {
            write_dev_field_defn(context, writer, field)?;
        }
    }

//    let v = Arc::new(defn_mesg);
//    context.field_definitions.insert(local_message_type, v.clone());

    Ok(())
}
