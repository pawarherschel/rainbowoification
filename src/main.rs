extern crate core;

// use rayon::prelude::*;
use std::time::Instant;

#[allow(clippy::wildcard_imports)]
use image::*;
use indicatif::ProgressIterator as _;
// #[allow(clippy::wildcard_imports)]
// use indicatif::*;
use oklab::{oklab_to_srgb, srgb_to_oklab, Oklab, RGB};

use crate::utils::get_pb;

mod macros;
mod utils;

#[derive(Default, Copy, Clone, Debug)]
struct MinMaxAB {
    pub min_a: f32,
    pub max_a: f32,
    pub min_b: f32,
    pub max_b: f32,
}

fn main() {
    let reference_values = time_it! {at once | "finding out max and min values" =>
        (0..=255u8).flat_map(|r|
            (0..=255u8).flat_map(move |g|
                (0..=255u8).map(move |b| RGB::from([r,g,b]))
            )
        )
        .progress_with(get_pb(256*256*256, "finding out max and min values"))
        .map(|it| (it, Oklab::from(it)))
        .fold(MinMaxAB::default(), |acc, (_rgb, oklab)| {
            let MinMaxAB {min_a,max_a,min_b,max_b} = acc;
            let Oklab { a,b, .. } = oklab;

            let min_a = min_a.min(a);
            let max_a = max_a.max(a);
            let min_b = min_b.min(b);
            let max_b = max_b.max(b);

            MinMaxAB {min_a, max_a, min_b, max_b}
        })
    };

    dbg!(&reference_values);

    let img = time_it! { "loading the image" =>
        image::io::Reader::open("bg.png").unwrap().decode().unwrap()
    };
    println!("dims: {:?}", img.dimensions());

    let len = time_it! {at once | "pixels count" =>
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
        .map(srgb_to_oklab)
        .map(|Oklab { l, a, b }| {
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
            // let l = l;
            let a = remap!(value: a, from: 0.0, 1.0, to: reference_values.min_a, reference_values.max_a);
            let b = remap!(value: b, from: 0.0, 1.0, to: reference_values.min_b, reference_values.max_b);

            Oklab { l, a, b }
        })
        .map(oklab_to_srgb)
        .flat_map(|RGB { r, g, b }| [r, g, b]);

    // collecting results
    let out_pixels = out_pixels.collect();

    println!("process took {:?}", process_start.elapsed());

    let out = RgbImage::from_vec(dims.0, dims.1, out_pixels).unwrap();
    out.into()
}
