#[derive(Copy, Clone)]
struct RgbQ {
    r: u8,
    g: u8,
    b: u8,
}

impl RgbQ {
    const fn from_u32(value: u32) -> Self {
        // let a = ((value & (0xff << (8 * 3))) >> (8 * 3)) as u8;
        let r = ((value & (0xff << (8 * 2))) >> (8 * 2)) as u8;
        let g = ((value & (0xff << 8)) >> 8) as u8;
        let b = (value & 0xff) as u8;

        Self { r, g, b }
    }
}

const MAX: u32 = 0x00_ff_ff_ffu32;
#[allow(long_running_const_eval)]
const ALL_RGB_QUANTIZED: [RgbQ; MAX as usize + 1] = {
    let mut output = [RgbQ { r: 0, g: 0, b: 0 }; MAX as usize + 1];

    let mut i = 0;
    while i <= MAX {
        let idx = i as usize;
        output[idx] = RgbQ::from_u32(i);

        i += 1;
    }

    output
};

#[derive(Copy, Clone)]
struct RgbC {
    r: f32,
    g: f32,
    b: f32,
}

impl RgbC {
    const fn from_RgbQ(value: RgbQ) -> Self {
        let RgbQ { r, g, b } = value;

        let r = remap_q_to_c(r);
        let g = remap_q_to_c(g);
        let b = remap_q_to_c(b);

        Self { r, g, b }
    }
}

const fn remap_q_to_c(value: u8) -> f32 {
    (value as f32) / (u8::MAX as f32)
}

const ALL_RGB_CONTINOUS: [RgbC; MAX as usize + 1] = {
    let mut output = [RgbC {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    }; MAX as usize + 1];

    let mut i = 0;
    while i <= MAX {
        output[i as usize] = RgbC::from_RgbQ(ALL_RGB_QUANTIZED[i as usize]);
    }

    output
};
