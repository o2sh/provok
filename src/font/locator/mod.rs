#![allow(dead_code)]
use crate::input::FontAttributes;
use failure::Fallible;
use std::path::PathBuf;

pub mod parser;

#[derive(Clone, PartialEq, Eq)]
pub enum FontDataHandle {
    OnDisk {
        path: PathBuf,
        index: u32,
    },
    #[allow(dead_code)]
    Memory {
        name: String,
        data: Vec<u8>,
        index: u32,
    },
}

pub trait FontLocator {
    fn load_fonts(&self, font_attributes: &[FontAttributes]) -> Fallible<Vec<FontDataHandle>>;
}
