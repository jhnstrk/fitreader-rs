

// std imports
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write, Seek};

use byteorder::{LittleEndian,  ReadBytesExt, WriteBytesExt};

use crate::profile;

use crate::fittypes::{Endianness, FitFile, FitFileContext, FitRecord, FitFileHeader};
use crate::fitcrc;

use crate::fitheader::{read_global_header};
use crate::fitrecord::{read_record, write_record, print_rec};
use crate::fitcheck::{check_rec};


impl FitFile {
    pub fn new() -> FitFile {
        return Default::default();
    }
}

struct FitFileReader<R: Read> {
    source: R,
    context: FitFileContext,
    data_size: Option<u32>,
}

impl<R: Read> FitFileReader<R> {
    pub fn new(source: R) -> FitFileReader<R> {
        return FitFileReader{source,
            context: Default::default(),
            data_size: None};
    }

    pub fn read_global_header(&mut self) -> std::io::Result< FitFileHeader > {
        let header = read_global_header(&mut self.context, &mut self.source)?;
        self.data_size = Some(header.data_size);
        return Ok(header);
    }

    pub fn read_next(&mut self) -> std::io::Result<FitRecord>  {
        if self.data_size.is_none() {
            return Ok(FitRecord::HeaderRecord(
                read_global_header(&mut self.context, &mut self.source)? ) );
        }
        println!("Read: {} len: {}", self.context.data_bytes_read , self.data_size.unwrap());
        if self.data_size.is_some() && (self.context.data_bytes_read < self.data_size.unwrap()) {
            return read_record(&mut self.context, &mut self.source);
        } else {
            let file_crc = self.source.read_u16::<LittleEndian>()?;
            let computed_crc = self.context.crc.digest();
            if file_crc == computed_crc {
                return Ok(FitRecord::EndOfFile(file_crc));
            } else {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "Bad CRC"));
            }
        }

    }

    pub fn source(&self) -> &R   {&self.source}
}

struct FitFileWriter<W: Write + Seek> {
    target: W,
    context: FitFileContext,
    header: FitFileHeader,
}

impl<W: Read + Write + Seek> FitFileWriter<W> {
    pub fn new(target: W) -> FitFileWriter<W> {
        FitFileWriter{target, context: Default::default(),
        header: Default::default(),
        }
    }

    pub fn write_global_header(&mut self, header: &FitFileHeader) -> std::io::Result<()> {
        self.header = header.clone();
        write_record(&mut self.context, &mut self.target,
                     &FitRecord::HeaderRecord(self.header))
    }

    pub fn write_next(&mut self, rec: &FitRecord) -> std::io::Result<()>  {
        write_record(&mut self.context, &mut self.target, rec)
    }

    pub fn finalize(&mut self) -> std::io::Result<()>  {
        self.target.flush()?;
        // Update data size, write new header.
        self.header.data_size = self.context.data_bytes_written;
        self.target.seek(std::io::SeekFrom::Start(0) )?;
        write_record(&mut self.context, &mut self.target,
                     &FitRecord::HeaderRecord(self.header))?;
        self.target.flush()?;  // Required.

        // compute new crc
        let crc_out = fitcrc::crc_for_file(&mut self.target )?;  // "inadvisable"
        self.target.seek(std::io::SeekFrom::End(0) )?;
        self.target.write_u16::<LittleEndian>(crc_out)
    }

    pub fn target(&self) -> &W   {&self.target}

}

pub fn read_file_filename(path: &str) -> std::io::Result<FitFile> {
    println!("Opening file: {}", path);
    let mut file = File::open(path)?;

    return read_file_read(&mut file);
}

