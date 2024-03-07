extern crate core;

mod macros;
mod utils;

use crate::utils::get_pb;
#[allow(clippy::wildcard_imports)]
use image::*;
use indicatif::ProgressIterator as _;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
// #[allow(clippy::wildcard_imports)]
// use indicatif::*;
use oklab::{oklab_to_srgb, srgb_to_oklab, Oklab, RGB};
// use rayon::prelude::*;
use std::time::Instant;

#[derive(Default, Clone, Debug)]
struct MinMaxAB {
    pub min_a: f32,
    pub max_a: f32,
    pub min_b: f32,
    pub max_b: f32,
    pub rgb_to_oklab: HashMap<MyRgb, MyOklab>,
    pub oklab_to_rgb: HashMap<MyOklab, MyRgb>,
}

#[derive(Default, Copy, Clone, Debug, Hash, Eq, PartialEq)]
struct MyRgb {
    r: u8,
    g: u8,
    b: u8,
}

impl From<RGB<u8>> for MyRgb {
    fn from(RGB { r, g, b }: RGB<u8>) -> Self {
        Self { r, g, b }
    }
}

impl Into<RGB<u8>> for MyRgb {
    fn into(self) -> RGB<u8> {
        let Self { r, g, b } = self;
        RGB { r, g, b }
    }
}

#[derive(Default, Copy, Clone, Debug, Hash, Eq, PartialEq)]
struct MyOklab {
    l: u32,
    a: u32,
    b: u32,
}

impl From<Oklab> for MyOklab {
    fn from(value: Oklab) -> Self {
        let Oklab { l, a, b } = value;

        let l = l.to_bits();
        let a = a.to_bits();
        let b = b.to_bits();

        Self { l, a, b }
    }
}

impl Into<Oklab> for MyOklab {
    fn into(self) -> Oklab {
        let Self { l, a, b } = self;

        let l = f32::from_bits(l);
        let a = f32::from_bits(a);
        let b = f32::from_bits(b);

        Oklab { l, a, b }
    }
}

fn main() {
    let reference_values = time_it! {at once | "finding out max and min values" =>
        (0..255u8).flat_map(|r|
            (0..255u8).flat_map(move |g|
                (0..255u8).map(move |b| RGB::from([r,g,b]))
            )
        )
        .progress_with(get_pb(256*256*256, "finding out max and min values"))
        .map(|it| (it, Oklab::from(it)))
        .fold(MinMaxAB::default(), |acc, (rgb, oklab)| {
            let MinMaxAB {min_a,max_a,min_b,max_b, mut rgb_to_oklab,mut oklab_to_rgb} = acc;
            let Oklab { a,b, .. } = oklab;

            let min_a = min_a.min(a);
            let max_a = max_a.max(a);
            let min_b = min_b.min(b);
            let max_b = max_b.max(b);

            rgb_to_oklab.insert(rgb.into(), oklab.into());
            oklab_to_rgb.insert(oklab.into(), rgb.into());

            MinMaxAB {min_a, max_a, min_b, max_b, rgb_to_oklab, oklab_to_rgb}
        })
    };

    // dbg!(&reference_values);

    let img = time_it! { "loading the image" =>
        image::io::Reader::open("bg.png").unwrap().decode().unwrap()
    };
    println!("dims: {:?}", img.dimensions());

    let len = time_it! { "pixels count" =>
        img.pixels().count()
    };
    println!("img pixels len {len}");

    let pixel_manipulation = |ConstL { l, a, b }| {
        let a = 1.0 - a;
        let b = 1.0 - b;

        ConstL { l, a, b }
    };

    let out = manipulate(&reference_values, &img, pixel_manipulation);
    let out = manipulate(&reference_values, &out, pixel_manipulation);

    time_it! { "saving buffer" =>
        out.save("out.png").unwrap()
    }

    let it = img
        .as_rgb8()
        .unwrap()
        .pixels()
        .progress_with(get_pb(
            (img.dimensions().0 * img.dimensions().1).into(),
            "processing",
        ))
        .map(|&Rgb([r, g, b])| RGB { r, g, b })
        .map(srgb_to_oklab)
        .collect::<Vec<_>>();

    let min_a = it
        .iter()
        .min_by(|x, y| x.a.partial_cmp(&y.a).unwrap())
        .unwrap()
        .a;
    let min_b = it
        .iter()
        .min_by(|x, y| x.b.partial_cmp(&y.b).unwrap())
        .unwrap()
        .b;
    let max_a = it
        .iter()
        .max_by(|x, y| x.a.partial_cmp(&y.a).unwrap())
        .unwrap()
        .a;
    let max_b = it
        .iter()
        .max_by(|x, y| x.b.partial_cmp(&y.b).unwrap())
        .unwrap()
        .b;

    dbg!((min_a, max_a));
    dbg!((min_b, max_b));
}

#[derive(Debug, Copy, Clone)]
struct ConstL {
    l: f32,
    a: f32,
    b: f32,
}

fn manipulate(
    reference_values: &MinMaxAB,
    img: &DynamicImage,
    pixel_manipulation: fn(ConstL) -> ConstL,
) -> DynamicImage {
    println!("processing single image to invert the a and b from OKLAB's l, a, and b");
    let process_start = Instant::now();

    let dims = img.dimensions();
    let img_pixels_len = (dims.0 * dims.1).into();

    // setup
    let out_pixels = img
        .as_rgb8()
        .unwrap()
        .pixels()
        .progress_with(get_pb(img_pixels_len, "processing"));

    // pre manip
    let out_pixels = out_pixels
        .map(|&Rgb([r, g, b])| RGB { r, g, b })
        .map(Into::into)
        .map(|it|
            reference_values.rgb_to_oklab.get(&it).unwrap().to_owned()
        )
        .map(|it| {
            let Oklab { l, a, b } = it.into();

            // let l = l;
            let a = remap!(value: a, from: reference_values.min_a, reference_values.max_a, to: 0.0, 1.0);
            let b = remap!(value: b, from: reference_values.min_b, reference_values.max_b, to: 0.0, 1.0);

            ConstL { l, a, b }
        });

    // manip
    let out_pixels = out_pixels.map(pixel_manipulation);

    // post manip
    let out_pixels = out_pixels
        .map(|ConstL { l, a, b }| {
            let a = remap!(value: a, from: 0.0, 1.0, to: reference_values.min_a, reference_values.max_a);
            let b = remap!(value: b, from: 0.0, 1.0, to: reference_values.min_b, reference_values.max_b);

            let it = Oklab { l, a, b };

            MyOklab::from(it)
        })
        .map(|it|
            reference_values.oklab_to_rgb.get(&it).unwrap().to_owned()
        )
        .map(Into::into)
        .flat_map(|RGB { r, g, b }| [r, g, b]);

    // collecting results
    let out_pixels = out_pixels.collect();

    println!("process took {:?}", process_start.elapsed());

    let out = RgbImage::from_vec(dims.0, dims.1, out_pixels).unwrap();
    out.into()
}
