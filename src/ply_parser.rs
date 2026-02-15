use std::fs::File;
use std::io::BufReader;
use ply_rs::parser::Parser;
use ply_rs::ply::{DefaultElement, Ply, PropertyAccess};
use crate::math::algebra_r3::{Transform, Vector};
use crate::math::math_lib::{Shape, TriFace, RGB};

pub struct PlyObject {
    vertices: Vec<Vector>,
    face_indices: Vec<[usize; 3]>,
    color: RGB,
    transform: Transform,
    file_name: String,
}

impl PlyObject {
    pub fn new(file_name: String) -> Self {
        let f = File::open(&file_name).unwrap();
        let mut f = BufReader::new(f);

        let parser = Parser::<DefaultElement>::new();
        let ply: Ply<DefaultElement> = parser.read_ply(&mut f).unwrap();

        let mut vertices = vec![];
        if let Some(vertex_list) = ply.payload.get("vertex") {
            for vertex in vertex_list {
                let x = vertex.get_float(&"x".to_owned()).unwrap();
                let y = vertex.get_float(&"y".to_owned()).unwrap();
                let z = vertex.get_float(&"z".to_owned()).unwrap();
                vertices.push(Vector { x, y, z} );
            }
        }
        let mut faces = vec![];
        if let Some(face_list) = ply.payload.get("face") {
            for face in face_list {
                let indices = face.get_list_int(&"vertex_indices".to_owned()).unwrap();
                if indices.len() == 3 {
                    let idxs = [indices[0] as usize, indices[1] as usize, indices[2] as usize];
                    faces.push(idxs);
                }
            }
        }


        Self {
            vertices,
            face_indices: faces,
            color: RGB::default(),
            transform: Transform::default(),
            file_name
        }
    }
}


impl Shape for PlyObject {
    fn get_color(&self) -> &RGB {
        &self.color
    }

    fn get_vertices(&self) -> &[Vector] {
        &self.vertices
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    fn produce_mesh(&self) -> Vec<TriFace> {
        let vertexes = self.transform_vectors(5000.);
        self.face_indices.iter().map(|idxs| {
            let verts = [vertexes[idxs[0]], vertexes[idxs[1]], vertexes[idxs[2]]];
            let tri = TriFace::from((verts, self.color));

            tri
        }).collect()
    }
}