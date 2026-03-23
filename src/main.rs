use std::{ffi::OsStr, fs, io, path::Path};

use image_codecs::{self, Image};

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
            // skip
            // println!("Can't compare normals");
        } else if tga.buf != png.buf {
            println!("Pixels don't match ({} vs {})", tga_filename, png_filename);
        } else {
            // skip
            // println!("Images are the same");
        }
    }

    Ok(())
}

fn main() {
    compare_textures().unwrap();
}
