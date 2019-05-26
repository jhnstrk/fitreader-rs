
// std imports

use chrono::{Utc};
use chrono::offset::TimeZone;


use crate::fittypes::{ FitFileContext,
                       FitFieldData,
                       FitRecord, FitDataMessage};

fn get_timestamp(data_message: &FitDataMessage) -> Option< u32 >
{
    match data_message.timestamp {
        None => {
            for f in &data_message.fields {
                if f.field_defn_num == 253 {
                    match &f.data {
                        FitFieldData::FitUint32(x) => {
                            if !x.is_empty() {
                                return Some(x[0])
                            }},
                        _ => {},
                    }
                }
            }
            None
        },
        Some(x) => {Some(x)},
    }
}

fn clamp_timestamp(v: i64) -> u32
{
    if v < 0 {
        0u32
    } else if v >= std::u32::MAX as i64 {
        std::u32::MAX - 1  // MAX is reserved for a bad value.
    } else {
        v as u32
    }
}

pub fn check_rec(context: &FitFileContext, rec: &FitRecord)
             -> Result< (), std::io::Error>
{
    let now = Utc::now();
    let base_datetime = Utc.ymd(1989, 12, 31).and_hms(0, 0, 0);
    let earliest_datetime = Utc.ymd(2018, 1, 1).and_hms(0, 0, 0);
    let latest_datetime = now.checked_add_signed(chrono::Duration::weeks(1) ).unwrap();

    //now.checked_sub_signed(Duration::years(2) );
    let offset_min = clamp_timestamp( earliest_datetime.timestamp() - base_datetime.timestamp());  // in seconds
    let offset_max = clamp_timestamp( latest_datetime.timestamp() - base_datetime.timestamp());  // in seconds

    match rec {
        FitRecord::HeaderRecord(_) => {},
        FitRecord::DefinitionMessage(_) => {},
        FitRecord::EndOfFile(_) => {},
        FitRecord::DataRecord(data_message) => {
            let timestamp_opt = get_timestamp(&data_message);
            match timestamp_opt {
                None => {},
                Some(x) => {
                    // Seconds since UTC 00:00 Dec 31 1989
                    let utc_dt = base_datetime + chrono::Duration::seconds(x as i64);
                    if x < offset_min || x > offset_max {
                        let errstr = format!("Timestamp error: Out of permitted range {}", utc_dt.to_rfc3339());
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, errstr));
                    } else if x < context.timestamp {
                        let errstr = format!("Timestamp error: Timestamp is before previous one {}", utc_dt.to_rfc3339());
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, errstr));
                    } else {
                        println!("Timestamp: {}", utc_dt.to_rfc3339());
                    }
                },
            }
        },
    };
    Ok(())
}

