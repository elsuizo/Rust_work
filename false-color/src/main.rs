extern crate image;
extern crate imageproc;
use imageproc::map::map_colors;
use core::u8::MAX;

fn false_color(pixel: image::Rgb<u8>) -> image::Rgb<u8> {
    let m = 255.0 / 43.0;

    let r = match pixel[0] {
        0..=43            => 255.0,
        value @ 44..=86   => -m * (value as f32 - 86.0),
        87..=172          => 0.0,
        value @ 173..=215 => m * (value as f32 - 173.0),
        216..=MAX         => 255.0,
    };
    let g = match pixel[1] {
        value @ 0..=43    => m * value as f32,
        44..=128          => 255.0,
        value @ 129..=172 => -m * (value as f32 - 172.0),
        173..=MAX         => 0.0,
    };
    let b = match pixel[2] {
        0..=86            => 0.0,
        value @ 87..=129  => m * (value as f32 - 87.0),
        130..=213         => 255.0,
        value @ 214..=MAX => -m * (value as f32 - 255.0),
    };

    image::Rgb([r as u8, g as u8, b as u8])
}

fn chromatics_coordinates(pixel: image::Rgb<u8>) -> image::Rgb<u8> {
    let r = pixel[0] as f64;
    let g = pixel[1] as f64;
    let b = pixel[2] as f64;
    // let s = pixel[0].wrapping_add(pixel[1]).wrapping_add(pixel[2]);
    let s = r + g + b;
    if s != 0.0 {
        let r_new = (r / s) * 255.0;
        let g_new = (g / s) * 255.0;
        let b_new = (b / s) * 255.0;
        println!("r: {}", r_new as u8);
        image::Rgb([r_new as u8, g_new as u8, b_new as u8])
    } else {
        image::Rgb([pixel[0], pixel[1], pixel[2]])
    }
}

fn main() {
    // open the image
    let image_original = image::open("/home/elsuizo/Repos/Rust_work/false-color/manos3.jpeg").unwrap();
    let image_original_rgb = image_original.to_rgb();
    let false_color_image = map_colors(&image_original_rgb, |pixel| chromatics_coordinates(pixel));
    false_color_image.save("suizo_loco.png").unwrap();
}
