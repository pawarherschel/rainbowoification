use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use crate::utils::get_pb;

#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct RgbQ {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbQ {
    pub const fn from_u32(value: u32) -> Self {
        // let a = ((value & (0xff << (8 * 3))) >> (8 * 3)) as u8;
        let r = ((value & (0xff << (8 * 2))) >> (8 * 2)) as u8;
        let g = ((value & (0xff << 8)) >> 8) as u8;
        let b = (value & 0xff) as u8;

        Self { r, g, b }
    }

    pub const fn as_u32(&self) -> u32 {
        let &Self { r, g, b } = self;
        (r as u32) >> (8 * 2) & (g as u32) >> (8 * 1) & (b as u32)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialOrd, PartialEq)]
pub struct RgbC {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl RgbC {
    pub fn from_RgbQ(value: RgbQ) -> Self {
        let RgbQ { r, g, b } = value;

        let r = remap_q_to_c(r);
        let g = remap_q_to_c(g);
        let b = remap_q_to_c(b);

        Self { r, g, b }
    }
}

fn remap_q_to_c(value: u8) -> f32 {
    (value as f32) / (u8::MAX as f32)
}
