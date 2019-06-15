
use std::collections::HashMap;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default)]
pub struct ProfileField {
    pub field_defn_num:u8,
    pub field_name: String,
    pub scale: Option<f64>,
    pub offset: Option<f64>,
    pub units: Option<String>,
    pub is_array: Option<bool>,
    pub field_type: String,
}

#[derive(Deserialize)]
#[derive(Clone, Debug, Default)]
pub struct ProfileType {
    pub base_type: String,
    pub type_name: String,
    pub values: HashMap<String,u32>,
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
    type_map: HashMap<String, ProfileType>,
}

pub fn build_profile() -> Result<ProfileData, String> {

    let json_messages = include_bytes!("messages.json");
    let json_types = include_bytes!("types.json");

    // Parse the string of data into serde_json::Value.
    let vec_from_json: Vec<ProfileMessage> = match serde_json::from_slice(json_messages){
        Ok(v) => {v},
        Err(e) => {return Err(e.to_string());},
    };

    let mut message_map: HashMap<u16, ProfileMessage> = HashMap::new();

    for message in &vec_from_json {
        message_map.insert(message.mesg_num, message.clone());
    }

    let type_vec: Vec<ProfileType> = match serde_json::from_slice(json_types){
        Ok(x) => {x},
        Err(e) => {return Err(e.to_string());},
    };
    let mut type_map: HashMap<String, ProfileType> = HashMap::new();

    for item in &type_vec {
        type_map.insert(item.type_name.clone(), item.clone());
    }

    Ok( ProfileData { message_map, type_map } )
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

    /// Find the value corresponding to the given type name.
    pub fn value_name(&self, type_name: &str, value: u32) -> Option<String>
    {
        let a_type = self.type_map.get(type_name)?;
        for (k,v) in &a_type.values {
            if v == &value {
                return Some(k.clone());
            }
        }
        return None;
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
        assert_eq!(p.value_name("file", 14), Some("blood_pressure".to_string()));
    }
}