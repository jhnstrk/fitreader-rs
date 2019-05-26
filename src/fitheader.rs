

// std imports
use std::fs::{File};
use std::io::{BufReader, Read, BufWriter, Write};
use std::sync::Arc;

use byteorder::{LittleEndian,  ReadBytesExt, WriteBytesExt};

// Local imports
use crate::fittypes::{ FitFileContext, FitFileHeader};
use crate::fitcrc;


pub fn read_global_header(context: &mut FitFileContext, reader: &mut BufReader<File>) -> Result< Arc<FitFileHeader>, std::io::Error> {

    let mut header_buf: [u8; 12] = [0; 12];
    reader.read_exact(&mut header_buf)?;


    let mut header_rdr = std::io::Cursor::new(header_buf);

    let mut header: FitFileHeader = Default::default();

    header.header_size = header_rdr.read_u8()?;
    header.protocol_version = header_rdr.read_u8()?;
    header.profile_version = header_rdr.read_u16::<LittleEndian>()?;
    header.data_size = header_rdr.read_u32::<LittleEndian>()?;
    header_rdr.read_exact(&mut header.type_signature )?;

    let expected_signature : [u8;4] = ['.' as u8, 'F' as u8, 'I' as u8, 'T' as u8 ];
    if header.type_signature != expected_signature {
        return Err( std::io::Error::new(std::io::ErrorKind::Other, "Invalid FIT signature"));
    }

    context.bytes_read = 12;

    // CRC is not present in older FIT formats.
    if header.header_size >= 14 {
        header.crc = reader.read_u16::<LittleEndian>().unwrap();
        context.bytes_read += 2;

        let actual_crc = fitcrc::compute(&header_buf);
        //println!("Actual: {} Expected: {}", actual_crc, my_file.header.crc);
        if (header.crc != 0) && (actual_crc != header.crc) {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Header CRC is invalid"));
        }
    }

    if header.header_size as u32 > context.bytes_read {
        while header.header_size as u32 > context.bytes_read {
            reader.read_u8()?;
        }
    }
    Ok( Arc::new(header) )
}

pub fn write_global_header(context: &mut FitFileContext, writer: &mut BufWriter<File>, header: &FitFileHeader)
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

    context.bytes_written = 12;

    // CRC is not present in older FIT formats.
    if header.header_size >= 14 {
        let crc = fitcrc::compute(&header_buf);
        writer.write_u16::<LittleEndian>(crc)?;
        context.bytes_written += 2;
    }

    if header.header_size as u32 > 14 {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Header size is invalid"));
    }
    Ok( () )
}
