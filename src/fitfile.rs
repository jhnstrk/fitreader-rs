

// std imports
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write, Seek};

use std::sync::Arc;

use byteorder::{LittleEndian,  ReadBytesExt, WriteBytesExt};

use crate::profile;

use crate::fittypes::{ Endianness, FitFile, FitRecord };
use crate::fitcrc;

use crate::fitheader::{read_global_header};
use crate::fitrecord::{read_record, write_rec, print_rec};
use crate::fitcheck::{check_rec};

pub fn read_file(path: &str) -> std::io::Result<FitFile> {
    let mut my_file: FitFile = Default::default();
    let p = profile::build_profile()?;

    println!("Opening file: {}", path);
    let file = File::open(path)?;

    let mut reader = BufReader::new(file);

    println!("Reading header from: {}", path);
    my_file.header = read_global_header(&mut my_file, &mut reader)?;


    let file_out = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open("/tmp/fit_out.fit")?;

    let mut writer = BufWriter::new(file_out);
    let mut out_file: FitFile = Default::default();
    out_file.context.architecture = Some(Endianness::Little);
    out_file.header = Arc::new((*my_file.header).clone() );

    let new_header_rec = FitRecord::HeaderRecord(out_file.header.clone());
    write_rec(&mut out_file, &mut writer, &new_header_rec)?;

    let mut num_rec = 1;  // Count the header as one record.

    // Read data, total file size is header + data + crc
    let len_to_read = my_file.header.header_size as u32 + my_file.header.data_size;
    while my_file.context.bytes_read < len_to_read {
        let rec = read_record(&mut my_file, &mut reader);
        match rec {
            Ok(v) => {
                print_rec(&v, &p);
                match check_rec(&my_file, &v ) {
                    Ok(_) => {write_rec(&mut out_file, &mut writer, &v) ?;},
                    Err(e) => println!("Skipping bad values in rec {}", e),
                }

            },
            Err(e) => println!("Skipping bad rec {}", e),
        }
        num_rec = num_rec + 1;
    }

    writer.flush()?;
    // Update data size, write new header.

    let mut new_header = (*out_file.header).clone();
    new_header.data_size = out_file.context.bytes_written - new_header.header_size as u32;
    writer.seek(std::io::SeekFrom::Start(0) )?;
    write_rec(&mut out_file, &mut writer, &FitRecord::HeaderRecord(Arc::new(new_header)))?;
    writer.flush()?;  // Required.

    // compute new crc
    let crc_out = fitcrc::crc_for_file(writer.get_mut() )?;  // "inadvisable"
    writer.seek(std::io::SeekFrom::End(0) )?;
    writer.write_u16::<LittleEndian>(crc_out)?;
    println!("Info: Read {:} records from {:} bytes", num_rec, my_file.context.bytes_read );

    // Read directly as we don't want the crc value included in the crc computation.
    let crc = reader.read_u16::<LittleEndian>()?;
    println!("CRC: Computed 0x{:x}, Provided 0x{:x}", my_file.context.crc.digest(), crc);

    Ok(my_file)
}
