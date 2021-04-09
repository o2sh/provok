#![allow(dead_code)]
use crate::input::FontAttributes;
use failure::Fallible;
use std::path::PathBuf;

pub mod parser;

#[derive(Clone)]
pub enum FontDataHandle {
    OnDisk { path: PathBuf, index: u32 },
    Memory { data: Vec<u8>, index: u32 },
}

pub trait FontLocator {
    fn load_fonts(&self, font_attributes: &[FontAttributes]) -> Fallible<Vec<FontDataHandle>>;
}