pub fn read_file_read(source: &mut dyn Read) -> std::io::Result<FitFile> {
    let mut my_file: FitFile = FitFile::new();
    let mut context: FitFileContext = Default::default();

    let mut reader = BufReader::new(source);
    my_file.header = read_global_header(&mut context, &mut reader)?;

    // Read data, total file size is header + data + crc
    let len_to_read =  my_file.header.data_size;
    while context.data_bytes_read < len_to_read {
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
    let p = match profile::build_profile(){
        Ok(p) => {p},
        Err(e) => {return Err(std::io::Error::new(std::io::ErrorKind::Other, e));},
    };

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
    let mut out_header = (my_file.header).clone();

    let new_header_rec = FitRecord::HeaderRecord(out_header.clone());
    write_record(&mut out_context, &mut writer, &new_header_rec)?;

    let mut num_rec = 1;  // Count the header as one record.

    // Read data, total file size is header + data + crc
    let len_to_read = my_file.header.data_size;
    while context.data_bytes_read < len_to_read {
        let rec = read_record(&mut context, &mut reader);
        match rec {
            Ok(v) => {
                print_rec(&v, &p);
                match check_rec(&context, &v ) {
                    Ok(_) => { write_record(&mut out_context, &mut writer, &v) ?;},
                    Err(e) => println!("Skipping bad values in rec {}", e),
                }

            },
            Err(e) => println!("Skipping bad rec {}", e),
        }
        num_rec = num_rec + 1;
    }

    writer.flush()?;
    // Update data size, write new header.

    out_header.data_size = out_context.data_bytes_written;
    writer.seek(std::io::SeekFrom::Start(0) )?;
    write_record(&mut out_context, &mut writer, &FitRecord::HeaderRecord(out_header))?;
    writer.flush()?;  // Required.

    // compute new crc
    let crc_out = fitcrc::crc_for_file(writer.get_mut() )?;  // "inadvisable"
    writer.seek(std::io::SeekFrom::End(0) )?;
    writer.write_u16::<LittleEndian>(crc_out)?;
    println!("Write CRC: 0x{:x}", crc_out);
    println!("Info: Read {:} records from {:} bytes", num_rec, context.data_bytes_read);

    // Read directly as we don't want the crc value included in the crc computation.
    let crc = reader.read_u16::<LittleEndian>()?;
    println!("CRC: Computed 0x{:x}, Provided 0x{:x}", context.crc.digest(), crc);

    Ok(my_file)
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::fittypes::{FitFieldData};
    use std::io::{Cursor, SeekFrom};
    use crate::fitcrc::FitCrc;

    /// This sample file is settings.fit from the FitSDKRelease_20.90.00
    fn get_settings_fit() -> Vec<u8> {
        base64::decode(
            "DBBHAEQAAAAuRklUQAABAAAEAQKEAgKEAwSMAAEAAAABA9wAAeJAA\
                   kAAAQADBQQChAEBAAIBAgMBAgUBAAADhAEcvgBAAAEABAEBAosAAGQ5UA==")
            .unwrap()
    }

    fn get_activity_fit() -> Vec<u8> {
        base64::decode("DBBkAPUCAAAuRklUQAABAAAFAwSMBASGAQKEAgKEAAEAAH////8p5gcSAA8AAQRAAAEAMQIAAoQ\
        BAQJAAAEAMQEAAoQAAPBBAAEAFQX9BIYDBIYAAQABAQAEAQJBAAEAFQX9BIYDAQAAAQABAQAEAQIBKeYHEgAAAABCAA\
        EAFAb9BIYABIUBBIUFBIYCAoQGAoQCKeYHEh2FYS7L+7SXAAAAAg8zAAACKeYHEx2FYS7L+7SYAAAAAg8zAAACKeYHF\
        B2FYS7L+7SYAAAAAg8zAAACKeYHFR2FYTnL+7SCAAAAFQ8zAAACKeYHFh2FYUDL+7R5AAAAHA8zAAACKeYHFx2FYUbL\
        +7RyAAAAIw8zAAACKeYHGB2FYUrL+7RsAAAAKQ8zAAACKeYHGR2FYXfL+7QUAAAAcg8zAAACKeYHGh2FYY3L+7O0AAA\
        AuQ8zAFwCKeYHGx2FYa7L+7M8AAABEw8zAJgCKeYHHB2FYczL+7LXAAABXw8zANECKeYHHR2FYarL+7J5AAABpg8zAQ\
        YCKeYHHh2FYV/L+7KNAAAB7Q8zATMCKeYHHx2FYRLL+7JXAAACPQ8zAXABKeYHHwAABABDAAEAExT9BIYCBIYDBIUEB\
        IUFBIUGBIUHBIYIBIYJBIb+AoQLAoQMAoQNAoQOAoQVAoQWAoQAAQABAQAYAQAZAQADKeYHoynmBxIdhWEuy/u0lx2F\
        YRLL+7JXAAA1tQAANbUAAAI9AAAAAAAAAaEBcAAAAAAJAQcBQQABABUF/QSGAwSGAAEAAQEABAECASnmB6MAAAABCAk\
        BRAABABIV/QSGAgSGAwSFBASFBwSGCASGCQSG/gKECwKEDQKEDgKEDwKEFgKEFwKEGQKEGgKEAAEAAQEABQEABgEAHA\
        EABCnmB6Mp5gcSHYVhLsv7tJcAADW1AAA1tQAAAj0AAAAAAAABoQFwAAAAAAAAAAEJAQEAAEUAAQAiB/0EhgAEhgUEh\
        gEChAIBAAMBAAQBAAUp5gejAAA1tSnlz2MAAQAaAdWh").unwrap()
    }

    // DeveloperData.fit
    fn get_developer_data_fit() -> Vec<u8> {
        base64::decode("DiBoBqIAAAAuRklUvtBAAAEAAAQBAoQAAQACAoQDBIwAAA8EIykAAAalQAABAM8CARANAw\
        ECAAEBAgMFCA0VIjdZkOl5YtsAQAABAM4FAAECAQECAgECAxEHCAoHAAAAAWRvdWdobnV0c19lYXJuZWQAZG91Z2hud\
        XRzAGAAAQAUBAMBAgQBAgUEhgYChAEAAQAAjFgAAMc4uYABAI9aAAMsgI5AAgCQXAAFqTiKEAPTng=").unwrap()
    }

    #[test]
    fn test_read_settings() {
        let settings_fit = get_settings_fit();
        let file_read = read_file_read(&mut settings_fit.as_slice());
        assert!(file_read.is_ok());

        let file_data = file_read.unwrap();
        assert_eq!(68, file_data.header.data_size);
        assert_eq!(6, file_data.records.len());

        match &file_data.records[0] {
            FitRecord::DefinitionMessage(x) => {
                assert_eq!(0, x.local_message_type);
                assert_eq!(0, x.global_message_number); //file_id
                assert_eq!(4, x.field_defns.len());
            },
            _ => { assert!(false) },
        }

        match &file_data.records[3] {
            FitRecord::DataRecord(x) => {
                assert_eq!(0, x.local_message_type);
                assert_eq!(3, x.global_message_number); //user_profile
                assert_eq!(5, x.fields.len());
                assert_eq!(4, x.fields[0].field_defn_num); //weight
                match &x.fields[0].data {
                    FitFieldData::FitUint16(x) => {
                        assert_eq!(1, x.len());
                        assert_eq!(900, x[0]);   // 90.0kg, scale factor 10.
                    },
                    _ => { assert!(false) },
                }
                assert_eq!(3, x.fields[3].field_defn_num); //weight
                match &x.fields[3].data {
                    FitFieldData::FitUint8(x) => {
                        assert_eq!(1, x.len());
                        assert_eq!(190, x[0]);   // 1.9m, scale factor 100.
                    },
                    _ => { assert!(false) },
                }
            },
            _ => { assert!(false) },
        }
    }

    #[test]
    fn test_read_write() -> Result<(), std::io::Error> {
        init();
        let settings_fit = get_settings_fit();
        let mut in_cursor = Cursor::new(settings_fit.clone());

        let mut reader = FitFileReader::new(&mut in_cursor);
        let out_cursor = Cursor::new(Vec::new());
        let mut writer = FitFileWriter::new(out_cursor);
        let header = reader.read_global_header()?;
        writer.write_global_header(&header)?;

        loop {
            let field = reader.read_next()?;
            match field {
                FitRecord::HeaderRecord(_) => { panic!("BAD header record"); },
                FitRecord::DataRecord(_) |
                FitRecord::DefinitionMessage(_) => {
                    writer.write_next(&field)?;
                },
                FitRecord::EndOfFile(_) => { break; },
            }
        }
        writer.finalize()?;

        let buf = writer.target().get_ref().clone();

        assert_eq!(settings_fit.len(), buf.len());
        assert_eq!(settings_fit, buf);
        Ok(())
    }

    fn dump_file( fit_data: &Vec<u8>) -> Result<(), std::io::Error> {
        init();
        println!("Test data: {} bytes", fit_data.len());
        let pf =  profile::build_profile().unwrap();

        let mut in_cursor = Cursor::new(fit_data);
        assert!(FitCrc::check_crc(&mut in_cursor, 0, fit_data.len() as u64)?);
        in_cursor.seek(SeekFrom::Start(0))?;

        let mut reader = FitFileReader::new(&mut in_cursor);
        let header = reader.read_global_header()?;

        print_rec(&FitRecord::HeaderRecord(header), &pf);

        loop {
            let field = reader.read_next()?;
            match field {
                FitRecord::HeaderRecord(_) |
                FitRecord::DataRecord(_) |
                FitRecord::DefinitionMessage(_) => {
                    print_rec(&field, &pf);
                },
                FitRecord::EndOfFile(_) => { break; },
            }
        }
        Ok(())
    }

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_dump_settings() -> Result<(), std::io::Error> {
        init();
        dump_file(&get_settings_fit() )
    }

    #[test]
    fn test_dump_activity() -> Result<(), std::io::Error> {
        init();
        dump_file(&get_activity_fit() )
    }

    #[test]
    fn test_dump_developer_data() -> Result<(), std::io::Error> {
        init();
        dump_file(&get_developer_data_fit() )
    }
}
