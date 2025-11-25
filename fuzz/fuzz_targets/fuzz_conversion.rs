#![no_main]
use libfuzzer_sys::fuzz_target;
use ply2splat::{PlyGaussian, SplatPoint};
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct Input {
    x: f32, y: f32, z: f32,
    f_dc_0: f32, f_dc_1: f32, f_dc_2: f32,
    opacity: f32,
    scale_0: f32, scale_1: f32, scale_2: f32,
    rot_0: f32, rot_1: f32, rot_2: f32, rot_3: f32,
}

fuzz_target!(|data: Input| {
    let p = PlyGaussian {
        x: data.x, y: data.y, z: data.z,
        f_dc_0: data.f_dc_0, f_dc_1: data.f_dc_1, f_dc_2: data.f_dc_2,
        opacity: data.opacity,
        scale_0: data.scale_0, scale_1: data.scale_1, scale_2: data.scale_2,
        rot_0: data.rot_0, rot_1: data.rot_1, rot_2: data.rot_2, rot_3: data.rot_3,
    };
    
    // Ensure this doesn't panic even with extreme floats (NaN, Inf, etc.)
    let _ = SplatPoint::from_ply(&p);
});
