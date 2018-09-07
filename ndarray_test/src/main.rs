//-------------------------------------------------------------------------
//  primeras pruebas para hacer un toolbox de robotica
//-------------------------------------------------------------------------
extern crate ndarray;
extern crate num;

use ndarray::prelude::*;

/// Brief.
///
/// Compute the rotation around the `x` axis(in cartesian coordinates)
///
/// Description
///
/// * `angle` - Angle of rotation in degrees
fn rotx(angle: f64) -> Array2<f64> {
    let c = angle.to_radians().cos();
    let s = angle.to_radians().sin();
    arr2(&[[1.0, 0.0, 0.0],
           [0.0,   c,  -s],
           [0.0,   s,   c]])
}

/// Brief.
///
/// Compute the rotation around the `y` axis(in cartesian coordinates)
///
/// Description
///
/// * `angle` - Angle of rotation in degrees
fn roty(angle: f64) -> Array2<f64> {
    let c = angle.to_radians().cos();
    let s = angle.to_radians().sin();
    arr2(&[[c,   0.0,   s],
           [0.0, 1.0, 0.0],
           [-s,  0.0,   c]])
}

/// Brief.
///
/// Compute the rotation around the `z` axis(in cartesian coordinates)
///
/// Description
///
/// * `angle` - Angle of rotation in degrees
fn rotz(angle: f64) -> Array2<f64> {
    let c = angle.to_radians().cos();
    let s = angle.to_radians().sin();
    arr2(&[[c,   -s,   0.0],
           [s,    c,   0.0],
           [0.0,  0.0, 1.0]])
}

fn rot2trans(r: &Array2<f64>) -> Array2<f64> {
    let mut R = Array2::<f64>::zeros((4,4));
    for row in 0..3 {
        for column in 0..3 {
            R[[row, column]] = r[[row, column]];
        }
    }
    R[[3, 3]] = 1.0;
    return R;
}

fn trotx(angle: f64) -> Array2<f64> {
    rot2trans(&rotx(angle.to_radians()))
}

fn troty(angle: f64) -> Array2<f64> {
    rot2trans(&roty(angle.to_radians()))
}

fn trotz(angle: f64) -> Array2<f64> {
    rot2trans(&rotz(angle.to_radians()))
}

fn euler2rot(angle_phi: f64, angle_theta: f64, angle_psi: f64) -> Array2<f64> {
    rotz(angle_phi) * roty(angle_theta) * rotz(angle_psi)
}

fn main() {
    let r = rotx(90.0);
    let R = rot2trans(&r);
    let r_z = trotx(90.0);
    println!("r: {:}", r);
}

//-------------------------------------------------------------------------
//                        tests
//-------------------------------------------------------------------------
#[test]
fn test_rotations() {
    assert_eq!(1, 1);
}
