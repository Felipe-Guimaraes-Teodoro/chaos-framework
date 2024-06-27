use std::ops::Add;
use glam::{vec2, vec3, vec4, Vec2, Vec3, Vec4};
use rand::prelude::*;

pub fn distance(a: f32, b: f32) -> f32{
    return f32::sqrt(a*a + b*b);
}

pub fn lerp(min: f32, max: f32, t: f32) -> f32{
    return min + (max - min) * t;
}

// generates a random value T between n1: T and n2: T
pub fn rand_betw
<
    T: std::cmp::PartialOrd +
    rand::distributions::uniform::SampleUniform,
>
(
    n1: T, 
    n2: T
) -> T {
    let mut r = thread_rng();
    r.gen_range(n1..n2)
}

pub fn rand_vec2() -> Vec2 {
    vec2(rand_betw(0.0, 1.0), rand_betw(0.0, 1.0))
}

pub fn rand_vec3() -> Vec3 {
    vec3(rand_betw(0.0, 1.0), rand_betw(0.0, 1.0), rand_betw(0.0, 1.0))
}

pub fn rand_vec4() -> Vec4 {
    vec4(rand_betw(0.0, 1.0), rand_betw(0.0, 1.0), rand_betw(0.0, 1.0), rand_betw(0.0, 1.0))
}

struct SecondOrderDynamics {

}
/* 
impl Add<f32> for i32 {
    type Output = i32;

    fn add(self, rhs: f32) -> Self::Output {
        return self as f32 + rhs;
    }
}
*/