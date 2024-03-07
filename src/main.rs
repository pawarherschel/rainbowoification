extern crate core;

use std::fs;

#[allow(clippy::wildcard_imports)]
use image::*;
use indicatif::{ParallelProgressIterator, ProgressIterator as _};
use oklab::{Oklab, oklab_to_srgb, RGB, srgb_to_oklab};
use rayon::prelude::*;

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

fn get_ref_vals() -> MinMaxAB {
    (0..=255u8)
        .flat_map(|r| (0..=255u8).flat_map(move |g| (0..=255u8).map(move |b| RGB::from([r, g, b]))))
        .progress_with(get_pb(256 * 256 * 256, "finding out max and min values"))
        .map(|it| (it, Oklab::from(it)))
        .fold(MinMaxAB::default(), |acc, (_rgb, oklab)| {
            let MinMaxAB {
                min_a,
                max_a,
                min_b,
                max_b,
            } = acc;
            let Oklab { a, b, .. } = oklab;

            let min_a = min_a.min(a);
            let max_a = max_a.max(a);
            let min_b = min_b.min(b);
            let max_b = max_b.max(b);

            MinMaxAB {
                min_a,
                max_a,
                min_b,
                max_b,
            }
        })
}

fn main() {
    let reference_values = time_it! {at once | "finding out max and min values" =>
        get_ref_vals()
    };

    dbg!(&reference_values);

    let frames = 840u32;

    let img = time_it! { "loading the image" =>
        image::io::Reader::open("bg.png").unwrap().decode().unwrap()
    };
    println!("dims: {:?}", img.dimensions());

    let len = time_it! {at once | "pixels count" =>
        img.pixels().count()
    };
    println!("img pixels len {len}");

    let pixel_manipulation_raw = |ConstL { l, a, b }, perc: f32| {
        // let perc = perc * 100.0;
        // let a_mul = ((perc as u32 / 10) as f32) / 100.0;
        // let b_mul = ((perc as u32 % 10) as f32) / 100.0;

        let a_fac = perc * 100.0;
        let b_fac = perc;

        let a = (a + a_fac) % 1.0;
        let b = (b + b_fac) % 1.0;

        ConstL { l, a, b }
    };

    let _ = fs::create_dir("out");

    (0..frames)
        .into_par_iter()
        .progress_with(get_pb(u64::from(frames), "processing..."))
        .for_each(|frame_no| {
            #[allow(clippy::cast_possible_truncation)]
                let perc = remap! {
                value: f64::from(frame_no + 1),
                from: 0f64, f64::from(frames),
                to: 0.0, 1.0
            } as f32;

            // println!("{frame_no}: {{ perc: {perc} }}");

            let pixel_manipulation = move |it| pixel_manipulation_raw(it, perc);
            let out = manipulate(&reference_values, &img, pixel_manipulation);

            out.save(format!("out/frame_{frame_no:03}.png")).unwrap();
        });
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
    pixel_manipulation: impl Fn(ConstL) -> ConstL,
) -> DynamicImage {
    // let process_start = Instant::now();

    let dims = img.dimensions();
    // let img_pixels_len = (dims.0 * dims.1).into();

    // setup
    let out_pixels = img.as_rgb8().unwrap().pixels();
    // .progress_with(get_pb(img_pixels_len, "processing"));

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

    // println!("process took {:?}", process_start.elapsed());

    let out = RgbImage::from_vec(dims.0, dims.1, out_pixels).unwrap();
    out.into()
}
