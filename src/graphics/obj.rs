extern crate cgmath;
extern crate image;

use cgmath::{InnerSpace, Point3, Vector2, Vector3, Zero};
use graphics::material::Material;
use graphics::mesh::{Mesh, Vertex};
use graphics::texture::Texture;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

static MATERIAL_PATH: &'static str = "data/materials";
static TEXTURE_PATH: &'static str = "data/textures";

/**
 * Load a Mesh from a wavefront .obj file.
 *
 * Expects faces to be triangulated.
 *
 * TODO(orglofch): Add better error handling.
 */
pub fn load(obj_path: &str) -> Mesh {
    let mut positions: Vec<Point3<f32>> = Vec::new();
    let mut normals: Vec<Vector3<f32>> = Vec::new();
    let mut tex_coords: Vec<Vector2<f32>> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut material_indices_by_name: HashMap<String, u32> = HashMap::new();

    let file = File::open(obj_path).expect(&format!("Failed to to open {}", obj_path));

    let reader = BufReader::new(file);

    let mut active_mat_index = None;

    for line in reader.lines() {
        let mut words = match line {
            Ok(ref line) => line[..].split_whitespace(),
            Err(e) => panic!("Failed to read obj line due to {}", e),
        };

        match words.next() {
            Some("#") | None => (), // Comment or nothing.
            Some("v") => {
                // Vertex.
                let (v1, v2, v3) = (words.next(), words.next(), words.next());
                positions.push(parse_vertex_position(v1, v2, v3));
            }
            Some("vt") => {
                // Vertex texture.
                let (tx1, tx2) = (words.next(), words.next());
                tex_coords.push(parse_tex_coords(tx1, tx2));
            }
            Some("vn") => {
                // Vertex normal.
                let (n1, n2, n3) = (words.next(), words.next(), words.next());
                normals.push(parse_vertex_normal(n1, n2, n3));

            }
            Some("f") => {
                // Face.
                // TODO(orglofch): Make this work for arbitrary faces.
                let (f1, f2, f3) = (words.next(), words.next(), words.next());
                debug_assert!(words.next().is_none()); // Only triangulated faces.
                faces.push(parse_face(f1, f2, f3, active_mat_index));
            }
            Some("g") => (), // Group. TODO(orglofch): We assume there's 1 right now.
            Some("s") => (), // Smooth shading.
            Some("o") => (), // Object.
            Some("usemtl") => {
                // Object material.
                let name = words.next().unwrap().to_owned();
                active_mat_index = match material_indices_by_name.get(&name) {
                    Some(&i) => Some(i),
                    None => panic!("Referencing material {} which was never loaded", name),
                };
            }
            Some("mtllib") => {
                // Material library.
                let file = words.next();
                let name = match words.next() {
                    Some(name) => name.to_owned(),
                    None => format!("material{}", materials.len()),
                };
                material_indices_by_name.insert(name, materials.len() as u32);
                materials.push(read_material(file.unwrap()));
            }
            Some(token) => panic!("Invalid obj token {}", token),
        }
    }
    return reindex_faces(positions, normals, tex_coords, faces, materials);
}


// TODO(orglofch): Maybe make custom hash.
#[derive(Eq, Hash, PartialEq)]
struct FaceIndex {
    p_index: u32,
    tx_index: Option<u32>,
    n_index: Option<u32>,
}

struct Face {
    indices: Vec<FaceIndex>,
    mat_index: Option<u32>,
}

impl Face {
    /** Generate a face normal, given a set of positions. */
    pub fn normal(&self, positions: &Vec<Point3<f32>>) -> Vector3<f32> {
        let p0 = positions[self.indices[0].p_index as usize - 1];
        let e1 = positions[self.indices[1].p_index as usize - 1] - p0;
        let e2 = positions[self.indices[2].p_index as usize - 1] - p0;

        e1.cross(e2).normalize()
    }
}

/** Parses a .obj vertex line into x, y, z position. */
fn parse_vertex_position(v1: Option<&str>, v2: Option<&str>, v3: Option<&str>) -> Point3<f32> {
    let (x, y, z) = match (v1, v2, v3) {
        (Some(v1), Some(v2), Some(v3)) => {
            (v1.parse::<f32>().unwrap(), v2.parse::<f32>().unwrap(), v3.parse::<f32>().unwrap())
        }
        _ => panic!("Could not parse {:?} {:?} {:?} as position", v1, v2, v3),
    };
    Point3::new(x, y, z)
}

/** Parses a .obj normal line into x, y, z normals. */
fn parse_vertex_normal(n1: Option<&str>, n2: Option<&str>, n3: Option<&str>) -> Vector3<f32> {
    let (x, y, z) = match (n1, n2, n3) {
        (Some(n1), Some(n2), Some(n3)) => {
            (n1.parse::<f32>().unwrap(), n2.parse::<f32>().unwrap(), n3.parse::<f32>().unwrap())
        }
        _ => panic!("Could not parse {:?} {:?} {:?} as normal", n1, n2, n3),
    };
    Vector3::new(x, y, z)
}

/** Parses a .obj tex-coord line into s, t texture coordinates. */
fn parse_tex_coords(tx1: Option<&str>, tx2: Option<&str>) -> Vector2<f32> {
    let (s, t) = match (tx1, tx2) {
        (Some(tx1), Some(tx2)) => (tx1.parse::<f32>().unwrap(), tx2.parse::<f32>().unwrap()),
        _ => panic!("Could not parse {:?} {:?} as tex-coords", tx1, tx2),
    };
    Vector2::new(s, t)
}

