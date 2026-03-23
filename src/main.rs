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

        println!("Decoding {}", png_filename);
        let png = Image::new(&png_raw).unwrap();
        println!("Decoding {}", tga_filename);
        let tga = Image::new(&tga_raw).unwrap();

        if tga.w != png.w || tga.h != png.h {
            println!("Dimensions don't match: {}x{} vs {}x{} ({} vs {})", tga.w, png.w, tga.h, png.h, tga_filename, png_filename);
        } else if tga.depth != png.depth {
            println!("Depths don't match: {} vs {} ({} vs {})", tga.depth, png.depth, tga_filename, png_filename);
        } else if is_ddn {
            // println!("Can't compare normals"); // skip
        } else if tga.channels != png.channels {
            if !eq_masks(&tga, &png) {
                println!("Masks aren't the same ({} channels vs {} channels): ({} vs {})", tga.channels, png.channels, tga_filename, png_filename);
            } else {
                println!("Mask comparison: ok");
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
