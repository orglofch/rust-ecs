extern crate cgmath;

use cgmath::{Quaternion, Vector3};

pub struct Camera {
    position: Vector3<f32>,
    rotation: Quaternion,

    front: Vector<f32>,
    up: Vector3<f32>,

    fov: f32,
    zoom: f32,
}
