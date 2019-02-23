
use serde_json::{Result, Value};
use std::collections::HashMap;

use serde::{Deserialize};

#[derive(Deserialize)]
#[derive(Clone, Debug, Default)]
struct ProfileField {
    field_defn_num:u8,
    field_name: String,
    scale: Option<f64>,
    offset: Option<f64>,
    units: Option<String>,
}

#[derive(Deserialize)]
#[derive(Clone, Debug, Default)]
struct ProfileMessage {
    mesg_num: u16,
    message_name: String,
    fields: Vec<ProfileField>,
}

pub struct ProfileData {
    message_map: HashMap<u16, ProfileMessage>,
}

pub fn build_profile() -> Result<ProfileData> {

    let json_data = include_bytes!("profile.json");

    // Parse the string of data into serde_json::Value.
    let vec_from_json: Vec<ProfileMessage> = serde_json::from_slice(json_data)?;

    let mut hmap: HashMap<u16, ProfileMessage> = HashMap::new();

    for message in &vec_from_json {
        hmap.insert(message.mesg_num, message.clone());
    }
    Ok( ProfileData { message_map: hmap } )
}

trait QueryProfile {
    fn get_message(&self, message_num: u16) -> Option<&ProfileMessage> ;
}

impl QueryProfile for ProfileData {
    fn get_message(&self, message_num: u16) -> Option<&ProfileMessage> {
        self.message_map.get(&message_num)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_build() {
        assert!(build_profile().is_ok(), "json could not be parsed")
    }

    #[test]
    fn test_message_lookup() {
        let p = build_profile().unwrap();

        let a_message = p.get_message(0).unwrap();
        assert_eq!(a_message.mesg_num, 0);
        assert_eq!(a_message.message_name, "file_id");
    }
}