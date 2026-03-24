use std::{ffi::OsStr, fs, io, path::Path};

use image_codecs::{self, Image};

fn eq_masks(img_a: &Image, img_b: &Image) -> bool {
    assert_eq!(img_a.w, img_b.w);
    assert_eq!(img_a.h, img_b.h);
    assert_eq!(img_a.depth, 8);
    assert_eq!(img_b.depth, 8);

    for y in 0..img_a.h as usize {
        for x in 0..img_a.w as usize {
            let mask_val = img_a.buf[(y * img_a.w as usize + x) * img_a.channels as usize + 0] as i32;

            for c in 1..img_a.channels as usize {
                let val = img_a.buf[(y * img_a.w as usize + x) * img_a.channels as usize + c] as i32;
                if val != mask_val {
                    return false;
                }
            }

            for c in 0..img_b.channels as usize {
                let val = img_b.buf[(y * img_b.w as usize + x) * img_b.channels as usize + c] as i32;
                if (val - mask_val).abs() > 2 {
                    println!("x: {x} y: {y} c: {c} mask_val: {mask_val} val: {val}");
                    return false;
                }
            }
        }
    }

    return true;
}

fn is_monochrome(img: &Image) -> bool {
    for y in 0..img.h as usize {
        for x in 0..img.w as usize {
            let val = img.buf[(y * img.w as usize + x) * img.channels as usize + 0];
            for c in 1..img.channels as usize {
                if img.buf[(y * img.w as usize + x) * img.channels as usize + c] != val {
                    return false;
                }
            }
        }
    }

    return true;
}

fn eq_normals(normal_map: &Image, bump_map: &Image) -> bool {
    assert_eq!(normal_map.w, bump_map.w);
    assert_eq!(normal_map.h, bump_map.h);

    assert_eq!(normal_map.depth, 8);
    assert_eq!(bump_map.depth, 8);

    assert_eq!(normal_map.channels, 3);

    assert!(is_monochrome(bump_map));

    // reference implementations:
    // https://web.archive.org/web/20161222201234/http://cs.williams.edu/~morgan/code/C++/normal2bump.cpp
    // https://github.com/morgan3d/misc/blob/master/normal2bump/normal2bump.cpp

    let w = normal_map.w as usize;
    let h = normal_map.h as usize;

    // compute laplacian
    let mut laplacian = vec![0f32; w * h];
    for y in 0..h as usize {
        for x in 0..w as usize {
            let ddx = normal_map.buf[(y * w + (x + 1) % w) as usize * 3 + 0] as f32 * (1f32 / 255f32) - normal_map.buf[(y * w + (x + w - 1) % w) as usize * 3 + 0] as f32 * (1f32 / 255f32);
            let ddy = normal_map.buf[((y + 1) % h * w + x) as usize * 3 + 1] as f32 * (1f32 / 255f32) - normal_map.buf[((y + h - 1) % h * w + x) as usize * 3 + 1] as f32 * (1f32 / 255f32);
            laplacian[y * w + x] = (ddx + ddy) / 2f32;
        }
    }

    // ping-pong
    let mut src = vec![0f32; w * h];
    let mut dst = vec![0.5f32; w * h];
    const N: u32 = 100;
    for _ in 0..N {
        std::mem::swap(&mut src, &mut dst);
        for y in 0..h {
            for x in 0..w {
                dst[y * w + x] = (src[y * w + (x + w - 1) % w] + src[(y + h - 1) % h * w + x]
                                + src[y * w + (x + 1) % w] + src[(y + 1) % h * w + x]
                                + laplacian[y * w + x]) * 0.25f32;
            }
        }
    }

    // normalize
    let lo = *dst.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let hi = *dst.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    for col in &mut dst {
        *col = (*col - lo) / (hi - lo);
    }

    // convert to u8
    let mut bump = vec![0u8; w * h];
    for i in 0..dst.len() {
        bump[i] = (dst[i] * 255f32 + 0.5f32) as u8;
    }

    // compare with the reference
    const THRESHOLD: i32 = 1;
    for y in 0..h {
        for x in 0..w {
            let val = bump_map.buf[(y * w + x) * bump_map.channels as usize + 0] as i32;
            let diff = (bump[y * w + x] as i32 - val).abs();
            if diff > THRESHOLD {
                println!("x: {x} y: {y} diff: {diff}");
                return false;
            }
        }
    }

    true
}

fn compare_textures() -> io::Result<()> {
    let path_mcguire = Path::new("sponza-versions/morgan_mcguire/textures");
    let path_meinl =   Path::new("sponza-versions/frank_meinl/textures");
    assert!(path_mcguire.is_dir());
    assert!(path_meinl.is_dir());

    for entry in fs::read_dir(path_mcguire)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.extension() != Some(OsStr::new("png")) {
            continue;
        }

        // convert to the matching name in Crytek's folder
        let png_filename = path.file_name().unwrap().to_str().unwrap();
        let tga_filename = png_filename
            .replace("_bump", "_ddn")
            .replace(".png", ".tga");
        let tga_file = path_meinl.join(&tga_filename);

        if !tga_file.is_file() {
            println!("{} doesn't exist", tga_filename);
            continue;
        }

        let is_ddn = tga_filename.ends_with("_ddn.tga");

        let png_raw = fs::read(&path).unwrap();
        let tga_raw = fs::read(&tga_file).unwrap();

        // println!("Decoding {}", png_filename);
        let png = Image::new(&png_raw).unwrap();
        // println!("Decoding {}", tga_filename);
        let tga = Image::new(&tga_raw).unwrap();

        if tga.w != png.w || tga.h != png.h {
            println!("Dimensions don't match: {}x{} vs {}x{} ({} vs {})", tga.w, png.w, tga.h, png.h, tga_filename, png_filename);
        } else if tga.depth != png.depth {
            println!("Depths don't match: {} vs {} ({} vs {})", tga.depth, png.depth, tga_filename, png_filename);
        } else if is_ddn {
            if !eq_normals(&tga, &png) {
                println!("Normals don't match ({} vs {})", tga_filename, png_filename);
            } else {
                // println!("Normals comparison: ok ({} vs {})", tga_filename, png_filename);
            }
        } else if tga.channels != png.channels {
            if !eq_masks(&tga, &png) {
                println!("Masks aren't the same: ({}c {} vs {}c {})", tga.channels, tga_filename, png.channels, png_filename);
            } else {
                // println!("Mask comparison: ok ({} vs {})", tga_filename, png_filename);
            }
        } else if tga.channels != png.channels {
            println!("Number of channels don't match: {} vs {} ({} vs {})", tga.channels, png.channels, tga_filename, png_filename);
        } else if tga.buf != png.buf {
            if tga.buf.len() != png.buf.len() {
                println!("tga_len != png_len: {} != {}", tga.buf.len(), png.buf.len());
            } else {
                println!("Pixels don't match ({} vs {})", tga_filename, png_filename);
            }
        } else {
            // println!("Images are the same"); // skip
        }
    }

    Ok(())
}

fn main() {
    compare_textures().unwrap();
}
