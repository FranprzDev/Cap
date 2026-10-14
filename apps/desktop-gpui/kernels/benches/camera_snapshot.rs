use cap_gpui_kernels::frame as candidate;
use std::{hint::black_box, time::Instant};

type Downsample = fn(&[u8], usize, usize, usize, u32, u32) -> Vec<u8>;

fn main() {
    println!("workload,sample,before_ns,after_ns");
    for (name, (width, height), (target_width, target_height)) in [
        ("1080p-to-540p", (1920usize, 1080usize), (960u32, 540u32)),
        ("1080p-to-720p", (1920, 1080), (1280, 720)),
        ("1440p-to-540p", (2560, 1440), (960, 540)),
        ("4k-to-540p", (3840, 2160), (960, 540)),
        ("4k-to-720p", (3840, 2160), (1280, 720)),
    ] {
        let stride = (width * 4).next_multiple_of(64) + 64;
        let data: Vec<u8> = (0..stride * height)
            .map(|i| (i.wrapping_mul(31) % 251) as u8)
            .collect();
        let run = |f: Downsample| {
            let start = Instant::now();
            let pixels = f(
                black_box(&data),
                black_box(width),
                black_box(height),
                black_box(stride),
                black_box(target_width),
                black_box(target_height),
            );
            (start.elapsed().as_nanos(), black_box(pixels))
        };
        assert_eq!(
            run(baseline::downsample_nearest).1,
            run(candidate::downsample_nearest).1,
            "{name}"
        );
        for sample in 0..35 {
            let (old, new) = if sample % 2 == 0 {
                let old = run(baseline::downsample_nearest).0;
                (old, run(candidate::downsample_nearest).0)
            } else {
                let new = run(candidate::downsample_nearest).0;
                (run(baseline::downsample_nearest).0, new)
            };
            if sample >= 5 {
                println!("{name},{},{old},{new}", sample - 5);
            }
        }
    }
}

// Preserved from 4aaade3e094ca3082bc2fbc14f9623bdf709e5e4 so both paths use the same build and inputs.
mod baseline {
    pub fn downsample_nearest(
        data: &[u8],
        width: usize,
        height: usize,
        stride: usize,
        target_width: u32,
        target_height: u32,
    ) -> Vec<u8> {
        let target_width = target_width as usize;
        let target_height = target_height as usize;
        let mut pixels = Vec::with_capacity(target_width * target_height * 4);
        for y in 0..target_height {
            let row = &data[y * height / target_height * stride..];
            for x in 0..target_width {
                let offset = x * width / target_width * 4;
                pixels.extend_from_slice(&row[offset..offset + 4]);
            }
        }
        pixels
    }
}
