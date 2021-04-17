#![allow(dead_code)]
pub mod allsorts;

#[derive(Debug)]
pub struct Names {
    full_name: String,
    unique: Option<String>,
    family: Option<String>,
    sub_family: Option<String>,
    postscript_name: Option<String>,
}

#[derive(Clone, PartialEq, Eq)]
pub enum FontDataHandle {
    Memory { name: String, data: Vec<u8>, index: u32 },
}
