

// std imports
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write, Seek};

use std::sync::Arc;

use byteorder::{LittleEndian,  ReadBytesExt, WriteBytesExt};

use crate::profile;

use crate::fittypes::{ Endianness, FitFile, FitFileContext, FitRecord };
use crate::fitcrc;

use crate::fitheader::{read_global_header};
use crate::fitrecord::{read_record, write_rec, print_rec};
use crate::fitcheck::{check_rec};

use std::io::{ Read };

impl FitFile {
    pub fn new() -> FitFile {
        return Default::default();
    }
}

struct FitFileReader<R: Read> {
    source: R
}

impl<R: Read> FitFileReader<R> {
    pub fn new(source: R) -> FitFileReader<R> {
        return FitFileReader{source};
    }

    pub fn read_all() -> std::io::Result<FitFile>  {
        //TOOD
        return Ok(FitFile::new());
    }
}


pub fn read_file_filename(path: &str) -> std::io::Result<FitFile> {
    println!("Opening file: {}", path);
    let mut file = File::open(path)?;

    return read_file_read(&mut file);
}

pub fn read_file_read(source: &mut Read) -> std::io::Result<FitFile> {
    let mut my_file: FitFile = FitFile::new();
    let mut context: FitFileContext = Default::default();

    let mut reader = BufReader::new(source);
    my_file.header = read_global_header(&mut context, &mut reader)?;

    // Read data, total file size is header + data + crc
    let len_to_read = my_file.header.header_size as u32 + my_file.header.data_size;
    while context.bytes_read < len_to_read {
        let rec = read_record(&mut context, &mut reader);
        match rec {
            Ok(v) => {
                my_file.records.push(v);

            },
            Err(e) => println!("Skipping bad rec {}", e),
        }
    }

    // Read directly as we don't want the crc value included in the crc computation.
    let crc = reader.read_u16::<LittleEndian>()?;
    println!("CRC: Computed 0x{:x}, Provided 0x{:x}", context.crc.digest(), crc);

    Ok(my_file)
}

pub fn read_file(path: &str) -> std::io::Result<FitFile> {
    let mut my_file: FitFile = FitFile::new();
    let mut context: FitFileContext = Default::default();
    let p = profile::build_profile()?;

    println!("Opening file: {}", path);
    let file = File::open(path)?;

    let mut reader = BufReader::new(file);

    println!("Reading header from: {}", path);
    my_file.header = read_global_header(&mut context, &mut reader)?;


    let file_out = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open("/tmp/fit_out.fit")?;

    let mut writer = BufWriter::new(file_out);
    let mut out_context: FitFileContext = Default::default();
    out_context.architecture = Some(Endianness::Little);
    let mut out_header = (*my_file.header).clone();

    let new_header_rec = FitRecord::HeaderRecord(Arc::new(out_header.clone() ));
    write_rec(&mut out_context, &mut writer, &new_header_rec)?;

    let mut num_rec = 1;  // Count the header as one record.

    // Read data, total file size is header + data + crc
    let len_to_read = my_file.header.header_size as u32 + my_file.header.data_size;
    while context.bytes_read < len_to_read {
        let rec = read_record(&mut context, &mut reader);
        match rec {
            Ok(v) => {
                print_rec(&v, &p);
                match check_rec(&context, &v ) {
                    Ok(_) => {write_rec(&mut out_context, &mut writer, &v) ?;},
                    Err(e) => println!("Skipping bad values in rec {}", e),
                }

            },
            Err(e) => println!("Skipping bad rec {}", e),
        }
        num_rec = num_rec + 1;
    }

    writer.flush()?;
    // Update data size, write new header.

    out_header.data_size = out_context.bytes_written - out_header.header_size as u32;
    writer.seek(std::io::SeekFrom::Start(0) )?;
    write_rec(&mut out_context, &mut writer, &FitRecord::HeaderRecord(Arc::new(out_header)))?;
    writer.flush()?;  // Required.

    // compute new crc
    let crc_out = fitcrc::crc_for_file(writer.get_mut() )?;  // "inadvisable"
    writer.seek(std::io::SeekFrom::End(0) )?;
    writer.write_u16::<LittleEndian>(crc_out)?;
    println!("Info: Read {:} records from {:} bytes", num_rec, context.bytes_read );

    // Read directly as we don't want the crc value included in the crc computation.
    let crc = reader.read_u16::<LittleEndian>()?;
    println!("CRC: Computed 0x{:x}, Provided 0x{:x}", context.crc.digest(), crc);

    Ok(my_file)
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_read() {
        // Take a sample fit file, last 2 bytes are the 16-bit crc in LittleEndian.
        // Pop them off, compare the computed CRC.
        // The sample file is settings.fit from the FitSDKRelease_20.90.00
        let settings_fit = base64::decode(
            "DBBHAEQAAAAuRklUQAABAAAEAQKEAgKEAwSMAAEAAAABA9wAAeJAA\
                   kAAAQADBQQChAEBAAIBAgMBAgUBAAADhAEcvgBAAAEABAEBAosAAGQ5UA==").unwrap();

//        let file_data = FitFile::read(settings_fit);
    }
}
