

// std imports
use std::io::{Read, Write};

use byteorder::{LittleEndian,  ReadBytesExt, WriteBytesExt};

// Local imports
use crate::fittypes::{FitFileContext, FitFileHeader, Endianness};
use crate::fitcrc;
use crate::fitread::{fit_read_u8, fit_read_u16};
use crate::fitwrite::{fit_write_u8, fit_write_u16};

pub fn read_global_header(context: &mut FitFileContext, reader: &mut Read) -> std::io::Result< FitFileHeader > {

    let mut header_buf: [u8; 12] = [0; 12];

    for _i in 0..12 {
        header_buf[_i] = fit_read_u8(context, reader)?;
    }

    let header_buf = header_buf;

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

    context.data_bytes_read = 12;

    // CRC is not present in older FIT formats.
    if header.header_size >= 14 {
        context.architecture = Some(Endianness::Little);
        header.crc = fit_read_u16(context, reader)?;
        context.data_bytes_read += 2;

        let actual_crc = fitcrc::compute(&header_buf);
        //debug!("Actual: {} Expected: {}", actual_crc, my_file.header.crc);
        if (header.crc != 0) && (actual_crc != header.crc) {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Header CRC is invalid"));
        }
    }

    if header.header_size as u32 > context.data_bytes_read {
        while header.header_size as u32 > context.data_bytes_read {
            fit_read_u8(context, reader)?;
        }
    }
    context.data_bytes_read = 0;
    Ok( header )
}

pub fn write_global_header(context: &mut FitFileContext, writer: &mut Write, header: &FitFileHeader)
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
    context.crc.reset();
    context.architecture = Some(Endianness::Little);
    for _i in 0..12 {
        fit_write_u8(context, writer, header_buf[_i])?;
    }

    // CRC is not present in older FIT formats.
    if header.header_size >= 14 {
        let crc = fitcrc::compute(&header_buf);
        fit_write_u16(context, writer, crc)?;
    }

    context.data_bytes_written = 0;

    if header.header_size as u32 > 14 {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Header size is invalid"));
    }
    Ok( () )
}
