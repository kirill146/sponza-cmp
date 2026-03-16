use std::fs;

use image_codecs::{self, Image};

fn main() {
    let tga = fs::read("sponza-versions/frank_meinl/textures/lion.tga").expect("Can't open file");
    let png = fs::read("sponza-versions/morgan_mcguire/textures/lion.png").expect("Can't open file");
    let tga_img = Image::new(&tga).expect("Can't decode tga");
    let png_img = Image::new(&png).expect("Can't decode png");
    if tga_img.w != png_img.w || tga_img.h != png_img.h || tga_img.channels != png_img.channels || tga_img.depth != png_img.depth {
        println!("Image parameters don't match");
    } else if tga_img.buf != png_img.buf {
        println!("Pixels don't match");
    } else {
        println!("Images are the same");
    }
}
