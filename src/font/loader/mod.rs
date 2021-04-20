#![allow(dead_code)]
pub mod parser;

#[derive(Debug)]
pub struct Names {
    full_name: String,
    unique: Option<String>,
    family: Option<String>,
    sub_family: Option<String>,
    postscript_name: Option<String>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct FontDataHandle {
    pub name: String,
    pub data: Vec<u8>,
    pub index: u32,
}