/** Parses a single vertex index for a face. */
fn parse_face_index(vertex: &str) -> FaceIndex {
    let mut indices = vertex.split('/');

    let p_index = indices
        .next()
        .and_then(|i| i.parse::<u32>().ok())
        .expect("A face vertex must contain a position index");
    let tx_index = indices.next()
        // A vertex with a position and normal may have an empty texture coordinate.
        .and_then(|i| if i.is_empty() { None } else { i.parse::<u32>().ok() })
        .or(None);
    let n_index = indices.next().and_then(|i| i.parse::<u32>().ok()).or(None);

    FaceIndex {
        p_index: p_index,
        tx_index: tx_index,
        n_index: n_index,
    }
}

/** Parses a .obj face line into an object container. */
fn parse_face(f1: Option<&str>,
              f2: Option<&str>,
              f3: Option<&str>,
              active_mat_index: Option<u32>)
              -> Face {
    match (f1, f2, f3) {
        (Some(v1), Some(v2), Some(v3)) => {
            let indices = vec![parse_face_index(v1),
                               parse_face_index(v2),
                               parse_face_index(v3)];
            Face {
                indices: indices,
                mat_index: active_mat_index,
            }
        }
        _ => panic!("Could not parse {:?} {:?} {:?} as face", f1, f2, f3),
    }
}

/** Reads a .mtl material file. */
fn read_material(filename: &str) -> Material {
    let material_path = Path::new(MATERIAL_PATH).join(filename);
    // TODO(orglofch): Better expect
    let file = File::open(material_path).expect(""); //&format!("Failed to to open {:?}", material_path));

    let reader = BufReader::new(file);

    let mut diffuse_texture = None;

    for line in reader.lines() {
        let mut words = match line {
            Ok(ref line) => line[..].split_whitespace(),
            Err(e) => panic!("Failed to read obj line due to {}", e),
        };

        match words.next() {
            Some("#") | None => (), // Comment or empty line.
            Some("newmtl") => (),
            Some("Ka") => (), // Ambient Colour.
            Some("Kd") => (), // Diffuse Colour.
            Some("Ks") => (), // Specular Colour.
            Some("Ns") => (), // Specular Exponent.
            Some("map_Ka") => (), // Ambient Texture Map.
            Some("map_Kd") => {
                // Diffuse Texture Map.
                // TODO(orglofch): Read options and args.
                let texture_file = words.next().expect("Expected file after map_Kd");
                let texture_path = Path::new(TEXTURE_PATH).join(texture_file);
                let img =
                    image::open(&texture_path)
                        .expect(&format!("Failed to load texture {}", texture_path.display()));

                // TODO(orglofch): Make safe?
                diffuse_texture = unsafe { Some(Texture::new(img)) };
            }
            Some("map_Ks") => {
                // Specular Texture Map.
            }
            Some("map_Ns") => (), // Specular Exponent Map.
            Some(token) => panic!("Invalid obj token {}", token),
        }
    }

    Material { diffuse_texture }
}

/**
 * Reindex faces into a Mesh with a single index buffer.
 *
 * OpenGL can only support a single index buffer so we rearrange the vertex
 * index buffer into unique vertices with respect to face indexes.
 */
fn reindex_faces(positions: Vec<Point3<f32>>,
                 normals: Vec<Vector3<f32>>,
                 tex_coords: Vec<Vector2<f32>>,
                 faces: Vec<Face>,
                 materials: Vec<Material>)
                 -> Mesh {
    // Fill a single index buffer by gathering unique vertices.
    // and arranging them into the buffers.

    // Maps face vertices into their unique index into the new mesh.
    // TODO(orglofch): Reserve and shrink.
    let mut final_index_by_face_index: HashMap<FaceIndex, u32> = HashMap::new();

    let mut final_vertices: Vec<Vertex> = Vec::new();
    let mut final_indices: Vec<u32> = Vec::new();

    for face in faces {
        // Generate the face normal in case it's necessary.
        // TODO(orglofch): Make this lazy.
        let face_normal = face.normal(&positions);

        // Reindex the indices.
        for index in face.indices {
            match final_index_by_face_index.get(&index) {
                Some(&i) => final_indices.push(i),
                None => {
                    // Note OBJS are 1 indexed, hence the need to subtract 1.

                    let position = positions[index.p_index as usize - 1];

                    // If the texture coordinates are provided then use them, otherwise
                    // use a zeroed tex-coord.
                    let tex_coords = match index.tx_index {
                        Some(i) => tex_coords[i as usize - 1],
                        None => Vector2::zero(),
                    };

                    // If the normal is provided then use it, otherwise use the face normal.
                    let normal = match index.n_index {
                        Some(i) => normals[i as usize - 1],
                        None => face_normal,
                    };

                    let new_vertex = Vertex {
                        position: position,
                        normal: normal,
                        tex_coords: tex_coords,
                    };

                    let new_index = final_vertices.len() as u32;
                    final_indices.push(new_index);
                    final_index_by_face_index.insert(index, new_index);
                    final_vertices.push(new_vertex);
                }
            }
        }
    }
    Mesh::new(final_vertices, final_indices, materials)
}
