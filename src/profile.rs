
use serde_json::{Result};
use std::collections::HashMap;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default)]
pub struct ProfileField {
    pub field_defn_num:u8,
    pub field_name: String,
    pub scale: Option<f64>,
    pub offset: Option<f64>,
    pub units: Option<String>,
}

#[derive(Deserialize)]
#[derive(Clone, Debug, Default)]
pub struct ProfileMessage {
    pub mesg_num: u16,
    pub message_name: String,
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

impl ProfileMessage {
    pub fn find_field(&self, field_defn_num: u8) -> Option<&ProfileField> {
        self.fields.iter().find( | &x| x.field_defn_num == field_defn_num)
    }
}

impl ProfileData {
    pub fn get_message(&self, message_num: u16) -> Option<&ProfileMessage> {
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