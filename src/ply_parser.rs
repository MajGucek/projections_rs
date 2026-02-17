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
    #[allow(unused)] // May come in handy?
    file_name: String,
}

impl PlyObject {
    pub fn try_load(file_name: String) -> Result<Self, String> {
        let f = File::open(&file_name).map_err(|err| format!("{:?}", err))?;
        let mut f = BufReader::new(f);

        let parser = Parser::<DefaultElement>::new();
        let ply: Ply<DefaultElement> = parser.read_ply(&mut f).map_err(|err| format!("{:?}", err))?;

        let vertex_list = ply.payload.get("vertex").ok_or("no vertex field in payload!")?;
        let mut vertices = Vec::with_capacity(vertex_list.len());
        for vertex in vertex_list {
            let x = vertex.get_float(&"x".to_owned()).ok_or("No x found!")?;
            let y = vertex.get_float(&"y".to_owned()).ok_or("No y found!")?;
            let z = vertex.get_float(&"z".to_owned()).ok_or("No z found!")?;
            vertices.push(Vector { x, y, z} );
        }

        let face_list = ply.payload.get("face").ok_or("no face field in payload")?;
        let mut faces = Vec::with_capacity(face_list.len());
        for face in face_list {
            let indices = face.get_list_int(&"vertex_indices".to_owned()).ok_or("No vertex_indices found!")?;
            if indices.len() == 3 {
                faces.push(
                    [indices[0] as usize, indices[1] as usize, indices[2] as usize]
                );
            }
        }


        Ok(Self {
            vertices,
            face_indices: faces,
            color: RGB::default(),
            transform: Transform::default(),
            file_name
        })
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
        self.face_indices.iter().map(|indexes| {
            TriFace::from((
                [vertexes[indexes[0]], vertexes[indexes[1]], vertexes[indexes[2]]],
                self.color
            ))
        }).collect()
    }
}