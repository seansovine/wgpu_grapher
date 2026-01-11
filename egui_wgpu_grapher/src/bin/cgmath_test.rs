// Testing our use of cgmath operations.

use cgmath::{Point3, SquareMatrix, Vector3, Vector4};

type Point = Point3<f32>;
type Vector = Vector3<f32>;
type Vec4 = Vector4<f32>;

fn main() {
    ///////////////////////////////
    // Test look-at transformation.

    let eye = Point::from([-1.0, 1.0, 1.0]);
    let origin = Point::from([0.0, 0.0, 0.0]);
    let up = Vector::from([0.0, 1.0, 0.0]);

    let look_at = cgmath::Matrix4::look_at_rh(eye, origin, up);
    dbg!(&look_at);

    let z_reflect = cgmath::Matrix4::from_diagonal([1.0, 1.0, -1.0, 1.0].into());
    let look_at = z_reflect * look_at;
    dbg!(&look_at);

    let pos_vec = Vec4::from([-1.0, 1.0, 1.0, 2.0]);
    let mut result = look_at * pos_vec;
    result /= result.w;
    dbg!(&result);
    // As expected, the distance in eye space is half the
    // distance from eye to origin in the eye space z-direction.

    let pos_vec = Vec4::from([0.0, 0.0, 0.0, 1.0]);
    let result = look_at * pos_vec;
    dbg!(&result);

    //////////////////////////////////
    // Test projection transformation.

    // We construct our own projection; compare with ortho() result.
    let scale = 1.0 / (look_at[3][2] + 6.0);
    let projection = cgmath::Matrix4::from_diagonal([1.0 / 6.0, 2.0 / 6.0, scale, 1.0].into());
    let translation = cgmath::Matrix4::from_translation([0.0, 0.0, -1.0].into());
    let projection = translation * projection;
    dbg!(&projection);

    let pos_vec = Vec4::from([0.0, 0.0, 6.0, 1.0]);
    let result = projection * look_at * pos_vec;
    dbg!(&result);
}
