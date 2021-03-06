
extern crate base64;

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek};

#[derive(Copy, Clone, Default)]
#[derive(Debug)]
pub struct FitCrc{
    crc: u16,
}

impl FitCrc {
    pub fn new() -> FitCrc
    {
        return FitCrc { crc: 0, };
    }

    pub fn consume(&mut self, byte_array: &[u8]) {
        self.crc = fit_crc_16(self.crc, byte_array);
    }

    pub fn digest(&self) -> u16 { self.crc }

    pub fn reset(&mut self) { self.crc = 0; }

    pub fn compute_crc<T: Read + Seek>(file: &mut T, from:u64, count:u64) -> Result<u16, std::io::Error>
    {
        file.seek(std::io::SeekFrom::Start(from))?;

        let mut buff = [0; 1024];
        let mut context = FitCrc::new();
        let mut remain = count;
        while remain != 0 {
            let to_read = std::cmp::min(buff.len(), count as usize);
            let n = match file.read(&mut buff[0..to_read]) {
                Ok(x) => {x},
                Err(e) => { return Err(e);},
            };
            if n == 0 {
                break;
            }
            context.consume(&buff[0..n]);
            remain = remain - (n as u64);
        }
        return Ok(context.digest());
    }

    pub fn read_crc<T: Read + Seek>(file: &mut T, offset:u64) -> Result<u16, std::io::Error>
    {
        file.seek(std::io::SeekFrom::Start(offset))?;
        let crc = file.read_u16::<LittleEndian>()?;
        return Ok(crc);
    }

    pub fn check_crc<T: Read + Seek>(file: &mut T, from:u64, count:u64) -> Result<bool, std::io::Error>
    {
        let crc_read = Self::read_crc(file, from + count - 2)?;
        let crc_computed = Self::compute_crc(file, from, count - 2)?;
        return Ok (crc_read == crc_computed);
    }
}
pub fn compute(data: &[u8]) -> u16 {
    let mut context = FitCrc::new();
    context.consume(data);
    return context.digest();
}

pub fn crc_for_file<T: Read + Seek>(file: &mut T) -> Result< u16, std::io::Error>
{
    file.seek(std::io::SeekFrom::Start(0))?;

    let mut buff = [0; 1024];
    let mut context = FitCrc::new();
    loop {
        let n = match file.read(&mut buff) {
            Ok(x) => {x},
            Err(e) => { return Err(e);},
        };
        if n == 0 {
            break;
        }
        context.consume(&buff[0..n]);
    }
    return Ok(context.digest());
}

fn fit_crc_16(mut crc: u16, byte_array: &[u8]) -> u16 {
    let crc_table: [u16; 16] =  [
        0x0000, 0xCC01, 0xD801, 0x1400, 0xF001, 0x3C00, 0x2800, 0xE401,
        0xA001, 0x6C00, 0x7800, 0xB401, 0x5000, 0x9C01, 0x8801, 0x4400
    ];

    for byte in byte_array.iter() {
        // compute checksum of lower four bits of byte
        let mut tmp = crc_table[(crc & 0xF) as usize];
        crc = (crc >> 4) & 0x0FFF;
        crc = crc ^ tmp ^ crc_table[(byte & 0xF) as usize];

        // now compute checksum of upper four bits of byte
        tmp = crc_table[(crc & 0xF) as usize];
        crc = (crc >> 4) & 0x0FFF;
        crc = crc ^ tmp ^ crc_table[((byte >> 4) & 0xFu8) as usize];
    }
    return crc;
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_crc() -> Result< (), std::io::Error>
    {
        let ba : [u8; 0] = [];
        assert_eq!(1234_u16, fit_crc_16(1234, &ba));

        let ba2 : [u8; 8] = [0,0,1,0,0,0,0,1];
        assert_eq!(4544, compute(&ba2));

        let ba3  = b"Hello World";
        assert_eq!(29657, fit_crc_16(45612, ba3));

        let ba_zeros : [u8; 8] = [0,0,0,0,0,0,0,0];
        assert_eq!(0, compute(&ba_zeros));

        // Take a sample fit file, last 2 bytes are the 16-bit crc in LittleEndian.
        // Pop them off, compare the computed CRC.
        // The sample file is settings.fit from the FitSDKRelease_20.90.00
        let mut settings_fit = base64::decode(
            "DBBHAEQAAAAuRklUQAABAAAEAQKEAgKEAwSMAAEAAAABA9wAAeJAA\
                   kAAAQADBQQChAEBAAIBAgMBAgUBAAADhAEcvgBAAAEABAEBAosAAGQ5UA==").unwrap();

        let mut in_cursor = Cursor::new(&settings_fit);
        assert!(FitCrc::check_crc(&mut in_cursor, 0, settings_fit.len() as u64)?);

        let crc1 = settings_fit.pop().unwrap();
        let crc2 = settings_fit.pop().unwrap();
        let settings_crc = (crc2 as u16) | ((crc1 as u16) << 8);
        println!("Length: {} crc: {:x}", settings_fit.len(), settings_crc);
        assert_eq!(settings_crc, compute(&settings_fit[..]));
        Ok(())
    }
}
