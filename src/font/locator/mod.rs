#![allow(dead_code)]
use crate::input::FontAttributes;
use failure::Fallible;

pub mod allsorts;

#[derive(Clone, PartialEq, Eq)]
pub enum FontDataHandle {
    Memory { name: String, data: Vec<u8>, index: u32 },
}

pub trait FontLocator {
    fn load_fonts(&self, font_attributes: &[FontAttributes]) -> Fallible<Vec<FontDataHandle>>;
}
