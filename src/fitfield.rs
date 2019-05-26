use std::io::{BufReader, BufWriter};
use std::fs::File;

use crate::fittypes::{FitDataType, FitFieldData, FitFileContext};
use crate::fitread::{fit_read_i8, fit_read_u8, fit_read_u16, fit_read_i16, fit_read_i32,
                     fit_read_u32, fit_read_string, fit_read_f32, fit_read_f64,
                     fit_read_i64, fit_read_u64};
use crate::fitwrite::{fit_write_u8, fit_write_u16, fit_write_i8, fit_write_i16,
                      fit_write_i32, fit_write_u32, fit_write_string, fit_write_f32,
                      fit_write_f64, fit_write_u64, fit_write_i64};

pub fn read_fit_field( context: &mut FitFileContext, reader: &mut BufReader<File>,
                   data_type: FitDataType, count: u8)
                   -> Result< FitFieldData, std::io::Error >
{
    //reader.read_u16_into::<NativeEndian>(&mut buffer[..])?;
    match data_type {
        FitDataType::FitEnum => {
            let mut v: Vec<u8> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u8(context, reader)?);
            }
            Ok(FitFieldData::FitEnum(v))
        },
        FitDataType::FitSint8 => {
            let mut v: Vec<i8> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_i8(context, reader)?);
            }
            Ok(FitFieldData::FitSint8(v))
        },
        FitDataType::FitUint8 => {
            let mut v: Vec<u8> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u8(context, reader)?);
            }
            Ok(FitFieldData::FitUint8(v))
        },
        FitDataType::FitSint16 => {
            let mut v: Vec<i16> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_i16(context, reader)?);
            }
            Ok(FitFieldData::FitSint16(v))
        },
        FitDataType::FitUint16 => {
            let mut v: Vec<u16> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u16(context, reader)?);
            }
            Ok(FitFieldData::FitUint16(v))
        },
        FitDataType::FitSint32 => {
            let mut v: Vec<i32> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_i32(context, reader)?);
            }
            Ok(FitFieldData::FitSint32(v))
        },
        FitDataType::FitUint32 => {
            let mut v: Vec<u32> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u32(context, reader)?);
            }
            Ok(FitFieldData::FitUint32(v))
        },
        FitDataType::FitString => {
            let v = fit_read_string(context, reader, &count)?;
            Ok(FitFieldData::FitString(v,count))
        },
        FitDataType::FitF32 => {
            let mut v: Vec<f32> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_f32(context, reader)?);
            }
            Ok(FitFieldData::FitF32(v))
        },
        FitDataType::FitF64 => {
            let mut v: Vec<f64> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_f64(context, reader)?);
            }
            Ok(FitFieldData::FitF64(v))
        },
        FitDataType::FitU8z => {
            let mut v: Vec<u8> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u8(context, reader)?);
            }
            Ok(FitFieldData::FitU8z(v))
        },
        FitDataType::FitU16z => {
            let mut v: Vec<u16> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u16(context, reader)?);
            }
            Ok(FitFieldData::FitU16z(v))
        },
        FitDataType::FitU32z => {
            let mut v: Vec<u32> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u32(context, reader)?);
            }
            Ok(FitFieldData::FitU32z(v))
        },
        FitDataType::FitByte => {
            let mut v: Vec<u8> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u8(context, reader)?);
            }
            Ok(FitFieldData::FitByte(v))
        },
        FitDataType::FitSInt64 => {
            let mut v: Vec<i64> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_i64(context, reader)?);
            }
            Ok(FitFieldData::FitSInt64(v))
        },
        FitDataType::FitUint64 => {
            let mut v: Vec<u64> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u64(context, reader)?);
            }
            Ok(FitFieldData::FitUint64(v))
        },
        FitDataType::FitUint64z => {
            let mut v: Vec<u64> = Vec::new();
            for _i in 0..count {
                v.push(fit_read_u64(context, reader)?);
            }
            Ok(FitFieldData::FitUint64z(v))
        },
    }
}

pub fn write_fit_field(context: &mut FitFileContext, writer: &mut BufWriter<File>, field: &FitFieldData)
                   -> Result< (), std::io::Error >
{
    match field {
        FitFieldData::FitEnum(x) |
        FitFieldData::FitUint8(x) |
        FitFieldData::FitU8z(x) |
        FitFieldData::FitByte(x)  => {
            for item in x {
                fit_write_u8(context, writer, *item)?;
            }
            Ok(())
        },
        FitFieldData::FitSint8(x)  => {
            for item in x {
                fit_write_i8(context, writer, *item)?;
            }
            Ok(())
        },
        FitFieldData::FitSint16(x)   => {
            for item in x {
                fit_write_i16(context, writer, *item)?;
            }
            Ok(())
        },
        FitFieldData::FitUint16(x)  |
        FitFieldData::FitU16z(x) => {
            for item in x {
                fit_write_u16(context, writer, *item)?;
            }
            Ok(())
        },
        FitFieldData::FitSint32(x)   => {
            for item in x {
                fit_write_i32(context, writer, *item)?;
            }
            Ok(())
        },
        FitFieldData::FitUint32(x)  |
        FitFieldData::FitU32z(x) => {
            for item in x {
                fit_write_u32(context, writer, *item)?;
            }
            Ok(())
        },
        FitFieldData::FitString(x, width) => {
            fit_write_string(context, writer, x.as_str(), width )?;
            Ok(())
        },
        FitFieldData::FitF32(x)  => {
            for item in x {
                fit_write_f32(context, writer, *item)?;
            }
            Ok(())
        },
        FitFieldData::FitF64(x)  => {
            for item in x {
                fit_write_f64(context, writer, *item)?;
            }
            Ok(())
        },

        FitFieldData::FitSInt64(x) => {
            for item in x {
                fit_write_i64(context, writer, *item)?;
            }
            Ok(())
        },
        FitFieldData::FitUint64(x) |
        FitFieldData::FitUint64z(x)  => {
            for item in x {
                fit_write_u64(context, writer, *item)?;
            }
            Ok(())
        },
    }
}

