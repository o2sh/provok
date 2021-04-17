#![allow(dead_code)]
pub mod allsorts;

#[derive(Clone, PartialEq, Eq)]
pub enum FontDataHandle {
    Memory { name: String, data: Vec<u8>, index: u32 },
}
