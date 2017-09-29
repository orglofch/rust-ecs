extern crate cgmath;
extern crate gl;

use cgmath::{Point3, Vector2, Vector3};
use graphics::material::Material;
use graphics::shader::Shader;
use std::ffi::CStr;
use std::mem::size_of;
use std::os::raw::c_void;
use std::ptr;

// TODO(orglofch): Use macros.rs
macro_rules! c_str {
    ($literal:expr) => {
        CStr::from_bytes_with_nul_unchecked(concat!($literal, "\0").as_bytes())
    }
}

macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(ptr::null() as *const $ty)).$field as *const _ as usize
    }
}

#[derive(Debug)]
pub struct Vertex {
    pub position: Point3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coords: Vector2<f32>,
}

/**
 * A mesh object which stores unique vertex information
 * along with faces indexes stored contiguously.
 *
 * TODO(orglofch): Delete the GPU loaded data afterwards.
 */
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub materials: Vec<Material>,

    vao: u32,
    vbo: u32,
    ebo: u32,
}

impl Mesh {
    /** Create a new mesh from a set of vertex information and triangulated indices. */
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, materials: Vec<Material>) -> Mesh {
        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;

        // Initialize gl buffers and bindings.
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            let size = (vertices.len() * size_of::<Vertex>()) as isize;
            let data = &vertices[0] as *const Vertex as *const c_void;
            gl::BufferData(gl::ARRAY_BUFFER, size, data, gl::STATIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            let size = (indices.len() * size_of::<u32>()) as isize;
            let data = &indices[0] as *const u32 as *const c_void;
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size, data, gl::STATIC_DRAW);

            let size = size_of::<Vertex>() as i32;
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0,
                                    3,
                                    gl::FLOAT,
                                    gl::FALSE,
                                    size,
                                    offset_of!(Vertex, position) as *const c_void);
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1,
                                    3,
                                    gl::FLOAT,
                                    gl::FALSE,
                                    size,
                                    offset_of!(Vertex, normal) as *const c_void);
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2,
                                    2,
                                    gl::FLOAT,
                                    gl::FALSE,
                                    size,
                                    offset_of!(Vertex, tex_coords) as *const c_void);

            gl::BindVertexArray(0);
        }

        Mesh {
            vertices: vertices,
            indices: indices,
            materials: materials,
            vao: vao,
            vbo: vbo,
            ebo: ebo,
        }
    }

    pub unsafe fn render(&self, shader: &Shader) {
        debug_assert!(self.materials.len() <= 1);

        // Bind the material textures.
        for material in self.materials.iter() {
            match material.diffuse_texture {
                Some(ref texture) => {
                    gl::Uniform1i(gl::GetUniformLocation(shader.id,
                                                         c_str!("diffuse_texture").as_ptr()),
                                  0);
                    gl::BindTexture(gl::TEXTURE_2D, texture.id);
                }
                None => (),
            }
        }

        gl::BindVertexArray(self.vao);
        gl::DrawElements(gl::TRIANGLES,
                         self.indices.len() as i32,
                         gl::UNSIGNED_INT,
                         ptr::null());
        gl::BindVertexArray(0);
    }
}
