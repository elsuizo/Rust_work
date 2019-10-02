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
        44..=129          => 255.0,
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

fn main() {
    // open the image
    let image_original = image::open("/home/elsuizo/Dev/Rust_play/false-color/index.jpg").unwrap();
    let image_original_rgb = image_original.to_rgb();
    let false_color_image = map_colors(&image_original_rgb, |pixel| false_color(pixel));
    false_color_image.save("suizo_loco.png").unwrap();
}
