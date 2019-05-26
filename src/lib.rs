// In order to use the Serialize and Deserialize macros in the model,
// we need to declare in the main module, that we are using them.
#[macro_use]
extern crate serde_derive;

pub mod fitcrc;
pub mod fitread;
pub mod fitwrite;
pub mod fittypes;
pub mod fitheader;
pub mod fitdefnmesg;
pub mod fitdatamesg;
pub mod fitfile;
pub mod fitcheck;
pub mod fitrecord;
pub mod fitfield;

pub mod profile;


