extern crate num;
extern crate image;

use image::ColorType;
use image::png::PNGEncoder;

use num::Complex;

use std::fs::File;
use std::str::FromStr;
use std::io::Write;

/// Try to determine if `c` is in the Mandelbrot set, using at most `limit` iterations to decide.
///
/// if `c` is not a member, return `Some(i)`, where `i` is the number of iterations it took for `c`
/// to leave the circle of radius two centered on the origin. If `c` seems to be a member (more
/// precisely, if we reached the iteration limit without being able to prove that `c` is not a
/// member) return `None`
fn escape_time(c: Complex<f64>, limit: u32) -> Option<u32> {
    let mut z = Complex{re: 0.0, im: 0.0};
    for i in 0..limit {
        z = z * z + c;
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
    }
    None
}

/// Parse the string `s` as a coordinate pair, like `"400x600"` or `"1.0,0.5"`
///
/// Specifically, `s` should have the form <left><sep><right>, where <sep> is the character given
/// by the the `separator` argument, and <left> and <right> are both strings that can be parsed by
/// `T::from_str`
/// if `s` has the proper form, return `Some<(x, y)>`. If doesn't parse correctly, return `None`
fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None            => None,
        Some(index)     => {
            match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
                (Ok(l), Ok(r)) => Some((l, r)),
                _              => None
            }
        }
    }
}

/// Function to parse a pair of coordinates separated by a comma as complex number
fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex{re, im}),
        None           => None
    }
}

/// Function given the row and the column of a pixel in the output image, return the corresponding
/// point on the complex plane.
///
/// `bound` is a pair given the width and the height of the image in pixels.
/// `pixel` is a (column, row) pair indicating a particular pixel in that image.
/// The `upper_left` and `lower_right` parameters are points on the complex plane designating the
/// area our image covers.
fn pixel_to_point(bounds: (usize, usize), pixel: (usize, usize), upper_left: Complex<f64>, lower_right: Complex<f64>) -> Complex<f64> {
    let (width, height) = (lower_right.re - upper_left.re, upper_left.im - lower_right.im);
    Complex{
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64,
        // porque hacemos una resta aca??? ---> porque pixel.1 crece cuando vamos a valores mas
        // altos pero la parte imaginaria se incrementa a medida que el valor crece
    }
}

/// Function to render a rectangle of the Mandelbrot set into a buffer of pixels.
///
/// The `bounds` argument gives the width and the height of the buffer `pixels`, which holds one
/// grayscale pixel per byte. The `upper_left` and `lower_right` arguments specify points on the
/// complex plane corresponding to the upper-left and lower-right corners of the pixel buffer
fn render(pixels: &mut [u8], bounds: (usize, usize), upper_left: Complex<f64>, lower_right: Complex<f64>) {
    assert!(pixels.len() == bounds.0 * bounds.1);

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = pixel_to_point(bounds, (column, row), upper_left, lower_right);
            pixels[row * bounds.0 + column] = match escape_time(point, 255) {
                None => 0,
                Some(count) => 255 - count as u8
            };
        }
    }
}

fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize)) -> Result<(), std::io::Error> {
    let output = File::create(filename)?;
    let encoder = PNGEncoder::new(output);
    encoder.encode(&pixels, bounds.0 as u32, bounds.1 as u32, ColorType::Gray(8))?;
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 5 {
        writeln!(std::io::stderr(), "Usage: mandelbrot FILE PIXELS UPPER_LEFT LOWER_RIGHT").unwrap();
        writeln!(std::io::stderr(), "Example: {} mandel.png  1000x750 -1.20,0.35 1,0.20", args[0]).unwrap();
        std::process::exit(1);
    }

    let bounds = parse_pair(&args[2], 'x').expect("error parsing image dimentions");
    let upper_left = parse_complex(&args[3]).expect("error parsing upper_left corner point");
    let lower_right = parse_complex(&args[4]).expect("error parsing lower right corner point");
    let mut pixels = vec![0; bounds.0 * bounds.1];
    render(&mut pixels, bounds, upper_left, lower_right);

    write_image(&args[1], &pixels, bounds).expect("error writing the png file");
}

#[test]
fn pair_test() {
    assert_eq!(parse_pair::<i32>("", ','), None);
    assert_eq!(parse_pair::<i32>("300,700", ','), Some((300, 700)));
}

// NOTE(elsuizo:2019-09-14): por como esta hecho el parse no se puede tener un espacio entre los
// coordinates
#[test]
fn complex_test() {
    assert_eq!(parse_complex("1.27,1.73"), Some(Complex{re:1.27, im:1.73}));
    assert_eq!(parse_complex("1.343, 12.333"), None); // con un espacio entre ellos falla...
}

#[test]
fn pixel_to_point_test() {
    assert_eq!(pixel_to_point((100,100), (25,75), Complex{re: -1.0, im: 1.0}, Complex{re: 1.0, im: -1.0}), Complex{re: -0.5, im: -0.5});
}
